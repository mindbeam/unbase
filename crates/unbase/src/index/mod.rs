use crate::subject::Subject;

mod fixed;
pub use self::fixed::IndexFixed;

trait Index{
    fn insert(&self, key: u64, subject: Subject);
    fn get(&self, key: u64) -> Option<Subject>;
}
