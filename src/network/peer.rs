use std::fmt;
use slab::Slab;
use memo::Memo;
use std::mem;


pub struct SlabRef {
    // TODO - update Slabref to reference either network addresses OR resident slabs
    //       attempt to avoid address lookups for resident slabs to minimize instructions
    pub slab_id: u32,
    slab: Slab,
    pub tx_queue: Vec<Memo>
}

impl SlabRef{
    pub fn new (slab: &Slab) -> SlabRef {
        SlabRef {
            slab_id: slab.id,
            slab:    slab.clone(),
            tx_queue: Vec::new()
        }
    }
    pub fn deliver_all_memos (&mut self){
        let mut tx_queue : Vec<Memo> = Vec::new();
        mem::swap(&mut tx_queue, &mut self.tx_queue);

        self.slab.put_memos(tx_queue);

    }
}

impl Clone for SlabRef {
    fn clone(&self) -> SlabRef {
        SlabRef {
            slab_id: self.slab_id,
            slab:    self.slab.clone(),
            tx_queue: Vec::new() // we don't want to clone the old tx_queue
        }
    }
}

impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("slab_id", &self.slab_id)
            .field("tx_queue", &self.tx_queue)
            .finish()
    }
}

/*
pub enum PeerSpec {
    Any(u8),
    List(Vec<SlabRef>),
}

impl fmt::Debug for PeerSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let mut dbg = fmt.debug_struct("PeerSpec");

        match self {
            &PeerSpec::Any(c) => {
                dbg.field("Any", &c);
            }
            &PeerSpec::List(ref v) => {
                for p in v {
                    dbg.field("Peer", &p);
                }
            }
        };

        dbg.finish()
    }
}
*/
