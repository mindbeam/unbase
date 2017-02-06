use std::fmt;
use std::sync::{Arc,Mutex,Weak};
use std::collections::HashMap;
use network::slabref::{SlabRef};
use network::channel::Sender;


use network::Network;
use memo::{Memo,MemoId};
use memoref::MemoRef;
use context::{Context,WeakContext};


/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

pub type SlabId = u32;

#[derive(Clone)]
pub struct Slab {
    pub id: SlabId,
    inner: Arc<SlabInner>
}

struct SlabShared{
    pub id: SlabId,
    memorefs_by_id: HashMap<MemoId,MemoRef>,
    subject_subscriptions: HashMap<u64, Vec<WeakContext>>,
    last_memo_id: u32,
    last_subject_id: u32,
    peer_refs: Vec<SlabRef>
}

pub struct SlabInner {
    pub id: SlabId,
    shared: Mutex<SlabShared>
}

#[derive(Clone)]
pub struct WeakSlab{
    pub id: u32,
    inner: Weak<SlabInner>
}

impl Slab {
    pub fn new(net: &Network) -> Slab {
        let slab_id = net.generate_slab_id();

        let shared = SlabShared {
            id: slab_id,
            memorefs_by_id:        HashMap::new(),
            subject_subscriptions: HashMap::new(),
            last_memo_id: 0,
            last_subject_id: 0,
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

        me
    }
    pub fn weak (&self) -> WeakSlab {
        WeakSlab {
            id: self.id,
            inner: Arc::downgrade(&self.inner)
        }
    }
    pub fn generate_subject_id(&self) -> u64 {
        let mut shared = self.inner.shared.lock().unwrap();
        shared.last_subject_id += 1;

        (self.id as u64).rotate_left(32) | shared.last_subject_id as u64
    }
    pub fn gen_memo_id (&self) -> u64 {
        let mut shared = self.inner.shared.lock().unwrap();
        shared.last_memo_id += 1;

        (self.id as u64).rotate_left(32) | shared.last_memo_id as u64
    }
    pub fn put_memos(&self, memos : Vec<Memo>){
        if memos.len() == 0 { return }
        let mut shared = self.inner.shared.lock().unwrap();

        shared.put_memos(memos);
    }
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        let shared = self.inner.shared.lock().unwrap();
        shared.memorefs_by_id.len() as u32
    }
    pub fn add_peer (&self, new_peer_ref: SlabRef) {
        let mut shared = self.inner.shared.lock().unwrap();
        shared.peer_refs.push(new_peer_ref);
    }
    pub fn peer_slab_count (&self) -> usize {
        let shared = self.inner.shared.lock().unwrap();
        shared.peer_refs.len()
    }
    pub fn create_context (&self) -> Context {
        Context::new(self)
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        let weakcontext : WeakContext = context.weak();

        let mut shared = self.inner.shared.lock().unwrap();

        if let Some(subs) = shared.subject_subscriptions.get_mut(&subject_id) {
            subs.push(weakcontext);
            return;
        }

        // Stoopid borrow checker
        shared.subject_subscriptions.insert(subject_id, vec![weakcontext]);
        return;
    }
    pub fn unsubscribe_subject (&self,  subject_id: u64, context: &Context ){

        let mut shared = self.inner.shared.lock().unwrap();

        if let Some(subs) = shared.subject_subscriptions.get_mut(&subject_id) {

            subs.retain(|c| {
                c.cmp(&context.weak()) // seems evil here
            });
            return;
        }
    }
    pub fn localize_memo (&self, _memoref: &mut MemoRef ) -> Result<Memo, String> {

        //let memo : Memo;
        //mem::replace( memoref, MemoRef::Resident(memo) );
        //memoref.set_memo();

        Err("unable to localize memo".to_owned())
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

    pub fn dispatch_subject_memorefs (&mut self, subject_id: u64, memorefs : &[MemoRef]){
        if let Some(subscribers) = self.subject_subscriptions.get( &subject_id ) {
            for weakcontext in subscribers {
                if let Some(context) = weakcontext.upgrade() {
                    context.put_subject_memos( subject_id, memorefs );
                }

            }
        /*        let maybecontext : WeakContext = cw;
                if let Some(context) = weakcontext.upgrade() {
                    context.put_subject_memos( subject_id, memorefs )
                }
            } */
        }
    }
    pub fn put_memos (&mut self, memos: Vec<Memo>){
        // TODO: Evaluate more efficient ways to group these memos by subject
        let mut groups : HashMap<u64, Vec<MemoRef>> = HashMap::new();

        // LEFT OFF HERE - Next steps:
        // test each memo for durability_score and emit accordingly
        self.emit_memos(&memos);

        for memo in memos {
            // TODO: NoOp here if the memo is already resident
            let memoref = MemoRef::new_from_memo(&memo);

            // TODO: rewrite this to use sort / split
            let mut done = false;
            if let Some(g) = groups.get_mut(&memo.subject_id) {
                g.push( memoref.clone() );
                done = true;
            }
            // Ohhhhh merciful borrow checker
            if !done {
                groups.insert(memo.subject_id, vec![memoref.clone()]);
            }

            self.memorefs_by_id.insert( memo.id, memoref );
        }

        for (subject_id,memorefs) in groups {
            self.dispatch_subject_memorefs(subject_id, &memorefs);
        }
    }
    pub fn emit_memos(&mut self, memos: &Vec<Memo>) {
        println!("Slab {} emit_memos {:?}", self.id, memos);

        // TODO - configurably auto-deliver these memos
        //        punting for now, because we want the test suite to monkey with delivery

        for memo in memos {
            let needs_peers = self.check_peering_target(&memo);
            for peer_ref in self.peer_refs.iter_mut().take( needs_peers as usize ) {
                peer_ref.send_memo( &memo );
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
            .field("memo_refs", &shared.memorefs_by_id)
            .finish()
    }
}
