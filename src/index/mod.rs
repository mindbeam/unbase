use subject::Subject;

pub mod fixed;

trait Index{
    fn insert(&self, u64, Subject);
    fn get(&self, u64) -> Option<Subject>;
}
