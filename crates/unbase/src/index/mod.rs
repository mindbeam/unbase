mod fixed;
pub use self::fixed::IndexFixed;
use crate::{
    memorefhead::MemoRefHead
};

trait Index{
    fn insert(&self, key: u64, head: MemoRefHead);
    fn get(&self, key: u64) -> Option<MemoRefHead>;
}
