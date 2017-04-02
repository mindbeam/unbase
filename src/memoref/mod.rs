
pub mod serde;
use memo::*;
use slab::*;
use network::*;
use subject::*;
use memorefhead::MemoRefHead;
use std::sync::{Arc,Mutex};
use std::fmt;
use error::RetrieveError;


#[derive(Clone)]
pub struct MemoRef {
    pub id:    MemoId,
    pub subject_id: Option<SubjectId>,
    shared: Arc<Mutex<MemoRefShared>>
}
#[derive(Debug)]
pub struct MemoPeer {
    slabref: SlabRef,
    status: PeeringStatus
}

#[derive(Debug)]
struct MemoPeerList (Vec<MemoPeer>);

#[derive(Debug)]
struct MemoRefShared {
    pub id:    MemoId,
    pub peers: MemoPeerList,
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
                    id: memo.id,
                    peers: MemoPeerList(Vec::with_capacity(3)),
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
                    id: memo_id,
                    peers: MemoPeerList(Vec::with_capacity(3)),
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

        let status = shared.peers.0.iter().any(|peer| {
            (peer.slabref.slab_id == slabref.slab_id && peer.status != PeeringStatus::NonParticipating)
        });

        status
    }
    pub fn get_memo (&self, slab: &Slab) -> Result<Memo,RetrieveError> {
        // This seems pretty crude, but using channels for now in the interest of expediency
        let channel;
        {
            let shared = self.shared.lock().unwrap();
            if let MemoRefPtr::Resident(ref memo) = shared.ptr {
                return Ok(memo.clone());
            }

            if shared.send_memo_requests( &self.id, &slab ) > 0 {
                channel = slab.memo_wait_channel(self.id);
            }else{
                return Err(RetrieveError::NotFound)
            }
        }


        // By sending the memo itself through the channel
        // we guarantee that there's no funny business with request / remotize timing


        use std::time;
        let timeout = time::Duration::from_millis(2000);

        for _ in 0..3 {

            match channel.recv_timeout(timeout) {
                Ok(memo)       =>{
                    return Ok(memo)
                }
                Err(rcv_error) => {
                    use std::sync::mpsc::RecvTimeoutError::*;
                    match rcv_error {
                        Timeout => {}
                        Disconnected => {
                            return Err(RetrieveError::SlabError)
                        }
                    }
                }
            }

            // have another go around
            if self.shared.lock().unwrap().send_memo_requests( &self.id, &slab ) == 0 {
                return Err(RetrieveError::NotFound)
            }

        }

        Err(RetrieveError::NotFoundByDeadline)

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
    pub fn residentize(&self, slab: &Slab, memo: &Memo) -> bool {
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
                MemoBody::Peering(self.id, slabref.presence.clone(), PeeringStatus::Resident)
            );

            for peer in shared.peers.0.iter() {
                peer.slabref.send_memo( &slabref, peering_memo.clone() );
            }

            // residentized
            true
        }else{
            // already resident
            false
        }
    }
    pub fn remotize(&self, slab: &Slab ) {
        println!("# MemoRef({}).remotize()", self.id);
        let mut shared = self.shared.lock().unwrap();

        if let MemoRefPtr::Resident(_) = shared.ptr {
            if shared.peers.0.len() == 0 {
                panic!("Attempt to remotize a non-peered memo")
            }

            let slabref = slab.get_ref();

            let peering_memo = Memo::new_basic(
                slab.gen_memo_id(),
                0,
                MemoRefHead::from_memoref(self.clone()),
                MemoBody::Peering(self.id, slabref.presence.clone() ,PeeringStatus::Participating)
            );

            for peer in shared.peers.0.iter() {
                peer.slabref.send_memo( &slabref, peering_memo.clone() );
            }
        }

        shared.ptr = MemoRefPtr::Remote;
    }
    pub fn update_peer (&self, slabref: &SlabRef, status: PeeringStatus){

        let mut shared = self.shared.lock().unwrap();

        let mut found : bool = false;
        for peer in shared.peers.0.iter_mut() {
            if peer.slabref.slab_id == slabref.slab_id {
                found = true;
                peer.status = status.clone();
                // TODO remove the peer entirely for PeeringStatus::NonParticipating
                // TODO prune excess peers - Should keep this list O(10) peers
            }
        }

        if !found {
            shared.peers.0.push(MemoPeer{
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


impl MemoRefShared {
    fn send_memo_requests (&self, my_memo_id: &MemoId, slab: &Slab) -> u8 {
        let slabref = slab.get_ref();
        let request_memo = Memo::new_basic(
            slab.gen_memo_id(),
            0,
            MemoRefHead::new(), // TODO: how should this be parented?
            MemoBody::MemoRequest(vec![my_memo_id.clone()],slabref.clone())
        );

        let mut sent = 0u8;
        for peer in self.peers.0.iter().take(5) {
            peer.slabref.send_memo( &slabref, request_memo.clone() );
            sent += 1;
        }

        sent
    }
}
impl Drop for MemoRefShared{
    fn drop(&mut self) {
        println!("# MemoRefShared({}).drop", self.id);
    }
}
