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
    },
    subject::{
        Subject,
        SubjectId,
        SubjectType
    },
};

use std::fmt;
use std::collections::HashMap;
use futures::{
    channel::mpsc,
};

#[derive(Clone)]
pub struct SubjectHandle {
    pub id: SubjectId,
    pub (crate) subject: Subject,
    pub (crate) context: Context
}

impl SubjectHandle{
    pub async fn new ( context: &Context, vals: HashMap<String, String> ) -> Result<SubjectHandle,WriteError> {

        let subject = Subject::new(&context, SubjectType::Record, vals ).await?;

        let handle = SubjectHandle{
            id: subject.id,
            subject: subject,
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
        self.subject.get_value(&self.context, key).await
    }
    pub async fn get_relation ( &mut self, key: RelationSlotId ) -> Result<Option<SubjectHandle>, RetrieveError> {

        match self.subject.get_relation(&self.context, key).await?{
        Some(rel_sub_subject) => {
            Ok(Some(SubjectHandle{
                id: rel_sub_subject.id,
                context: self.context.clone(),
                subject: rel_sub_subject
            }))
            },
            None => Ok(None)
        }
    }
    pub async fn set_value (&mut self, key: &str, value: &str) -> Result<bool,WriteError> {
        self.subject.set_value(&self.context, key, value).await
    }
    pub async fn set_relation (&mut self, key: RelationSlotId, relation: &Self) -> Result<(),WriteError> {
        self.subject.set_relation(&self.context, key, &relation.subject).await
    }
    pub async fn get_all_memo_ids ( &self ) -> Result<Vec<MemoId>,RetrieveError> {
        self.subject.get_all_memo_ids( self.context.slab.clone() ).await
    }
    pub fn observe (&self) -> mpsc::Receiver<MemoRefHead> {
        self.subject.observe(&self.context.slab)
    }
}


impl fmt::Debug for SubjectHandle {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Subject")
            .field("subject_id", &self.subject.id)
            .field("head", &self.subject.head)
            .finish()
    }
}