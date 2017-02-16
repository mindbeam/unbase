use memo::*;
use slab::*;
use network::*;
use std::sync::{Arc,Mutex};
use std::fmt;

#[derive(Clone)]
pub struct MemoRef {
    pub id:    MemoId,
    shared: Arc<Mutex<MemoRefShared>>
}
#[derive(Debug)]
struct MemoPeer {
    slabref: SlabRef,
    status: PeeringStatus
}
#[derive(Debug)]
struct MemoRefShared {
    peers: Vec<MemoPeer>,
    ptr:   MemoRefPtr
}
#[derive(Debug)]
pub enum MemoRefPtr {
    Resident(Memo),
    Remote
}

impl MemoRef {
    pub fn new_from_memo (memo : &Memo) -> Self {
        MemoRef {
            id: memo.id,
            shared: Arc::new(Mutex::new(
                MemoRefShared {
                    peers: Vec::with_capacity(3),
                    ptr: MemoRefPtr::Resident( memo.clone() )
                }
            ))
        }
    }
    pub fn new_remote (memo_id: MemoId) -> Self {
        MemoRef {
            id: memo_id,
            shared: Arc::new(Mutex::new(
                MemoRefShared {
                    peers: Vec::with_capacity(3),
                    ptr: MemoRefPtr::Remote
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

    pub fn get_memo_if_resident(&self) -> Option<Memo> {
        let shared = self.shared.lock().unwrap();

        match shared.ptr {
            MemoRefPtr::Resident(ref memo) => Some(memo.clone()),
            _ => None
        }
    }
    pub fn is_peered_with_slabref(&self, slabref: &SlabRef) -> bool {
        let shared = self.shared.lock().unwrap();

        shared.peers.iter().any(|peer| {
            (peer.slabref.slab_id == slabref.slab_id) && peer.status != PeeringStatus::NonParticipating
        })
    }
    pub fn get_memo (&mut self, slab: &Slab) -> Result<Memo, String> {
        {
            let shared = self.shared.lock().unwrap();
            if let MemoRefPtr::Resident(ref memo) = shared.ptr {
                return Ok(memo.clone());
            }
        }

        slab.localize_memo(self)
    }
    pub fn descends (&mut self, memoref: &MemoRef, slab: &Slab) -> bool {
        match self.get_memo( slab ) {
            Ok(my_memo) => {
                if my_memo.descends(&memoref, slab) { return true }
            }
            Err(_) => {
                panic!("Unable to retrieve my memo")
            }
        };

        false
    }
    pub fn residentize(&self, slabref: &SlabRef, memo: &Memo) {
        let mut shared = self.shared.lock().unwrap();

        if self.id != memo.id {
            panic!("Attempt to residentize mismatching memo");
        }

        if let MemoRefPtr::Remote = shared.ptr {
            shared.ptr = MemoRefPtr::Resident( memo.clone() );

            let peering_memo = Memo::new_basic(
                memo.id, 0,
                vec![self.clone()],
                MemoBody::Peering(self.id,slabref.clone(),PeeringStatus::Resident)
            );

            for peer in shared.peers.iter() {
                peer.slabref.send_memo( slabref, peering_memo.clone() );
            }
        }
    }
    pub fn update_peer (&self, slabref: &SlabRef, status: &PeeringStatus){

        let mut shared = self.shared.lock().unwrap();

        let mut found : bool = false;
        for peer in shared.peers.iter_mut() {
            if peer.slabref.slab_id == slabref.slab_id {
                found = true;
                peer.status = status.clone();
                // TODO remove the peer entirely for PeeringStatus::NonParticipating
                // TODO prune excess peers - Should keep this list O(10) peers
            }
        }

        if !found {
            shared.peers.push(MemoPeer{
                slabref: slabref.clone(),
                status: status.clone()
            })
        }
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
