use std::fmt;
use slab::{Slab,SlabSender};
use memo::Memo;
use std::mem;
use std::sync::{Arc,Mutex};
use std::sync::mpsc;

#[derive(Clone)]
pub struct SlabRef {
    // TODO - update Slabref to reference either network addresses OR resident slabs
    //       attempt to avoid address lookups for resident slabs to minimize instructions
    inner: Arc<SlabRefInner>
}
struct SlabRefInner {
    slab_id: u32,
    //slab: Slab,
    sender: SlabSender
}

impl SlabRef{
    pub fn new (slab: &Slab) -> SlabRef {
        SlabRef {
            inner: Arc::new (SlabRefInner {
                slab_id: slab.id,
                sender:  slab.get_sender()
            })
        }
    }

    pub fn send_memo (&mut self, memo: &Memo) {
        self.inner.sender.send(memo);
    }
    pub fn deliver_all_memos (&mut self){
        /*let mut tx_queue : Vec<Memo> = Vec::new();
        mem::swap(&mut tx_queue, &mut self.tx_queue);

        self.slab.put_memos(&tx_queue);
*/
    }
}

impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("slab_id", &self.inner.slab_id)
            .finish()
    }
}
