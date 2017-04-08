mod common_structs;
mod inner;
pub mod memo;
pub mod slabref;
pub mod memoref;
mod memohandling;

pub use self::common_structs::*;
pub use self::slabref::{SlabRef,SlabRefInner};
pub use self::memoref::{MemoRef,MemoRefInner,MemoRefPtr};
pub use self::memo::{Memo,MemoBody};
pub use self::inner::SlabInner;
use self::inner::*;

use std::fmt;

use network::{Network,Transmitter};
use subject::SubjectId;
use memorefhead::*;
use context::{Context,WeakContext};


use std::ops::{Deref, DerefMut};
use std::sync::{Arc,Weak,RwLock};
use network::TransportAddress;

use std::collections::HashMap;

/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

pub type SlabId = u32;

#[derive(Clone)]
pub struct Slab(Arc<SlabInner>);

impl Deref for Slab {
    type Target = SlabInner;
    fn deref(&self) -> &SlabInner {
        &*self.0
    }
}

#[derive(Clone)]
pub struct WeakSlab{
    pub id: u32,
    inner: Weak<SlabInner>
}


struct SlabInner{
    pub id: SlabId,
    memorefs_by_id: RwLock<HashMap<MemoId,MemoRef>>,
    memo_wait_channels: Mutex<HashMap<MemoId,Vec<mpsc::Sender<Memo>>>>, // TODO: HERE HERE HERE - convert to per thread wait channel senders?
    subject_subscriptions: RwLock<HashMap<SubjectId, Vec<WeakContext>>>,

    counters: RwLock<SlabCounters>,

    pub my_ref: SlabRef,
    peer_refs: RwLock<Vec<SlabRef>>,
    net: Network
}
struct SlabCounters{
    last_memo_id: u32,
    last_subject_id: u32,
    memos_received: u64,
    memos_redundantly_received: u64,
}

impl Slab {
    pub fn new(net: &Network) -> Slab {
        let slab_id = net.generate_slab_id();

        let my_ref_inner = SlabRefInner {
            slab_id: slab_id,
            owning_slab_id: slab_id, // I own my own ref to me, obviously
            presence: RwLock::new(vec![]), // this bit is just for show
            tx: RwLock::new(Transmitter::new_blackhole()),
            return_address: TransportAddress::Local,
        };

        let my_ref = SlabRef(Arc::new(my_ref_inner));

        let inner = SlabInner {
            id: slab_id,
            memorefs_by_id:        RwLock::new(HashMap::new()),
            memo_wait_channels:    RwLock::new(HashMap::new()),
            subject_subscriptions: RwLock::new(HashMap::new()),

            counters: RwLock::new(SlabCounters {
                last_memo_id: 5000,
                last_subject_id: 0,
                memos_received: 0,
                memos_redundantly_received: 0,
            }),

            my_ref: my_ref,
            peer_refs: RwLock::new(Vec::new()),
            net: net.clone()
        };

        let me = Slab(inner);

        net.register_local_slab(&me);
        net.conditionally_generate_root_index_seed(&me);

        me
    }
    pub fn weak (&self) -> WeakSlab {
        WeakSlab {
            id: self.id,
            inner: Arc::downgrade(&self.inner)
        }
    }
}

impl WeakSlab {
    pub fn upgrade (&self) -> Option<Slab> {
        match self.0.upgrade() {
            Some(i) => Some( Slab { id: self.id, inner: i } ),
            None    => None
        }
    }
}

impl fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.inner.shared.lock().unwrap();

        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("peer_refs", &shared.peer_refs)
            .field("memo_refs", &shared.memorefs_by_id)
            .finish()
    }
}
