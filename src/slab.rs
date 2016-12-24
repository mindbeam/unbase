
extern crate linked_hash_map;

use std::fmt;
use std::sync::{Arc,Mutex};
use linked_hash_map::LinkedHashMap;
use network::peer::{SlabRef};
use network::Network;
use memo::Memo;

/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

struct SlabInner{
    pub id: u32,
    map: LinkedHashMap<u64, Memo>,
    last_memo_id: u32,
    net: Network,
    peer_refs: Vec<SlabRef>
}

pub struct Slab {
    pub id: u32,
    inner: Mutex<SlabInner>
}

pub type SlabOuter = Arc<Slab>;

/*impl Clone for Slab {
    fn clone(&self) -> Slab {
        Slab {
            id: self.id,
            inner: self.inner.clone()
        }
    }
}*/

impl Slab {
    pub fn new(net: &Network) -> SlabOuter {
        let slab_id = net.generate_slab_id();

        let inner = SlabInner {
            id: slab_id,
            net: net.clone(),
            map: LinkedHashMap::new(),
            last_memo_id: 0,
            peer_refs: Vec::new()
        };

        let me = Arc::new(Slab {
            id: slab_id,
            inner: Mutex::new(inner)
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

        for memo in memos.iter() {
            inner.map.insert( memo.id, memo.clone() );
        }

        inner.emit_memos( memos );
    }
    pub fn count_of_memos_resident( &self ) -> u32 {
        let inner = self.inner.lock().unwrap();
        inner.map.len() as u32
    }
    fn do_ping (&self){
        Memo::new(&self);
    }
    pub fn add_peer (&self, new_peer_ref: SlabRef) {
        let mut inner = self.inner.lock().unwrap();
        inner.peer_refs.push(new_peer_ref);
    }
    pub fn deliver_all_memos (&self){
        let mut inner = self.inner.lock().unwrap();

        for peer_ref in inner.peer_refs.iter_mut() {
            peer_ref.deliver_all_memos()
        }
    }
}

impl SlabInner {

    pub fn emit_memos(&mut self, memos: Vec<Memo>) {
        println!("Slab {} emit_memos {:?}", self.id, memos);

        // TODO - configurably auto-deliver these memos
        //        punting for now, because we want the test suite to monkey with delivery

        for memo in memos {
            let needs_peers = self.check_peering_target(&memo);
            for peer_ref in self.peer_refs.iter_mut().take( needs_peers as usize ) {
                peer_ref.tx_queue.push(memo.clone());
            }
        }

    }

    fn check_peering_target( &self, _memo: &Memo ) -> u8 {
        5
    }
/*    pub fn memo_durability_score( &self, _memo: &Memo ) -> u8 {
        // TODO: devise durability_score algo
        //       Should this number be inflated for memos we don't care about?
        //       Or should that be a separate signal?

        // Proposed factors:
        // Estimated number of copies in the network (my count = count of first order peers + their counts weighted by: uptime?)
        // Present diasporosity ( my diasporosity score = mean peer diasporosity scores weighted by what? )
        0
    }
*/
}

impl Drop for Slab {
    fn drop(&mut self) {
        println!("> Dropping Slab {}", self.id);
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}

impl fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.inner.lock().unwrap();

        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("peer_refs", &inner.peer_refs)
            .finish()
    }
}
