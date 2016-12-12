
extern crate linked_hash_map;

use std::sync::{Arc,Mutex};
use linked_hash_map::LinkedHashMap;
use std::thread;
use std::thread::JoinHandle;

use network::PeerSpec;
use network::Network;
use memo::Memo;

/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

struct SlabInner{
    map: LinkedHashMap<u64, Memo>,
    last_memo_id: u32,
    net: Network,
    rx_thread: Option<JoinHandle<()>>,
}

pub struct Slab {
    pub id: u32,
    inner: Arc<Mutex<SlabInner>>
}

impl Clone for Slab {
    fn clone(&self) -> Slab {
        Slab {
            id: self.id,
            inner: self.inner.clone()
        }
    }
}

impl Slab {
    pub fn new(net: &Network) -> Slab {

        let slab_id = net.generate_slab_id();

        let mut inner = SlabInner {
            net: net.clone(),
            map: LinkedHashMap::new(),
            rx_thread: None,
            last_memo_id: 0
        };

        let me = Slab {
            id: slab_id,
            inner: Arc::new(Mutex::new(inner))
        };

        let rx = net.register_slab(&me);

        // TODO: Cloning the outer slab for the thread closure is super ugly
        //       There must be a better way to do this

        let me_clone  = me.clone();
        inner.rx_thread = Some(thread::spawn(move || {
            for memo in rx.iter() {
                me_clone.put_memo(memo);
            }
        }));

        me.do_ping();
        me
    }
    pub fn gen_memo_id (&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        inner.last_memo_id += 1;

        (self.id as u64).rotate_left(32) | inner.last_memo_id as u64
    }
    pub fn put_memo(&self, memo : Memo){
        let mut inner = self.inner.lock().unwrap();

        let needs_peers = self.check_peering_target(&memo);
        if needs_peers > 0 {
            inner.net.transmit_memo( memo.clone(), PeerSpec::Any(needs_peers) );
        }

        inner.map.insert(memo.id,memo);
    }
    pub fn count_of_memos_received( &self ) -> u32 {
        let inner = self.inner.lock().unwrap();
        inner.map.len() as u32
    }
    pub fn check_peering_target( &self, _memo: &Memo ) -> u8 {
        5
    }
    fn do_ping (&self){
        Memo::new(&self);
    }
}
