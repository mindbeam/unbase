use core::ops::Deref;
use std::fmt;
use std::collections::HashMap;
use std::sync::{Arc,RwLock,Mutex,Weak,MutexGuard};

use slab::*;
use memorefhead::*;
use context::{Context,ContextRef};
use error::*;

pub type SubjectId     = u64;
pub type SubjectField  = String;
pub const SUBJECT_MAX_RELATIONS : usize = 256;

#[derive(Clone)]
pub struct Subject(Arc<SubjectInner>);
impl Deref for Subject {
    type Target = SubjectInner;
    fn deref(&self) -> &SubjectInner {
        &*self.0
    }
}
pub struct WeakSubject(Weak<SubjectInner>);

pub struct SubjectInner {
    pub id:     SubjectId,
    head:       RwLock<MemoRefHead>,
    contextref: ContextRef,
}

impl Subject {
    pub fn new ( context: &Context, vals: HashMap<String, String>, is_index: bool ) -> Result<Subject,String> {
        Self::new_with_contextref( ContextRef::Strong(context.clone()), vals, is_index )
    }
    pub fn new_with_contextref ( contextref: ContextRef, vals: HashMap<String, String>, is_index: bool ) -> Result<Subject,String> {
        // don't store this
        let context = contextref.get_context();

        let slab = context.slab;
        let subject_id = slab.generate_subject_id();
        println!("# Subject({}).new()",subject_id);

        let memoref = slab.new_memo_basic_noparent(
                Some(subject_id),
                MemoBody::FullyMaterialized {v: vals, r: HashMap::new() }
            );
        let head = memoref.to_head();
        // Not 100% sure we can do this before subscribing, but it saves us a clone, so lets try it!
        context.subject_updated( subject_id, &head );

        let subject = Subject(Arc::new(SubjectInner{
            id: subject_id,
            head: RwLock::new(head),
            contextref: contextref
        }));

        context.subscribe_subject( &subject );

        // HACK HACK HACK - this should not be a flag on the subject, but something in the payload I think
        if !is_index {
            // NOTE: important that we do this after the subject.shared.lock is released
            context.insert_into_root_index( subject_id, &subject );
        }
        Ok(subject)
    }
    pub fn reconstitute (contextref: ContextRef, head: MemoRefHead) -> Subject {

        let context = contextref.get_context();
        let subject_id = head.first_subject_id( context.slab ).unwrap();

        let subject = Subject(Arc::new(SubjectInner{
            id: subject_id,
            head: RwLock::new(head),
            contextref: contextref
        }));

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
    pub fn get_value ( &self, key: &str ) -> Option<String> {
        println!("# Subject({}).get_value({})",self.id,key);

        self.head.read().unwrap().project_value(&self.contextref.get_context(), key)
    }
    pub fn get_relation ( &self, key: RelationSlotId ) -> Result<Subject, RetrieveError> {
        println!("# Subject({}).get_relation({})",self.id,key);

        let context = self.contextref.get_context();
        match self.head.read().unwrap().project_relation(&context, key) {
            Ok((subject_id, head)) => context.get_subject_with_head(subject_id,head),
            Err(e)   => Err(e)

        }
    }
    pub fn set_value (&self, key: &str, value: &str) -> bool {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let context = self.contextref.get_context();
        let slab = context.slab;
        let head = self.head.write().unwrap();

        let memoref = slab.new_memo_basic(
            Some(self.id),
            head.clone(),
            MemoBody::Edit(vals)
        );

        head.apply_memoref(&memoref, &slab);
        context.subject_updated( self.id,  &head );

        true
    }
    pub fn set_relation (&self, key: RelationSlotId, relation: &Self) {
        println!("# Subject({}).set_relation({}, {})", &self.id, key, relation.id);
        let mut memoref_map : HashMap<RelationSlotId, (SubjectId,MemoRefHead)> = HashMap::new();
        memoref_map.insert(key, (relation.id, relation.get_head().clone()) );

        let context = self.contextref.get_context();
        let slab = context.slab;
        let head = self.head.write().unwrap();

        let memoref = slab.new_memo(
            Some(self.id),
            head.clone(),
            MemoBody::Relation(memoref_map)
        );

        head.apply_memoref(&memoref, &slab);
        context.subject_updated( self.id, &head );

    }
    // TODO: get rid of apply_head and get_head in favor of Arc sharing heads with the context
    pub fn apply_head (&mut self, new: &MemoRefHead){
        println!("# Subject({}).apply_head({:?})", &self.id, new.memo_ids() );

        let context = self.contextref.get_context();
        let slab = context.slab.clone(); // TODO: find a way to get rid of this clone

        println!("# Record({}) calling apply_memoref", self.id);
        self.head.write().unwrap().apply(&new, &slab);
    }
    pub fn get_head (&self) -> MemoRefHead {
        self.head.read().unwrap().clone()
    }
    pub fn get_all_memo_ids ( &self ) -> Vec<MemoId> {
        println!("# Subject({}).get_all_memo_ids()",self.id);
        let context = self.contextref.get_context();
        let slab = context.slab.clone(); // TODO: find a way to get rid of this clone
        self.head.read().unwrap().causal_memo_iter( &slab ).map(|m| m.id).collect()
    }
    pub fn weak (&self) -> WeakSubject {
        WeakSubject(Arc::downgrade(&self.0))
    }
    pub fn is_fully_materialized (&self) -> bool {
        let context = self.contextref.get_context();
        self.head.read().unwrap().is_fully_materialized(context.slab)
    }
    pub fn fully_materialize (&mut self, _slab: &Slab) -> bool {
        unimplemented!();
        //self.shared.lock().unwrap().head.fully_materialize(slab)
    }
}

impl Drop for SubjectInner {
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
impl fmt::Debug for SubjectInner {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &self.head)
            .finish()
    }
}

impl fmt::Debug for Subject {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &self.head)
            .finish()
    }
}

impl WeakSubject {
    pub fn upgrade (&self) -> Option<Subject> {
        match self.0.upgrade() {
            Some(s) => Some( Subject(s) ),
            None    => None
        }
    }
}
