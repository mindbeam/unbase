use std::collections::hash_map::Entry;

pub use self::common_structs::*;
pub use self::slabref::{SlabRef,SlabRefInner};
pub use self::memoref::{MemoRef,MemoRefInner,MemoRefPtr};
pub use self::memo::{MemoId,Memo,MemoInner,MemoBody};
pub use self::memoref::serde as memoref_serde;
pub use self::memo::serde as memo_serde;


use crate::subject::SubjectId;
use crate::memorefhead::*;
use crate::context::{Context,WeakContext};
use crate::network::{Network,Transmitter,TransportAddress};

use std::sync::{Arc,RwLock,Mutex};
use futures::channel::mpsc;
use std::fmt;

mod state;
mod agent;
mod common_structs;
mod handle;

mod memo;
mod slabref;
mod memoref;

pub use handle::SlabHandle;

pub type SlabId = u32;

use crate::slab::agent::SlabAgent;
use futures::{StreamExt, Future};
use futures::future::RemoteHandle;

type DispatcherFuture = impl Future<Output = u32> + Send;
type Dispatcher = impl Fn() -> DispatcherFuture;

#[derive(Clone)]
pub struct Slab{
    pub id: SlabId,
    agent: Arc<SlabAgent>,
    net: Network,
    pub my_ref: SlabRef,
    dispatch_channel: mpsc::Sender<MemoRef>,
    dispatcher: Arc<RemoteHandle<Dispatcher>>,
}

impl Slab {
    pub fn new(net: &Network) -> Slab {
        let id = net.generate_slab_id();

        let my_ref_inner = SlabRefInner {
            slab_id: id,
            owning_slab_id: id, // I own my own ref to me, obviously
            presence: RwLock::new(vec![]), // this bit is just for show
            tx: Mutex::new(Transmitter::new_blackhole(id)),
            return_address: RwLock::new(TransportAddress::Local),
        };

        let my_ref = SlabRef(Arc::new(my_ref_inner));
        // TODO: figure out how to reconcile this with the simulator

        let (dispatch_tx_channel, dispatch_rx_channel) = mpsc::channel::<MemoRef>(10);

        let agent = Arc::new(SlabAgent::new( net, my_ref.clone() ));

        let agent2 = agent.clone();
        let dispatcher  = crate::util::task::spawn_with_handle( async move || {
            let mut dispatch_rx_channel = dispatch_rx_channel;
            while let Some(memoref) = dispatch_rx_channel.next().await {
                agent2.recv_memoref(memoref);
            }
        });

        let me = Slab{
            id,
            dispatch_channel: dispatch_tx_channel,
            dispatcher: Arc::new(dispatcher),
            net: net.clone(),
            my_ref: my_ref,
            agent
        };

        net.register_local_slab(me.handle() );

        net.conditionally_generate_root_index_seed(&me);

        me
    }
    pub fn handle(&self) -> SlabHandle {
        SlabHandle::new(self )
    }
    pub fn create_context (&self) -> Context {
        Context::new(self)
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        //println!("Slab({}).subscribe_subject({})", self.id, subject_id );
        let weakcontext : WeakContext = context.weak();

        match self.subject_subscriptions.write().unwrap().entry(subject_id){
            Entry::Occupied(mut e) => {
                e.get_mut().push(weakcontext)
            }
            Entry::Vacant(e) => {
                e.insert(vec![weakcontext]);
            }
        }
        return;
    }
    pub fn unsubscribe_subject (&self,  subject_id: u64, context: &Context ){
        if let Some(subs) = self.subject_subscriptions.write().unwrap().get_mut(&subject_id) {
            let weak_context = context.weak();
            subs.retain(|c| {
                c.cmp(&weak_context)
            });
            return;
        }
    }
    pub fn memo_wait_channel (&self, memo_id: MemoId ) -> futures::channel::oneshot::Receiver<Memo> {
        let (tx, rx) = futures::channel::oneshot::channel::<Memo>(10);

        match self.memo_wait_channels.lock().unwrap().entry(memo_id) {
            Entry::Vacant(o)       => { o.insert( vec![tx] ); }
            Entry::Occupied(mut o) => { o.get_mut().push(tx); }
        };

        rx
    }
    pub fn generate_subject_id(&self) -> SubjectId {
        let mut counters = self.counters.write().unwrap();
        counters.last_subject_id += 1;
        (self.id as u64).rotate_left(32) | counters.last_subject_id as u64
    }
    fn _memo_durability_score( &self, _memo: &Memo ) -> u8 {
        // TODO: devise durability_score algo
        //       Should this number be inflated for memos we don't care about?
        //       Or should that be a separate signal?

        // Proposed factors:
        // Estimated number of copies in the network (my count = count of first order peers + their counts weighted by: uptime?)
        // Present diasporosity ( my diasporosity score = mean peer diasporosity scores weighted by what? )
        0
    }

    // Counters,stats, reporting
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        self.memorefs_by_id.read().unwrap().len() as u32
    }
    pub fn count_of_memos_received( &self ) -> u64 {
        self.counters.read().unwrap().memos_received as u64
    }
    pub fn count_of_memos_reduntantly_received( &self ) -> u64 {
        self.counters.read().unwrap().memos_redundantly_received as u64
    }
    pub fn peer_slab_count (&self) -> usize {
        self.peer_refs.read().unwrap().len() as usize
    }
    pub fn remotize_memo_ids( &self, memo_ids: &[MemoId] ) -> Result<(),String>{
        //println!("# Slab({}).remotize_memo_ids({:?})", self.id, memo_ids);

        let mut memorefs : Vec<MemoRef> = Vec::with_capacity(memo_ids.len());

        {
            let memorefs_by_id = self.memorefs_by_id.read().unwrap();
            for memo_id in memo_ids.iter() {
                if let Some(memoref) = memorefs_by_id.get(memo_id) {
                    memorefs.push( memoref.clone() )
                }
            }
        }

        for memoref in memorefs {
            self.remotize_memoref(&memoref)?;
        }

        Ok(())
    }
    pub fn slabref_from_local_slab(&self, peer_slab: &SlabHandle) -> SlabRef {

        //let args = TransmitterArgs::Local(&peer_slab);
        let presence = SlabPresence{
            slab_id: peer_slab.id,
            address: TransportAddress::Local,
            lifetime: SlabAnticipatedLifetime::Unknown
        };

        self.assert_slabref(peer_slab.id, &vec![presence])
    }
}

impl fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("peer_refs", &self.peer_refs)
            .field("memo_refs", &self.memorefs_by_id)
            .finish()
    }
}