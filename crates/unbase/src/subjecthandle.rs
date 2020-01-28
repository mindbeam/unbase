use crate::{
    context::Context,
    error::{
        WriteError,
        RetrieveError
    },
    memorefhead::{
        MemoRefHead,
    },
    slab::{
        RelationSlotId,
        MemoId,
        SlabHandle,
        MemoBody,
        RelationSet,
        EdgeSet,
        SubjectId,
        SubjectType
    },
};

use std::fmt;
use std::collections::HashMap;
use futures::{
    channel::mpsc,
};

use tracing::debug;


// TODO - merge rename SubjectHandle to Entity
#[derive(Clone)]
pub struct SubjectHandle {
    //TODO - remove the redundancy between id and head.subject_id()
    pub id: SubjectId,
    pub (crate) head: MemoRefHead,
    pub (crate) context: Context
}

impl SubjectHandle{
    pub async fn new ( context: &Context, vals: HashMap<String, String> ) -> Result<SubjectHandle,WriteError> {

        let slab: &SlabHandle = &context.slab;
        let id = slab.generate_subject_id(SubjectType::Record);

        debug!("SubjectHandle({}).new()",id);

        let head = slab.new_memo(
            Some(id),
            MemoRefHead::Null,
            MemoBody::FullyMaterialized {v: vals, r: RelationSet::empty(), e: EdgeSet::empty(), t: id.stype.clone() }
        ).to_head();

        context.update_indices(id, &head ).await?;

        let handle = SubjectHandle{
            id: id,
            head: head,
            context: context.clone()
        };

        Ok(handle)
    }
    pub async fn new_blank ( context: &Context ) -> Result<SubjectHandle,WriteError> {
        Self::new( context, HashMap::new() ).await
    }
    pub async fn new_kv ( context: &Context, key: &str, value: &str ) -> Result<SubjectHandle,WriteError> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals ).await
    }
    pub async fn get_value ( &mut self, key: &str ) -> Result<Option<String>,RetrieveError> {

        self.context.mut_update_head_for_consistency( &mut self.head ).await?;

        self.head.get_value(&self.context.slab, key).await
    }
    pub async fn get_edge ( &mut self, key: RelationSlotId ) -> Result<Option<SubjectHandle>, RetrieveError> {

        self.context.mut_update_head_for_consistency( &mut self.head ).await?;

        match self.head.get_edge(&self.context.slab, key).await? {
            Some(head) => {
                Ok( Some( self.context.get_subject_from_head(head).await? ) )
            },
            None => {
                Ok(None)
            }
        }
    }
    pub async fn get_relation ( &mut self, key: RelationSlotId ) -> Result<Option<SubjectHandle>, RetrieveError> {

        self.context.mut_update_head_for_consistency( &mut self.head ).await?;

        match self.head.get_relation(&self.context.slab, key).await? {
            Some(rel_subject_id) => {
                self.context.get_subject(rel_subject_id).await
            },
            None => Ok(None)
        }
    }
    pub async fn set_value (&mut self, key: &str, value: &str) -> Result<(),WriteError> {

        self.head.set_value(&self.context.slab, key, value).await?;

        // Update our indices before returning to ensure that subsequence queries against this context are self-consistent
        self.context.update_indices(self.id, &self.head ).await?;

        Ok(())
    }
    pub async fn set_relation (&mut self, key: RelationSlotId, relation: &Self) -> Result<(),WriteError> {
        self.head.set_relation(&self.context.slab, key, &relation.head).await?;

        // Update our indices before returning to ensure that subsequence queries against this context are self-consistent
        self.context.update_indices(self.id, &self.head ).await?;

        Ok(())
    }
    pub async fn get_all_memo_ids ( &self ) -> Result<Vec<MemoId>,RetrieveError> {
        self.head.get_all_memo_ids( self.context.slab.clone() ).await
    }
    pub fn observe (&self) -> mpsc::Receiver<MemoRefHead> {
        let (tx, rx) = mpsc::channel(1000);

        // get an initial value, rather than waiting for the value to change?
//        tx.send( self.head.clone() ).wait().unwrap();

        // BUG HERE? - not applying MRH to our head here, but double check as to what we were expecting from indexes
        self.context.slab.observe_subject( self.id, tx );

        rx
    }

}

// TODO POSTMERGE dig into https://docs.rs/futures-signals/0.3.11/futures_signals/tutorial/index.html and think about API
//struct SubjectState {
//    subject: SubjectHandle,
//}

impl fmt::Debug for SubjectHandle {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &self.head)
            .finish()
    }
}

impl Drop for SubjectHandle {
    fn drop (&mut self) {
        //println!("# Subject({}).drop", &self.id);
        // TODO: send a drop signal to the owning context via channel
        // self.drop_channel.send(self.id);
    }
}