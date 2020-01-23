
use crate::{
    error::{
        RetrieveError,
        WriteError,
    },
    memorefhead::MemoRefHead,
    subject::{
        Subject,
        SubjectId,
    },
    subjecthandle::SubjectHandle,
};

use super::Context;
use std::time::Duration;

/// Internal interface functions
impl Context {
    pub (crate) async fn insert_into_root_index(&self, subject_id: SubjectId, subject: &Subject) -> Result<(),WriteError> {
        self.root_index(Duration::from_secs(5)).await?.insert(self, subject_id.id, subject).await
    }

    /// Called by the Slab whenever memos matching one of our subscriptions comes in, or by the Subject when an edit is made
    pub (crate) async fn apply_head(&self, head: &MemoRefHead) -> Result<MemoRefHead,WriteError> {
        // println!("Context.apply_subject_head({}, {:?}) ", subject_id, head.memo_ids() );
        self.stash.apply_head(&self.slab, head)
    }
    pub (crate) async fn get_subject(&self, subject_id: SubjectId) -> Result<Option<Subject>, RetrieveError> {
        let root_index = self.root_index(Duration::from_secs(5)).await?;
        root_index.get(self, subject_id.id).await
    }
    /// Retrieve a subject for a known MemoRefHead – ususally used for relationship traversal.
    /// Any relevant context will also be applied when reconstituting the relevant subject to ensure that our consistency model invariants are met
    pub async fn get_subject_with_head(&self,  mut head: MemoRefHead)  -> Result<Subject, RetrieveError> {

        if head.len() == 0 {
            return Err(RetrieveError::InvalidMemoRefHead);
        }

        if let Some(subject_id) = head.subject_id() {
            head.apply_mut(&self.stash.get_head(subject_id), &self.slab ).await?;
        }
        
        let subject = Subject::reconstitute(&self, head)?;
        return Ok(subject);

    }
    pub (crate) async fn get_subject_handle_with_head (&self, head: MemoRefHead)  -> Result<SubjectHandle, RetrieveError> {
        Ok(SubjectHandle{
            id: head.subject_id().ok_or( RetrieveError::InvalidMemoRefHead )?,
            subject: self.get_subject_with_head(head).await?,
            context: self.clone()
        })
    }
}