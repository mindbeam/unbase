use std::fmt;
use std::collections::HashMap;
use slab::memo::*;
use memorefhead::*;
use context::{Context,ContextRef};
use error::*;
use slab::Slab;
use std::sync::{Arc,Mutex,Weak,MutexGuard};

pub type SubjectId     = u64;
pub type SubjectField  = String;
pub const SUBJECT_MAX_RELATIONS : usize = 256;

#[derive(Clone)]
pub struct Subject {
    pub id:  SubjectId,
    shared: Arc<Mutex<SubjectShared>>
}

pub struct WeakSubject {
    pub id:  SubjectId,
    shared: Weak<Mutex<SubjectShared>>
}

pub struct SubjectShared {
    id:      SubjectId,
    head:    MemoRefHead,
    contextref: ContextRef,
}

impl Subject {
    pub fn new ( context: &Context, vals: HashMap<String, String>, is_index: bool ) -> Result<Subject,String> {
        Self::new_with_contextref( ContextRef::Strong(context.clone()), vals, is_index )
    }
    pub fn new_with_contextref ( contextref: ContextRef, vals: HashMap<String, String>, is_index: bool ) -> Result<Subject,String> {
        // don't store this
        let context = contextref.get_context();

        let slab = context.get_slab();
        let subject_id = slab.inner().generate_subject_id();
        println!("# Subject({}).new()",subject_id);

        let shared = Arc::new(Mutex::new(SubjectShared{
            id: subject_id,
            head: MemoRefHead::new(),
            contextref: contextref
        }));

        let subject = Subject {
            id:      subject_id,
            shared:  shared
        };

        context.subscribe_subject( &subject );

        let memoref = slab.inner().new_memo_basic_noparent(
                Some(subject_id),
                MemoBody::FullyMaterialized {v: vals, r: HashMap::new() }
            );

        {
            subject.inner().head.apply_memoref(&memoref, slab);
            context.subject_updated( subject_id, &shared.head );
        }

        // IMPORTANT: Need to wait to insert this into the index until _after_ the first memo
        // has been issued, sent to the slab, and added to the subject head via the subscription mechanism.

        // HACK HACK HACK - this should not be a flag on the subject, but something in the payload I think
        if !is_index {
            // NOTE: important that we do this after the subject.shared.lock is released
            context.insert_into_root_index( subject_id, &subject );
        }
        Ok(subject)
    }
    pub fn reconstitute (contextref: ContextRef, head: MemoRefHead) -> Subject {

        let context = contextref.get_context();
        let subject_id = head.first_subject_id( context.get_slab() ).unwrap();

        let shared = SubjectShared{
            id: subject_id,
            head: head,
            contextref: contextref
        };

        let subject = Subject {
            id:      subject_id,
            shared: Arc::new(Mutex::new(shared))
        };

        context.subscribe_subject( &subject );

        subject
    }
    pub fn new_blank ( context: &Context ) -> Result<Subject,String> {
        Self::new( context, HashMap::new(), false )
    }
    pub fn new_kv ( context: &Context, key: &str, value: &str ) -> Result<Subject,String> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals, false )
    }
    pub fn inner (&self) -> MutexGuard<SubjectShared> {
        self.shared.lock().unwrap()
    }
    pub fn get_value ( &self, key: &str ) -> Option<String> {
        println!("# Subject({}).get_value({})",self.id,key);

        let shared = self.shared.lock().unwrap();
        shared.head.project_value(&shared.contextref.get_context(), key)
    }
    pub fn get_relation ( &self, key: RelationSlotId ) -> Result<Subject, RetrieveError> {
        println!("# Subject({}).get_relation({})",self.id,key);

        let shared = self.shared.lock().unwrap();
        let context = shared.contextref.get_context();
        match shared.head.project_relation(&context, key) {
            Ok((subject_id, head)) => context.get_subject_with_head(subject_id,head),
            Err(e)   => Err(e)

        }
    }
    pub fn set_value (&self, key: &str, value: &str) -> bool {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let my_slab;
        let memoref;
        let context;
        {
            let shared = self.shared.lock().unwrap();
            context = shared.contextref.get_context();
            my_slab = context.get_slab().clone();

            memoref = my_slab.inner().new_memo_basic(
                Some(self.id),
                shared.head.clone(),
                MemoBody::Edit(vals)
            );
        }

        let mut inner = self.inner();
        inner.head.apply_memoref(&memoref, &my_slab);
        context.subject_updated( self.id,  &inner.head );

        true
    }
    pub fn set_relation (&self, key: RelationSlotId, relation: &Self) {
        println!("# Subject({}).set_relation({}, {})", &self.id, key, relation.id);
        let mut memoref_map : HashMap<RelationSlotId, (SubjectId,MemoRefHead)> = HashMap::new();
        memoref_map.insert(key, (relation.id, relation.get_head().clone()) );

        let slab;
        let memoref;
        let context;
        {
            let shared = self.shared.lock().unwrap();
            context = shared.contextref.get_context();
            slab = context.get_slab();

            let memoref = slab.inner().new_memo(
                Some(self.id),
                shared.head.clone(),
                MemoBody::Relation(memoref_map)
            );
        }


        // TODO: determine conclusively whether it's possible for apply_memorefs
        //       to result in a retrieval that retults in a context addition that
        //       causes a deadlock
        let mut shared = self.shared.lock().unwrap();
        shared.head.apply_memoref(&memoref, &slab);
        context.subject_updated( self.id, &shared.head );

    }
    // TODO: get rid of apply_head and get_head in favor of Arc sharing heads with the context
    pub fn apply_head (&mut self, new: &MemoRefHead){
        println!("# Subject({}).apply_head({:?})", &self.id, new.memo_ids() );

        let mut shared = self.shared.lock().unwrap();
        let context = shared.contextref.get_context();
        let slab = context.get_slab().clone(); // TODO: find a way to get rid of this clone

        println!("# Record({}) calling apply_memoref", self.id);
        shared.head.apply(&new, &slab);
    }
    pub fn get_head (&self) -> MemoRefHead {
        let shared = self.shared.lock().unwrap();
        shared.head.clone()
    }
    pub fn get_all_memo_ids ( &self ) -> Vec<MemoId> {
        println!("# Subject({}).get_all_memo_ids()",self.id);
        let slab = self.shared.lock().unwrap().contextref.get_context().get_slab().clone();

        self.get_head().causal_memo_iter( &slab ).map(|m| m.id).collect()
    }
    pub fn weak (&self) -> WeakSubject {
        WeakSubject {
            id: self.id,
            shared: Arc::downgrade(&self.shared)
        }
    }
    pub fn is_fully_materialized (&self, slab: &Slab) -> bool {
        self.shared.lock().unwrap().head.is_fully_materialized(slab)
    }
    pub fn fully_materialize (&mut self, _slab: &Slab) -> bool {
        unimplemented!();
        //self.shared.lock().unwrap().head.fully_materialize(slab)
    }
}

impl Drop for SubjectShared {
    fn drop (&mut self) {
        println!("# Subject({}).drop", &self.id);
        match self.contextref {
            ContextRef::Strong(ref c) => {
                c.unsubscribe_subject(self.id);
            }
            ContextRef::Weak(ref c) => {
                match c.upgrade() {
                    Some(c) => {
                        c.unsubscribe_subject(self.id);
                    }
                    None => {}
                }
            }
        }
    }
}
impl fmt::Debug for SubjectShared {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &self.head)
            .finish()
    }
}

impl fmt::Debug for Subject {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.shared.lock().unwrap();

        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &shared.head)
            .finish()
    }
}

impl WeakSubject {
    pub fn upgrade (&self) -> Option<Subject> {
        match self.shared.upgrade() {
            Some(s) => Some( Subject { id: self.id, shared: s } ),
            None    => None
        }
    }
}
