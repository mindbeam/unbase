use memo::*;
use slab::*;
use network::slabref::*;
use std::sync::{Arc,Mutex};
use std::fmt;

#[derive(Clone)]
pub struct MemoRef {
    pub id:    MemoId,
    shared: Arc<Mutex<MemoRefShared>>
}
#[derive(Debug)]
struct MemoRefShared {
    peers: Vec<SlabRef>,
    ptr:   MemoRefPtr
}
#[derive(Debug)]
pub enum MemoRefPtr {
    Resident(Memo),
    Remote
}

impl MemoRef {
    pub fn new_from_memo (memo : &Memo) -> MemoRef {
        MemoRef {
            id: memo.id,
            shared: Arc::new(Mutex::new(
                MemoRefShared {
                    peers: Vec::new(),
                    ptr: MemoRefPtr::Resident( memo.clone() )
                }
            ))
        }
    }
    /* pub fn id (&self) -> MemoId {
        match self {
            MemoRef::Resident(memo) => memo.id,
            MemoRef::Remote(id)     => id
        }
    }*/
    pub fn get_memo (&mut self, slab: &Slab) -> Result<Memo, String> {
        {
            let shared = &self.shared.lock().unwrap();
            if let MemoRefPtr::Resident(ref memo) = shared.ptr {
                return Ok(memo.clone());
            }
        }

        slab.localize_memo(self)
    }

}

impl fmt::Debug for MemoRef{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = &self.shared.lock().unwrap();
        fmt.debug_struct("MemoRef")
           .field("memo_id", &self.id)
           .field("peers", &shared.peers)
           .field("ptr", &shared.ptr)
           .finish()
    }
}
