extern crate core;
extern crate linked_hash_map;
extern crate itertools;


#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

//#[doc(inline)]
pub mod network;
pub mod slab;
pub mod subject;
pub mod context;
pub mod error;
pub mod index;
pub mod memorefhead;
pub mod util;

pub use network::Network;
pub use subject::Subject;
pub use slab::Slab;

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
