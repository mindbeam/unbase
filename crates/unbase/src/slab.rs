//TODO MERGE topic/topo-compaction3

use futures::{
    StreamExt,
    channel::mpsc,
    future::RemoteHandle,
};
use std::collections::hash_map::Entry;

pub use self::{
    common_structs::*,
    slabref::{SlabRef, SlabRefInner},
    memoref::{MemoRef, MemoRefInner, MemoRefPtr},
    memo::{MemoId, Memo, MemoInner, MemoBody},
    memoref::serde as memoref_serde,
    memo::serde as memo_serde,
    handle::SlabHandle,
};


use crate::{
    context::Context,
    memorefhead::*,
    network::{
        Network,
        Transmitter,
        TransportAddress
    },
    slab::agent::SlabAgent,
    subject::SubjectId,
};

use std::sync::{Arc,RwLock,Mutex};
use std::ops::Deref;

mod state;
pub (crate) mod agent;
mod common_structs;
mod handle;

mod memo;
mod slabref;
mod memoref;

pub type SlabId = u32;

#[derive(Clone)]
pub struct Slab{
    pub id: SlabId,
    pub (crate) agent: Arc<SlabAgent>,
    pub (crate) net: Network,
    pub my_ref: SlabRef,
    dispatch_channel: mpsc::Sender<MemoRef>,
    dispatcher: Arc<RemoteHandle<()>>,
    handle: SlabHandle,
}

impl Deref for Slab {
    type Target = SlabHandle;
    fn deref(&self) -> &SlabHandle {
        &self.handle
    }
}

impl Slab {
    #[tracing::instrument]
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

        let agent = Arc::new(SlabAgent::new(net, my_ref.clone()));

        let agent2 = agent.clone();
        let dispatcher_task = (async move || {
            let mut dispatch_rx_channel = dispatch_rx_channel;
            while let Some(memoref) = dispatch_rx_channel.next().await {
                agent2.recv_memoref(memoref);
            }
        })();

        let dispatcher: RemoteHandle<()> = crate::util::task::spawn_with_handle(dispatcher_task);

        let handle = SlabHandle {
            my_ref: my_ref.clone(),
            net: net.clone(),
            dispatch_channel: dispatch_tx_channel.clone(),
            agent: agent.clone()
        };

        let me = Slab {
            id,
            dispatch_channel: dispatch_tx_channel,
            dispatcher: Arc::new(dispatcher),
            net: net.clone(),
            my_ref: my_ref,
            handle,
            agent
        };

        net.register_local_slab(me.handle());

        net.conditionally_generate_root_index_seed(&me.handle);

        me
    }
    pub fn handle(&self) -> SlabHandle {
        self.handle.clone()
    }
    pub fn create_context(&self) -> Context {
        Context::new(self.handle())
    }
    pub (crate) fn observe_index (&self, tx: mpsc::Sender<MemoRefHead> ) {
        self.index_subscriptions.lock().unwrap().push(tx);
    }
    fn _memo_durability_score(&self, _memo: &Memo) -> u8 {
        // TODO: devise durability_score algo
        //       Should this number be inflated for memos we don't care about?
        //       Or should that be a separate signal?

        // Proposed factors:
        // Estimated number of copies in the network (my count = count of first order peers + their counts weighted by: uptime?)
        // Present diasporosity ( my diasporosity score = mean peer diasporosity scores weighted by what? )
        0
    }
}

impl Drop for Slab {
    fn drop(&mut self) {
        self.agent.stop();
        self.net.deregister_local_slab(self.id);
    }
}

impl std::fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("agent", &self.agent)
            .finish()
    }
}