use core::ops::Deref;
use std::{
    collections::HashMap,
    fmt,
    sync::{
        Arc,
        RwLock,
        Weak
    }
};
use tracing::debug;
use futures::{
    stream::StreamExt
};

use crate::slab::*;
use crate::memorefhead::*;
use crate::context::{Context,ContextRef};
use crate::error::*;

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
    pub async fn new ( context: &Context, vals: HashMap<String, String>, is_index: bool ) -> Result<Subject,String> {
        Self::new_with_contextref( ContextRef::Strong(context.clone()), vals, is_index ).await
    }
    pub async fn new_with_contextref ( contextref: ContextRef, vals: HashMap<String, String>, is_index: bool ) -> Result<Subject,String> {
        // don't store this
        let context = contextref.get_context();

        let slab = &context.slab;
        let subject_id = slab.generate_subject_id();
        debug!(%subject_id);

        let memoref = slab.new_memo_basic_noparent(
                Some(subject_id),
                MemoBody::FullyMaterialized {v: vals, r: RelationSlotSubjectHead(HashMap::new()) }
            );
        let head = memoref.to_head();

        let subject = Subject(Arc::new(SubjectInner{
            id: subject_id,
            head: RwLock::new(head),
            contextref: contextref
        }));

        context.subscribe_subject( &subject );

        // HACK HACK HACK - this should not be a flag on the subject, but something in the payload I think
        if !is_index {
            // NOTE: important that we do this after the subject.shared.lock is released
            context.insert_into_root_index( subject_id, &subject ).await;
        }
        Ok(subject)
    }
    pub fn reconstitute (contextref: ContextRef, head: MemoRefHead) -> Subject {
        let context = contextref.get_context();

        let subject_id = head.first_subject_id().unwrap();

        let subject = Subject(Arc::new(SubjectInner{
            id: subject_id,
            head: RwLock::new(head),
            contextref: contextref
        }));

        context.subscribe_subject( &subject );

        subject
    }
    pub async fn new_blank ( context: &Context ) -> Result<Subject,String> {
        Self::new( context, HashMap::new(), false ).await
    }
    #[tracing::instrument]
    pub async fn new_kv ( context: &Context, key: &str, value: &str ) -> Result<Subject,String> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals, false ).await
    }
    #[tracing::instrument]
    pub async fn get_value ( &self, key: &str ) -> Option<String> {
        self.head.read().unwrap().project_value(&self.contextref.get_context(), key).await
    }
    #[tracing::instrument]
    pub async fn get_relation ( &self, key: RelationSlotId ) -> Result<Subject, RetrieveError> {

        let context = self.contextref.get_context();
        let head = {
            self.head.read().unwrap().clone()
        };

        match head.project_relation(&context, key).await {
            Ok((subject_id, relhead)) => context.get_subject_with_head(subject_id,relhead).await,
            Err(e)   => Err(e)
        }
    }
    #[tracing::instrument]
    pub async fn set_value (&self, key: &str, value: &str) -> bool {
        //TODO: guard against race conditions between different newheads with simultaneous sets
        // Was managing this with a mutex, but can't do it in the same way given asyncification

        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let context = self.contextref.get_context();
        let slab = &context.slab;

        let head = {
            self.head.read().unwrap().clone()
        };

        let memoref = slab.new_memo_basic(
            Some(self.id),
            head,
            MemoBody::Edit(vals)
        );

        let newhead = memoref.to_head();

        context.apply_subject_head( self.id,  &newhead, false ).await;

        *(self.head.write().unwrap()) = newhead;

        true
    }
    #[tracing::instrument]
    pub async fn set_relation (&self, key: RelationSlotId, relation: &Self) {
        let mut memoref_map : HashMap<RelationSlotId, (SubjectId,MemoRefHead)> = HashMap::new();
        memoref_map.insert(key, (relation.id, relation.get_head().clone()) );

        let context = self.contextref.get_context();
        let slab = &context.slab;
        let head = {
            self.head.read().unwrap().clone()
        };

        let memoref = slab.agent.new_memo(
            Some(self.id),
            head,
            MemoBody::Relation(RelationSlotSubjectHead(memoref_map))
        );

        let newhead = memoref.to_head();

        context.apply_subject_head( self.id,&newhead, false ).await;

        *(self.head.write().unwrap()) = newhead;
    }
    // TODO: get rid of apply_head and get_head in favor of Arc sharing heads with the context
    #[tracing::instrument]
    pub async fn apply_head (&self, new: MemoRefHead){

        let context = self.contextref.get_context();
        let slab = context.slab.clone(); // TODO: find a way to get rid of this clone

        let head = {
            self.head.read().unwrap().clone()
        };

        let newhead = head.apply(&new, &slab).await;

        *(self.head.write().unwrap()) = newhead;
    }
    pub fn get_head (&self) -> MemoRefHead {
        self.head.read().unwrap().clone()
    }
    pub async fn get_all_memo_ids ( &self ) -> Vec<MemoId> {
        let context = self.contextref.get_context();
        let slab = context.slab.clone(); // TODO: find a way to get rid of this clone
        let memostream = self.head.read().unwrap().causal_memo_stream( &slab );
        memostream.map(|m| m.id).collect().await
    }
    pub fn weak (&self) -> WeakSubject {
        WeakSubject(Arc::downgrade(&self.0))
    }
    pub async fn is_fully_materialized (&self) -> bool {
        let context = self.contextref.get_context();
        self.head.read().unwrap().is_fully_materialized(&context.slab).await
    }
    pub fn fully_materialize (&self, _slab: &Slab) -> bool {
        unimplemented!();
        //self.shared.lock().unwrap().head.fully_materialize(slab)
    }
}

impl Drop for SubjectInner {
    #[tracing::instrument]
    fn drop (&mut self) {
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
            .field("id", &self.id)
//            .field("head", &self.head)
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
