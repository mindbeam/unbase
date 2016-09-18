
use std::sync::Mutex;
use network::Network;
use linked_hash_map::LinkedHashMap;
use memo::Memo;

/* Initial plan:
 * Use Mutex internal struct to manage slab storage
 * Later, refactor to allow concurrent memo insertions
 */

struct SlabInternals{
    map: LinkedHashMap<u64, Memo>,
    last_memo_id: u32
}
pub struct Slab {
    pub id: u32,
    internals: Mutex<SlabInternals>
}

impl Slab {
    pub fn new(net : &Network) -> Slab {

        let internals = SlabInternals {
            map: LinkedHashMap::new(),
            last_memo_id: 0
        };

        let me = Slab {
            id: net.generate_slab_id(),
            internals: Mutex::new(internals)
        };

        me.do_ping();
        me
    }
    pub fn gen_memo_id (&self) -> u64 {
        let mut internals = self.internals.lock().unwrap();
        internals.last_memo_id += 1;

        (self.id as u64).rotate_left(32) | internals.last_memo_id as u64
    }
    pub fn put_memo(&self, memo : Memo){
        let mut internals = self.internals.lock().unwrap();
        internals.map.insert(memo.id,memo);
    }
    pub fn memos_received( &self ) -> u32 {
        let internals = self.internals.lock().unwrap();
        internals.map.len() as u32
    }
    fn do_ping (&self){
        Memo::new(&self);
    }
}
