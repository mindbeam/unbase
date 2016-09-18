use std::sync::{Arc,Mutex};
//use std::error::Error;
//use std::{thread, time};

struct NetworkInternals{
    next_slab_id: u32
}
pub struct NetworkShared {
    internals: Mutex<NetworkInternals>
}
pub struct Network {
    shared: Arc<NetworkShared>
}

/// Returns a new `Pool` referencing the same state as `self`.
impl Clone for Network {
    fn clone(&self) -> Network {
        Network {
            shared: self.shared.clone()
        }
    }
}
/*
impl fmt::Debug for Network{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.0.internals.lock();

        fmt.debug_struct("Pool")
           .field("connections", &inner.num_conns)
           .field("idle_connections", &inner.conns.len())
           .field("config", &self.0.config)
           .field("manager", &self.0.manager)
           .finish()
    }
}
*/

impl Network {
    pub fn new() -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0
        };
        let shared = NetworkShared {
            internals: Mutex::new(internals)
        };
        Network {
            shared: Arc::new(shared)
        }
    }
    pub fn generate_slab_id(&self) -> u32 {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.next_slab_id += 1;

        internals.next_slab_id
    }
}
