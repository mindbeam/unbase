
mod serde;
use memo::*;
use slab::*;
use network::*;
use subject::*;
use memorefhead::MemoRefHead;
use std::sync::{Arc,Mutex};
use std::fmt;
use std::error::Error;
use serde::ser::*;


#[derive(Clone)]
pub struct MemoRef {
    pub id:    MemoId,
    pub subject_id: Option<SubjectId>,
    pub shared: Arc<Mutex<MemoRefShared>>
}
#[derive(Debug, Serialize, Deserialize)]
struct MemoPeer {
    slabref: SlabRef,
    status: PeeringStatus
}
#[derive(Debug)]
struct MemoRefShared {
    pub peers: Vec<MemoPeer>,
    pub ptr:   MemoRefPtr
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
            subject_id: Some(memo.subject_id),
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
            subject_id: None,
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
    pub fn get_memo (&self, slab: &Slab) -> Result<Memo, String> {
        // This seems pretty crude, but using channels for now in the interest of expediency
        let channel;

        {
            let shared = self.shared.lock().unwrap();
            if let MemoRefPtr::Resident(ref memo) = shared.ptr {
                return Ok(memo.clone());
            }

            let slabref = slab.get_ref();
            let request_memo = Memo::new_basic(
                slab.gen_memo_id(),
                0,
                MemoRefHead::new(), // TODO: how should this be parented?
                MemoBody::MemoRequest(vec![self.id],slabref.clone())
            );

            channel = slab.memo_wait_channel(self.id);

            for peer in shared.peers.iter().take(5) {
                peer.slabref.send_memo( &slabref, request_memo.clone() );
            }
        }

        // By sending the memo itself through the channel
        // we guarantee that there's no funny business with request / remotize timing
        match channel.recv() {
            Ok(memo)       => Ok(memo),
            Err(rcv_error) => Err(rcv_error.description().to_string()) // HACK
        }

    }
    pub fn descends (&self, memoref: &MemoRef, slab: &Slab) -> bool {
        match self.get_memo( slab ) {
            Ok(my_memo) => {
                if my_memo.descends(&memoref, slab) {
                    return true }
            }
            Err(_) => {
                panic!("Unable to retrieve my memo")
            }
        };

        false
    }
    pub fn residentize(&self, slab: &Slab, memo: &Memo) {
        println!("# MemoRef({}).residentize()", self.id);

        let mut shared = self.shared.lock().unwrap();

        if self.id != memo.id {
            panic!("Attempt to residentize mismatching memo");
        }

        if let MemoRefPtr::Remote = shared.ptr {
            shared.ptr = MemoRefPtr::Resident( memo.clone() );

            let slabref = slab.get_ref();

            let peering_memo = Memo::new_basic(
                slab.gen_memo_id(),
                0,
                MemoRefHead::from_memoref(self.clone()),
                MemoBody::Peering(self.id,slabref.clone(),PeeringStatus::Resident)
            );

            for peer in shared.peers.iter() {
                peer.slabref.send_memo( &slabref, peering_memo.clone() );
            }

        }
    }
    pub fn remotize(&self, slab: &Slab ) {
        println!("# MemoRef({}).remotize()", self.id);
        let mut shared = self.shared.lock().unwrap();

        if let MemoRefPtr::Resident(_) = shared.ptr {
            if shared.peers.len() == 0 {
                panic!("Attempt to remotize a non-peered memo")
            }

            let slabref = slab.get_ref();

            let peering_memo = Memo::new_basic(
                slab.gen_memo_id(),
                0,
                MemoRefHead::from_memoref(self.clone()),
                MemoBody::Peering(self.id,slabref.clone(),PeeringStatus::Participating)
            );

            for peer in shared.peers.iter() {
                peer.slabref.send_memo( &slabref, peering_memo.clone() );
            }
        }

        shared.ptr = MemoRefPtr::Remote;
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

impl PartialEq for MemoRef {
    fn eq(&self, other: &MemoRef) -> bool {
        // TODO: handle the comparision of pre-hashed memos as well as hashed memos
        self.id == other.id
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
