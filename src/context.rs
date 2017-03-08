use std::fmt;
use std::collections::HashMap;
use slab::Slab;
use memoref::MemoRef;
use memorefhead::MemoRefHead;
use error::RetrieveError;
use index::IndexFixed;

use subject::*;
use std::sync::{Mutex,Arc,Weak};

pub struct ContextShared {
    subject_heads: HashMap<SubjectId, MemoRefHead>,
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
                    subjects: HashMap::new()
                })
            })
        };

        let index = IndexFixed::new_from_memorefhead(&new_self, 5, slab.get_root_index_seed() );

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
            shared.subject_heads.entry(subject_id).or_insert( MemoRefHead::new() ).apply_memoref(memoref, &self.inner.slab);
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

        {

            let mut shared = self.inner.shared.lock().unwrap();
            shared.subjects.remove( &subject_id );
        }
        self.inner.slab.unsubscribe_subject(subject_id, self);
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

            match shared.get_subject_if_resident(&subject_id) {
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
    pub fn apply_subject_head (&self, subject_id: &SubjectId, head: &MemoRefHead){
        //QUESTION: Should we be updating our query context here? arguably yes?

        if let Some(mut subject) = self.inner.shared.lock().unwrap().get_subject_if_resident(subject_id) {
            subject.apply_head(head)
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
    pub fn subject_head_iter (&self) -> ContextSubjectHeadIter {
        // TODO: Do this in a less ridiculous way,
        //       and move it into ContextSubjectHeadIter::new
        let subject_ids : Vec<SubjectId>;
        {
            let shared = self.inner.shared.lock().unwrap();
            subject_ids = shared.subject_heads.keys().map(|k| k.to_owned()).collect();
        }
        ContextSubjectHeadIter {
            subject_ids: subject_ids,
            context: self.clone()
        }
    }
    pub fn fully_materialize (&self) {
        // Iterate over subjects ( what about cross subject links? )
        // Create said subject with the respective head
        // project said subject into a keyframe memo
        // what about partial keyframes?


        // iterate over all subjects and ask them to fully materialize
        // TODO: deconflict get_subject and get_subject_with_head ( this is broken )
        // TODO: Figure out how to materialize Subject memos without incurring the overhead of subject creation/destruction
        // TODO: ensure that materializing a given subject will cause it to update the context
        // TODO: ensure that updating the context with subject A that references the head of subject B
        //       causes subject B to be removed from the context.
        // TODO: find a way to try to perform these in
        //       referential order - child to parent. Maybe annotate the subject_heads with cross-subject descendent references?

        for (subject_id,head) in self.subject_head_iter() {
            if let Ok(ref mut subject) = self.get_subject_with_head(subject_id, head){
                subject.fully_materialize()
            }
        }


    }
    pub fn is_fully_materialized (&self) -> bool {

        for (_,head) in self.subject_head_iter() {
            if ! head.is_fully_materialized(&self.inner.slab) {
                return false
            }
        }

        return true;

    }
}

impl ContextShared {
    fn get_subject_if_resident (&mut self, subject_id: &SubjectId) -> Option<Subject> {

        if let Some(weaksub) = self.subjects.get_mut(subject_id) {
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

pub struct ContextSubjectHeadIter{
    context: Context,
    subject_ids: Vec<SubjectId>
}

impl Iterator for ContextSubjectHeadIter {
    type Item = (SubjectId, MemoRefHead);
    fn next (&mut self) -> Option<(SubjectId, MemoRefHead)> {

        //NOTE: Some pretttyy shenanegous stuff here, but taking the
        //      low road for now in the interest of time. Playing
        //      stupid games to try to avoid a deadlock with the slab
        //      inserting new memos mid-iteration via update_subject_head
        if let Some(subject_id) = self.subject_ids.pop() {
            if let Some(head) = self.context.inner.shared.lock().unwrap().subject_heads.get(&subject_id) {
                Some((subject_id,head.clone()))
            }else{
                None
            }
        }else{
            None
        }
    }
}
