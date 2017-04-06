
pub mod serde;
use super::memo::*;
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
    pub owning_slab_id: SlabId,
    pub subject_id: Option<SubjectId>,
    pub shared: Arc<Mutex<MemoRefShared>>
}

#[derive(Debug)]
pub struct MemoRefShared {
    pub id:       MemoId,
    pub peerlist: MemoPeerList,
    pub ptr:      MemoRefPtr
}
#[derive(Debug)]
pub enum MemoRefPtr {
    Resident(Memo),
    Remote
}

impl MemoRefPtr {
    pub fn to_peering_status (&self) -> MemoPeeringStatus {
        match self {
            &MemoRefPtr::Resident(_) => MemoPeeringStatus::Resident,
            &MemoRefPtr::Remote      => MemoPeeringStatus::Participating
        }
    }
}

impl MemoRef {
    pub fn from_memo (slab: &Slab, memo : &Memo) -> Self {
        MemoRef {
            id: memo.id,
            owning_slab_id: slab.id,
            subject_id: memo.subject_id,
            shared: Arc::new(Mutex::new(
                MemoRefShared {
                    id: memo.id,
                    peerlist: MemoPeerList(Vec::with_capacity(3)),
                    ptr: MemoRefPtr::Resident( memo.clone() )
                }
            ))
        }
    }
    pub fn inner (&self) -> MutexGuard<MemoRefShared> {
        self.shared.lock().unwrap()
    }
    pub fn to_head (&self) -> MemoRefHead {
        MemoRefHead::from_memoref(self.clone())
    }
    pub fn apply_peers (&self, peers: &MemoPeerList ) -> bool {
        unimplemented!();
    }
    pub fn get_peerlist_for_peer (&self, my_ref: &SlabRef, dest_slabref: &SlabRef) -> MemoPeerList {
        let shared = *(self.shared.lock().unwrap());
        let list : Vec<MemoPeer> = Vec::with_capacity(shared.peerlist.0.len() + 1);

        list.push(MemoPeer{
            slabref: my_ref.clone(),
            status: shared.ptr.to_peering_status()
        });

        // Tell the peer about all other presences except for ones belonging to them
        // we don't need to tell them they have it. They know, they were there :)

        for peer in shared.peerlist.0.iter().filter(|p| p.slabref.0.slab_id != dest_slabref.0.slab_id ) {
            list.push(*peer.clone());
        }

        MemoPeerList(list)

    }
    pub fn is_resident(&self) -> bool {
        let shared = self.shared.lock().unwrap();

        match shared.ptr {
            MemoRefPtr::Resident(_) => true,
            _                       => false
        }
    }
    pub fn get_memo_if_resident(&self) -> Option<Memo> {
        let shared = self.shared.lock().unwrap();

        match shared.ptr {
            MemoRefPtr::Resident(ref memo) => Some(memo.clone()),
            _ => None
        }
    }
    pub fn is_peered_with_slabref(&self, slabref: &SlabRef) -> bool {
        let shared = self.shared.lock().unwrap();

        let status = shared.peerlist.0.iter().any(|peer| {
            (peer.slabref.0.slab_id == slabref.0.slab_id && peer.status != MemoPeeringStatus::NonParticipating)
        });

        status
    }
    pub fn get_memo (&self, slab: &Slab) -> Result<Memo,RetrieveError> {
        assert!(self.owning_slab_id == slab.id);

    // *********************************************************
    // IMPORTANT TODO: avoid blocking with an active SlabInner.
    // *********************************************************


        // This seems pretty crude, but using channels for now in the interest of expediency
        let channel;
        {
            let shared = self.shared.lock().unwrap();
            if let MemoRefPtr::Resident(ref memo) = shared.ptr {
                return Ok(memo.clone());
            }

            if slab.inner().request_memo(self) > 0 {
                channel = slab.memo_wait_channel(self.id, "convert request_memo to return a wait channel?");
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
            if slab.inner().request_memo( &self ) == 0 {
                return Err(RetrieveError::NotFound)
            }

        }

        Err(RetrieveError::NotFoundByDeadline)

    }
    pub fn descends (&self, memoref: &MemoRef, slab: &Slab) -> bool {
        assert!(self.owning_slab_id == slab.id);
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
    pub fn update_peer (&self, slabref: &SlabRef, status: MemoPeeringStatus){

        let mut shared = self.shared.lock().unwrap();

        let mut found : bool = false;
        for peer in shared.peerlist.0.iter_mut() {
            if peer.slabref.0.slab_id == slabref.0.slab_id {
                found = true;
                peer.status = status.clone();
                // TODO remove the peer entirely for MemoPeeringStatus::NonParticipating
                // TODO prune excess peers - Should keep this list O(10) peers
            }
        }

        if !found {
            shared.peerlist.0.push(MemoPeer{
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
           .field("peerlist", &shared.peerlist)
           .field("ptr", &shared.ptr)
           .finish()
    }
}


impl MemoRefShared {

}
impl Drop for MemoRefShared{
    fn drop(&mut self) {
        println!("# MemoRefShared({}).drop", self.id);
    }
}
