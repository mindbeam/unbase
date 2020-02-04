mod fixed;
pub use self::fixed::IndexFixed;
use crate::head::Head;

//#[cfg(test)]
// use crate::{
//    context::Context,
//    error::RetrieveError,
//    subjecthandle::SubjectHandle,
//};

trait Index {
    fn insert(&self, key: u64, head: Head);
    fn get(&self, key: u64) -> Option<Head>;
    //    #[cfg(test)]
    //    async fn test_get_subject_handle(&self, context: &Context, key: u64) -> Result<Option<SubjectHandle>,
    // RetrieveError>;
}
