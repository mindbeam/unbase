pub mod serde;
use super::*;
use subject::*;
use memorefhead::MemoRefHead;
use error::RetrieveError;

use std::sync::{Arc,RwLock};
use std::fmt;


#[derive(Clone, Debug)]
pub struct MemoRef(pub Arc<MemoRefInner>);

impl Deref for MemoRef {
    type Target = MemoRefInner;
    fn deref(&self) -> &MemoRefInner {
        &*self.0
    }
}

#[derive(Debug)]
pub struct MemoRefInner {
    pub id:       MemoId,
    pub owning_slab_id: SlabId,
    pub subject_id: Option<SubjectId>,
    pub peerlist: RwLock<MemoPeerList>,
    pub ptr:      RwLock<MemoRefPtr>
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
        MemoRef(Arc::new(
            MemoRefInner {
                id: memo.id,
                owning_slab_id: slab.id,
                subject_id: memo.subject_id,
                peerlist: RwLock::new(MemoPeerList::new(Vec::with_capacity(3))),
                ptr: RwLock::new(MemoRefPtr::Resident( memo.clone() ))
            }
        ))
    }
}
impl MemoRef {
    pub fn to_head (&self) -> MemoRefHead {
        MemoRefHead::from_memoref(self.clone())
    }
    pub fn apply_peers (&self, _peers: &MemoPeerList ) -> bool {
        unimplemented!();
    }
    pub fn get_peerlist_for_peer (&self, my_ref: &SlabRef, dest_slabref: &SlabRef) -> MemoPeerList {
        let mut list : Vec<MemoPeer> = Vec::new();

        list.push(MemoPeer{
            slabref: my_ref.clone(),
            status: self.ptr.read().unwrap().to_peering_status()
        });

        // Tell the peer about all other presences except for ones belonging to them
        // we don't need to tell them they have it. They know, they were there :)

        for peer in self.peerlist.read().unwrap().iter().filter(|p| p.slabref.0.slab_id != dest_slabref.0.slab_id ) {
            list.push((*peer).clone());
        }

        MemoPeerList::new(list)

    }
    pub fn is_resident(&self) -> bool {
        match *self.ptr.read().unwrap() {
            MemoRefPtr::Resident(_) => true,
            _                       => false
        }
    }
    pub fn get_memo_if_resident(&self) -> Option<Memo> {
        match *self.ptr.read().unwrap() {
            MemoRefPtr::Resident(ref memo) => Some(memo.clone()),
            _ => None
        }
    }
    pub fn is_peered_with_slabref(&self, slabref: &SlabRef) -> bool {
        let status = self.peerlist.read().unwrap().iter().any(|peer| {
            (peer.slabref.0.slab_id == slabref.0.slab_id && peer.status != MemoPeeringStatus::NonParticipating)
        });

        status
    }
    pub fn get_memo (&self, slab: &Slab) -> Result<Memo,RetrieveError> {
        assert!(self.owning_slab_id == slab.id);

        // This seems pretty crude, but using channels for now in the interest of expediency
        let channel;
        {
            if let MemoRefPtr::Resident(ref memo) = *self.ptr.read().unwrap() {
                return Ok(memo.clone());
            }

            if slab.request_memo(self) > 0 {
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
            if slab.request_memo( &self ) == 0 {
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

        let mut found : bool = false;
        let ref mut list = self.peerlist.write().unwrap().0;
        for peer in list.iter_mut() {
            if peer.slabref.slab_id == slabref.slab_id {
                found = true;
                peer.status = status.clone();
                // TODO remove the peer entirely for MemoPeeringStatus::NonParticipating
                // TODO prune excess peers - Should keep this list O(10) peers
            }
        }

        if !found {
            list.push(MemoPeer{
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

impl Drop for MemoRefInner{
    fn drop(&mut self) {
        println!("# MemoRefInner({}).drop", self.id);
    }
}
