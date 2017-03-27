//mod subject_graph;
//mod topo_subject_head_iter;

use std::fmt;
use std::collections::HashMap;
use slab::Slab;
use memoref::MemoRef;
use memorefhead::MemoRefHead;
use error::RetrieveError;
use index::IndexFixed;
//use self::subject_graph::*;

use subject::*;
use std::sync::{Mutex,Arc,Weak};

pub struct ContextShared {
    //This is for consistency model enforcement
    subject_heads: HashMap<SubjectId, MemoRefHead>,

    //This is for compaction of the subject_heads
    //subject_graph : SubjectGraph,

    //This is for active subjects / subject subscription management
    subjects: HashMap<SubjectId, WeakSubject>

}

pub struct ContextInner {
    slab: Slab,
    shared: Mutex<ContextShared>,
    root_index: Mutex<Option<IndexFixed>>
}
#[derive(Clone)]
pub struct Context {
    inner: Arc<ContextInner>
}

pub struct WeakContext {
    inner: Weak<ContextInner>
}


impl Context{
    pub fn new ( slab: &Slab ) -> Context {

        let new_self = Context {
            inner: Arc::new(ContextInner {
                slab: slab.clone(),
                root_index: Mutex::new(None),
                shared: Mutex::new(ContextShared {
                    subject_heads: HashMap::new(),
                    //subject_graph: SubjectGraph::new(),
                    subjects: HashMap::new(),
                })
            })
        };

        let index = IndexFixed::new_from_memorefhead(&new_self, 5, slab.get_root_index_seed().expect("Uninitialized slab") );

        {
            *new_self.inner.root_index.lock().unwrap() = Some(index);
        }

        new_self
    }
    pub fn insert_into_root_index (&self, subject_id: SubjectId, subject: &Subject) {
        if let Some(ref index) = *self.inner.root_index.lock().unwrap() {
            index.insert(subject_id,subject);
        }else{
            panic!("no root index")
        }
    }
    pub fn add (&self, mut memorefs: Vec<MemoRef>) {
        let mut shared = self.inner.shared.lock().unwrap();

        // TODO: trim existing context based on descendants

        for memoref in memorefs.drain(0..) {
            let subject_id = memoref.get_memo(&self.inner.slab).unwrap().subject_id;

            println!("# Context calling apply_memoref");
            shared.subject_heads.entry(subject_id).or_insert( MemoRefHead::new() ).apply_memoref(&memoref, &self.inner.slab);
        }
    }
    pub fn get_slab (&self) -> &Slab {
        &self.inner.slab
    }
    pub fn subscribe_subject (&self, subject: &Subject) {
        {
            let mut shared = self.inner.shared.lock().unwrap();
            shared.subjects.insert( subject.id, subject.weak() );
        }
        self.inner.slab.subscribe_subject(subject.id, self);
    }
    pub fn unsubscribe_subject (&self, subject_id: SubjectId ){
        println!("# Context.unsubscribe_subject({})", subject_id);

    /*
    BUG/TODO: Temporarily disabled unsubscription
    1. Because it was causing deadlocks on the context AND slab mutexes
       when the thread in the test case happened to drop the subject
       when we were busy doing apply_subject_head, which locks context,
       and is called by slab – so clearly this is untenable
    2. It was always sort of a hack that the subject was managing subscriptions
       in this way anyways. Lets put together a more final version of the subscriptions
       before we bother with fixing unsubscription

        {
            let mut shared = self.inner.shared.lock().unwrap();
            shared.subjects.remove( &subject_id );
        }

        self.inner.slab.unsubscribe_subject(subject_id, self);
        println!("# Context.unsubscribe_subject({}) - FINISHED", subject_id);
        */

    }
    pub fn get_subject_by_id (&self, subject_id: SubjectId) -> Result<Subject, RetrieveError> {

        match *self.inner.root_index.lock().unwrap() {
            Some(ref index) => index.get(subject_id),
            None            => Err(RetrieveError::IndexNotInitialized)
        }
    }

    pub fn get_subject_with_head (&self, subject_id: SubjectId, mut head: MemoRefHead) -> Result<Subject, RetrieveError> {
        println!("# Context.get_subject_with_head({},{:?})", subject_id, head.memo_ids() );

        if head.len() == 0 {
            return Err(RetrieveError::InvalidMemoRefHead);
        }

        {
            let mut shared = self.inner.shared.lock().unwrap();

            if let Some(relevant_context_head) = shared.subject_heads.get(&subject_id) {
                println!("# \\ Relevant context head is ({:?})", relevant_context_head.memo_ids() );

                head.apply( relevant_context_head, &self.inner.slab );

            }else{
                println!("# \\ No relevant head found in context");
            }

            match shared.get_subject_if_resident(subject_id) {
                Some(ref mut subject) => {
                    subject.apply_head(&head);
                    return Ok(subject.clone());
                }
                None =>{}
            }
        }

        // NOTE: Subject::reconstitute calls back to Context.subscribe_subject()
        //       so we need to release the mutex prior to this
        let subject = Subject::reconstitute(self,head);
        return Ok(subject);

    }
    // specifically for created/updated subjects
    // Called by Subject::new, set_*
    pub fn subject_updated (&self, subject_id: SubjectId, head: &MemoRefHead){
        let mut shared = self.inner.shared.lock().unwrap();

        let my_subject_head = shared.subject_heads.entry(subject_id).or_insert( MemoRefHead::new() );
        my_subject_head.apply(head, &self.inner.slab);

        // Necessary bookkeeping for topological traversal
        //shared.subject_graph.update( &self.inner.slab, subject_id, my_subject_head.project_all_relation_links( &self.inner.slab ));

    }
    // Called by the Slab whenever memos matching one of our subscriptions comes in
    pub fn apply_subject_head (&self, subject_id: SubjectId, head: &MemoRefHead){

        // NOTE: In all liklihood, there is significant room to optimize this.
        //       We're applying heads to heads redundantly

        //QUESTION: Should we be updating our query context here?
        //          not sure if this should happen implicitly or require explicit context exchange
        //          I think there's a pretty strong argument for implicit, but I want to think
        //          about this a bit more before I say yes for certain.
        //
        //ANSWER:   It occurs to me that we're only getting subject heads from the slab which we expressly
        //          subscribed to, so this strengthens the case quite a bit

        // Have to make sure the subject we retrieve
        // doesn't go out of scope while we're locked, or we'll deadlock
        let _maybe_subject : Option<Subject>;

        {
            let mut shared = self.inner.shared.lock().unwrap();

            if let Some(mut subject) = shared.get_subject_if_resident(subject_id) {
                subject.apply_head(head);

                _maybe_subject = Some(subject);
            }

            // TODO: It probably makes sense to stop playing telephone between the context and the subject
            //       And simply use an Arc<Mutex<MemoRefHead>> which is shared between the subject and the context
            //       We both have it around the same time really. To do otherwise would be silly
            //       The main question is: what threading model do we want to optimize for?
            //       Will the context usually / always be in the same thread as the subjects?
            //       If so, then switch to Rc and screw this Arc<Mutex<>> business
            //       If not, then this really makes me wonder about whether the clone of the MemoRefHead
            //       and the duplicate work of merging it twice might actually make sense vs having to cross
            //       the thread bountary to retrieve the data we want ( probably not, but asking anway)

            let my_subject_head = shared.subject_heads.entry(subject_id).or_insert( MemoRefHead::new() );
            my_subject_head.apply(&head, &self.inner.slab);

            // Necessary bookkeeping for topological traversal
            // TODO: determine if it makes sense to calculate only the relationship diffs to minimize cost
            //shared.subject_graph.update( &self.inner.slab, subject_id, my_subject_head.project_all_relation_links( &self.inner.slab ));
        }
    }

    pub fn cmp (&self, other: &Self) -> bool{
        // stable way:
        &*(self.inner) as *const _ != &*(other.inner) as *const _

        // unstable way:
        //Arc::ptr_eq(&self.inner,&other.inner)
    }
    pub fn weak (&self) -> WeakContext {
        WeakContext {
            inner: Arc::downgrade(&self.inner)
        }
    }
    /*
    Putting this on hold for now
    pub fn topo_subject_head_iter (&self) -> TopoSubjectHeadIter {
        TopoSubjectHeadIter::new( &self )
    }*/

    // Subject A -> B -> E
    //          \-> C -> F
    //          \-> D -> G
    //
    // Steps:
    //  1. iterate over context subject heads, starting with leaves, working to the root
    //     NOTE: This may not form a contiguous tree, as we're dealing with memos
    //     which have been delivered from other slabs too, not just local edits
    //     NOTE: We can actually have referential cycles here, because a subject
    //     is not just a DAG of Memos, but rather the projection of a DAG *plus* whatever
    //     is in our context. If we tried to continuously materialize such a structure,
    //     it would generate an infinite number of memos - so we'll need to break cycles.
    //  2. Materialize each subject head in ascending topological order
    //  3. If any other context subject heads reference the subject head materialized
    //     Issue a relation edit referencing it (ensuring that it gets added to the context)
    //     and drop the materialized subject head from the context.
    //  4. Continue until the list is exhausted, or a cycle is detected
    //
    // subject_relation_map:
    // E: []
    // B: [E]
    // A: [B]
    // etc
/*
    pub fn fully_compress (&self) {

        let slab = self.get_slab();

        // Iterate the contextualized subject heads in reverse topological order
        for (head, ref_by, ref_to) in self.topo_subject_head_iter().rev() {
            // Materialization not really necessary for compression ( I think )
            // try to materialize it (create a memo that flattens known preceeding operations)
            // head.fully_materialize( &slab );

            // OK we did compress and issue a new "Materialized" memo
            // ( it should really only be one Memo in the new MemoRefHead,
            // but assuming that would limit flexibility, and destandardize our handling)

            if ref_by.len() > 0 {
                // OK, somebody is pointing to us, so lets issue an edit for them
                // to point to the new materialized memo for their relevant relations
                self.repoint_subject_relations(ref_by, materialized_head);

                // Now that we know they are pointing to the new materialized MemoRefHead,
                // and that the resident subject struct we have is already updated, we can
                // remove this subject MemoRefHead from the context head, because subsequent
                // index/graph traversals should find this updated parent.
                //
                // When trying to materialize/compress fully (not that we'll want to do this often),
                // this would continue all the way to the root index node, and we should be left
                // with a very small context head

                self.remove(subject) // should be removed from the context
            }
        }

    }
    */
    pub fn is_fully_materialized (&self) -> bool {

        for (_,head) in self.inner.shared.lock().unwrap().subject_heads.iter() {
            if ! head.is_fully_materialized(&self.inner.slab) {
                return false
            }
        }

        return true;

    }
}

impl ContextShared {
    fn get_subject_if_resident (&mut self, subject_id: SubjectId) -> Option<Subject> {

        if let Some(weaksub) = self.subjects.get_mut(&subject_id) {
            if let Some(subject) = weaksub.upgrade() {
                //NOTE: In theory we shouldn't need to apply the current context
                //      to this subject, as it shouldddd have already happened
                return Some(subject);
            }
        }

        None
    }

}

impl Drop for ContextShared {
    fn drop (&mut self) {
        println!("# ContextShared.drop");
    }
}
impl fmt::Debug for ContextShared {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("ContextShared")
            .field("subject_heads", &self.subject_heads)
            // TODO: restore Debug for WeakSubject
            //.field("subjects", &self.subjects)
            .finish()
    }
}
impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.inner.shared.lock().unwrap();

        fmt.debug_struct("Context")
            .field("inner", &shared)
            .finish()
    }
}

impl WeakContext {
    pub fn upgrade (&self) -> Option<Context> {
        match self.inner.upgrade() {
            Some(i) => Some( Context { inner: i } ),
            None    => None
        }
    }
    pub fn cmp (&self, other: &WeakContext) -> bool{
        if let Some(context) = self.upgrade() {
            if let Some(other) = other.upgrade(){
                // stable way:
                &*(context.inner) as *const _ != &*(other.inner) as *const _

                // unstable way:
                //Arc::ptr_eq(&context.inner,&other.inner)
            }else{
                false
            }
        }else {
            false
        }


    }
}
