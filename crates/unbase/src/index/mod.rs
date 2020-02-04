mod fixed;
pub use self::fixed::IndexFixed;
use crate::head::Head;

trait Index {
    fn insert(&self, key: u64, head: Head);
    fn get(&self, key: u64) -> Option<Head>;
}
