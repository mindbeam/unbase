
extern crate linked_hash_map;

use std::sync::{Arc,Mutex};
use linked_hash_map::LinkedHashMap;
use network::peer::PeerSpec;
use network::{Network,TxItem};
use memo::Memo;

/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

struct SlabInner{
    map: LinkedHashMap<u64, Memo>,
    last_memo_id: u32,
    net: Network
}

pub struct Slab {
    pub id: u32,
    inner: Arc<Mutex<SlabInner>>
}

pub type SlabOuter = Arc<Slab>;

impl Clone for Slab {
    fn clone(&self) -> Slab {
        Slab {
            id: self.id,
            inner: self.inner.clone()
        }
    }
}

impl Slab {
    pub fn new(net: &Network) -> SlabOuter {
        let slab_id = net.generate_slab_id();

        let inner = SlabInner {
            net: net.clone(),
            map: LinkedHashMap::new(),
            last_memo_id: 0
        };

        let me = Arc::new(Slab {
            id: slab_id,
            inner: Arc::new(Mutex::new(inner))
        });

        net.register_slab(&me);

        // TODO: Cloning the outer slab for the thread closure is super ugly
        //       There must be a better way to do this

        me.do_ping();
        me
    }
    pub fn gen_memo_id (&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        inner.last_memo_id += 1;

        (self.id as u64).rotate_left(32) | inner.last_memo_id as u64
    }
    pub fn put_memos(&self, memos : Vec<Memo>){
        let mut inner = self.inner.lock().unwrap();

        let mut tx_items = Vec::new();

        for memo in memos {
            // TODO: delete check_peering_target and replace with memo_durability_score
            let needs_peers = self.check_peering_target(&memo);
            if needs_peers > 0 {
                tx_items.push(TxItem {
                    memo:      memo.clone(),
                    peer_spec: PeerSpec::Any(needs_peers)
                });
            }

            inner.map.insert(memo.id,memo);
        }

        inner.net.transmit_memos( tx_items );
    }
    pub fn count_of_memos_resident( &self ) -> u32 {
        let inner = self.inner.lock().unwrap();
        inner.map.len() as u32
    }
    pub fn check_peering_target( &self, _memo: &Memo ) -> u8 {
        5
    }
    pub fn memo_durability_score( &self, _memo: &Memo ) -> u8 {
        // TODO: devise durability_score algo
        //       Should this number be inflated for memos we don't care about?
        //       Or should that be a separate signal?

        // Proposed factors:
        // Estimated number of copies in the network (my count = count of first order peers + their counts weighted by: uptime?)
        // Present diasporosity ( my diasporosity score = mean peer diasporosity scores weighted by what? )
        0
    }
    fn do_ping (&self){
        Memo::new(&self);
    }
}

impl Drop for Slab {
    fn drop(&mut self) {
        println!("> Dropping Slab {}", self.id);
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}
