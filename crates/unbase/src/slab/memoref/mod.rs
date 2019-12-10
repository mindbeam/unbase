use core::ops::Deref;

pub mod serde;
use super::*;
use crate::memorefhead::MemoRefHead;
use crate::error::RetrieveError;

use std::sync::{Arc,RwLock};
use std::fmt;
use futures::future::{select,Either};
use timer::Delay;


#[derive(Clone)]
pub struct MemoRef(pub Arc<MemoRefInner>);

impl Deref for MemoRef {
    type Target = MemoRefInner;
    fn deref(&self) -> &MemoRefInner {
        &*self.0
    }
}

pub struct MemoRefInner {
    pub id:       MemoId,
    pub owning_slab_id: SlabId,
    pub subject_id: Option<SubjectId>,
    pub peerlist: RwLock<MemoPeerList>,
    pub ptr:      RwLock<MemoRefPtr>
}

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
    pub fn to_head (&self) -> MemoRefHead {
        MemoRefHead::from_memoref(self.clone())
    }
    pub fn apply_peers ( &self, apply_peerlist: &MemoPeerList ) -> bool {

        let peerlist = &mut *self.peerlist.write().unwrap();
        let mut acted = false;
        for apply_peer in apply_peerlist.0.clone() {
            if apply_peer.slabref.slab_id == self.owning_slab_id {
                println!("WARNING - not allowed to apply self-peer");
                //panic!("memoref.apply_peers is not allowed to apply for self-peers");
                continue;
            }
            if peerlist.apply_peer(apply_peer) {
                acted = true;
            }
        }
        acted
    }
    pub fn get_peerlist_for_peer (&self, my_ref: &SlabRef, maybe_dest_slab_id: Option<SlabId>) -> MemoPeerList {
        //println!("MemoRef({}).get_peerlist_for_peer({:?},{:?})", self.id, my_ref, maybe_dest_slab_id);
        let mut list : Vec<MemoPeer> = Vec::new();

        list.push(MemoPeer{
            slabref: my_ref.clone(),
            status: self.ptr.read().unwrap().to_peering_status()
        });

        // Tell the peer about all other presences except for ones belonging to them
        // we don't need to tell them they have it. They know, they were there :)

        if let Some(dest_slab_id) = maybe_dest_slab_id {
            for peer in self.peerlist.read().unwrap().iter() {
                if peer.slabref.0.slab_id != dest_slab_id {
                    list.push((*peer).clone());
                }
            }
        }else{
            list.append(&mut self.peerlist.read().unwrap().0.clone());
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
    pub async fn get_memo (&self, slab: &SlabHandle) -> Result<Memo,RetrieveError> {
//        println!("Slab({}).MemoRef({}).get_memo()", self.owning_slab_id, self.id );
        assert!(self.owning_slab_id == slab.my_ref.slab_id,"requesting slab does not match owning slab");

        // This seems pretty crude, but using channels for now in the interest of expediency
        let channel;
        {
            if let MemoRefPtr::Resident(ref memo) = *self.ptr.read().unwrap() {
                return Ok(memo.clone());
            }
        }

        if slab.request_memo(self) > 0 {
            channel = slab.memo_wait_channel(self.id);
        }else{
            return Err(RetrieveError::NotFound)
        }

        // By sending the memo itself through the channel
        // we guarantee that there's no funny business with request / remotize timing


        use std::time;
        let duration = time::Duration::from_millis(1000);

        for _ in 0..3 {
            let timeout = Delay::new( duration );
            match select(channel, timeout).await {
                Either::Left((Ok(memo),_)) => {
                    return Ok(memo)
                },
                _ => {
                    // timed out or canceled
                }
            }

            // have another go around
            if slab.request_memo( &self ) == 0 {
                return Err(RetrieveError::NotFound)
            }

        }

        Err(RetrieveError::NotFoundByDeadline)

    }
    pub async fn descends (&self, memoref: &MemoRef, slab: &SlabHandle) -> bool {
        assert!(self.owning_slab_id == slab.my_ref.slab_id);
        match self.get_memo( slab ).await {
            Ok(my_memo) => {
                if my_memo.descends(&memoref, slab).await {
                    return true
                }
            }
            Err(_) => {
                // TODO: convert this into a Result<>
                panic!("Unable to retrieve memo")
            }
        };

        false
    }
    pub fn update_peer (&self, slabref: &SlabRef, status: MemoPeeringStatus) -> bool {

        let mut acted = false;
        let mut found = false;
        let ref mut list = self.peerlist.write().unwrap().0;
        for peer in list.iter_mut() {
            if peer.slabref.slab_id == self.owning_slab_id {
                println!("WARNING - not allowed to apply self-peer");
                //panic!("memoref.update_peers is not allowed to apply for self-peers");
                continue;
            }
            if peer.slabref.slab_id == slabref.slab_id {
                found = true;
                if peer.status != status {
                    acted = true;
                    peer.status = status.clone();
                }
                // TODO remove the peer entirely for MemoPeeringStatus::NonParticipating
                // TODO prune excess peers - Should keep this list O(10) peers
            }
        }

        if !found {
            acted = true;
            list.push(MemoPeer{
                slabref: slabref.clone(),
                status: status.clone()
            })
        }

        acted
    }
}

impl fmt::Debug for MemoRef{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("MemoRef")
           .field("id", &self.id)
           .field("owning_slab_id", &self.owning_slab_id)
           .field("subject_id", &self.subject_id)
           .field("peerlist", &*self.peerlist.read().unwrap())

           .field("resident", &match *self.ptr.read().unwrap() {
               MemoRefPtr::Remote      => false,
               MemoRefPtr::Resident(_) => true
           })
           .finish()
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
        //println!("# MemoRefInner({}).drop", self.id);
    }
}
