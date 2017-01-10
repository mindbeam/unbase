//extern crate linked_hash_map;
//use linked_hash_map::LinkedHashMap;

use std::fmt;
use std::mem;
use std::sync::{Arc,Mutex,Weak};
use std::collections::HashMap;
use network::peer::{SlabRef};
use network::Network;
use memo::{Memo,MemoId,MemoRef};
use context::Context;

/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

struct SlabShared{
    pub id: u32,
    memos_by_id:    HashMap<MemoId, Memo>,
    memorefs_by_id: HashMap<MemoId,MemoRef>,
    record_subscriptions: HashMap<u64, Vec<Context>>,
    last_memo_id: u32,
    last_record_id: u32,
    _net: Network,
    peer_refs: Vec<SlabRef>
}

pub struct SlabInner {
    pub id: u32,
    shared: Mutex<SlabShared>
}

#[derive(Clone)]
pub struct Slab {
    pub id: u32,
    inner: Arc<SlabInner>
}

pub struct WeakSlab{
    pub id: u32,
    inner: Weak<SlabInner>
}

impl Slab {
    pub fn new(net: &Network) -> Slab {
        let slab_id = net.generate_slab_id();

        let shared = SlabShared {
            id: slab_id,
            _net: net.clone(),
            memos_by_id:          HashMap::new(),
            memorefs_by_id:       HashMap::new(),
            record_subscriptions: HashMap::new(),
            last_memo_id: 0,
            last_record_id: 0,
            peer_refs: Vec::new()
        };

        let me = Slab {
            id: slab_id,
            inner: Arc::new(SlabInner {
                id: slab_id,
                shared: Mutex::new(shared)
            })
        };

        net.register_slab(&me);

        // TODO: Cloning the outer slab for the thread closure is super ugly
        //       There must be a better way to do this

        //me.do_ping();
        me
    }
    pub fn weak (&self) -> WeakSlab {
        WeakSlab {
            id: self.id,
            inner: Arc::downgrade(&self.inner)
        }
    }
    pub fn generate_record_id(&self) -> u64 {
        let mut shared = self.inner.shared.lock().unwrap();
        shared.last_record_id += 1;

        (self.id as u64).rotate_left(32) | shared.last_record_id as u64
    }
    pub fn gen_memo_id (&self) -> u64 {
        let mut shared = self.inner.shared.lock().unwrap();
        shared.last_memo_id += 1;

        (self.id as u64).rotate_left(32) | shared.last_memo_id as u64
    }
    pub fn put_memos(&self, memos : Vec<Memo>){
        if memos.len() == 0 { return }

        let mut groups : HashMap<u64, Vec<Memo>> = HashMap::new();

        let mut shared = self.inner.shared.lock().unwrap();

        for memo in memos {
            shared.memos_by_id   .insert( memo.id, memo.clone() );
            shared.memorefs_by_id.insert( memo.id, memoref );
            // TODO: rewrite this to use sort / split
            let mut done = false;
            if let Some(g) = groups.get_mut(&memo.record_id) {
                g.push( memo.clone() );
                done = true;
            }

            // Ohhhhh merciful borrow checker
            if !done {
                groups.insert(memo.record_id, vec![memo.clone()]);
            }

        }

        for (record_id,memos) in groups {
            shared.dispatch_record_memos(record_id, &memos);
        }

        // LEFT OFF HERE - Next steps:
        //   hook up context subscriptions
        //   test each memo for durability_score and emit accordingly
    }
    pub fn count_of_memos_resident( &self ) -> u32 {
        let shared = self.inner.shared.lock().unwrap();
        shared.memos_by_id.len() as u32
    }
/*    fn do_ping (&self){
        Memo::new(&self);
    }
    */
    pub fn add_peer (&self, new_peer_ref: SlabRef) {
        let mut shared = self.inner.shared.lock().unwrap();
        shared.peer_refs.push(new_peer_ref);
    }
    pub fn peer_slab_count (&self) -> usize {
        let shared = self.inner.shared.lock().unwrap();
        shared.peer_refs.len()
    }
    pub fn deliver_all_memos (&self){
        let mut shared = self.inner.shared.lock().unwrap();

        for peer_ref in shared.peer_refs.iter_mut() {
            peer_ref.deliver_all_memos()
        }
    }
    pub fn create_context (&self) -> Context {
        Context::new(self)
    }
    pub fn subscribe_record (&self, record_id: u64, context: &Context) {
        let shared = self.inner.shared.lock().unwrap();
        if let Some(subs) = shared.record_subscriptions.get(&record_id) {
            subs.push(context.clone());
        }else{
            shared.record_subscriptions.insert(record_id, vec![context.clone()]);
        }
    }
    pub fn fetch_memo (&self, memo_id: MemoId, memoref: &mut MemoRef ) {

        //let memo : Memo;
        //mem::replace( memoref, MemoRef::Resident(memo) );

        unimplemented!()
    }
}

impl WeakSlab {
    pub fn upgrade (&self) -> Option<Slab> {
        match self.inner.upgrade() {
            Some(i) => Some( Slab { id: self.id, inner: i } ),
            None    => None
        }
    }
}

impl SlabShared {

    pub fn dispatch_record_memos (&mut self, record_id: u64, memos : &[Memo]){
        if let Some(subscribers) = self.record_subscriptions.get( &record_id ) {
            for sub in subscribers {
                sub.put_memos( memos )
            }
        }
    }
    pub fn emit_memos(&mut self, memos: Vec<&Memo>) {
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

impl Drop for SlabInner {
    fn drop(&mut self) {
        println!("> Dropping Slab {}", self.id);
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}

impl fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.inner.shared.lock().unwrap();

        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("peer_refs", &shared.peer_refs)
            .finish()
    }
}
