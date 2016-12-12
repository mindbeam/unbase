use std::fmt;

pub struct PeerSlab {
    id: u32
}
pub enum PeerSpec {
    Any (u8),
    List(Vec<PeerSlab>)
}

impl fmt::Debug for PeerSlab{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("PeerSlab")
           .field("id", &self.id)
           .finish()
    }
}
impl fmt::Debug for PeerSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let mut dbg = fmt.debug_struct("PeerSpec");

        match self {
            &PeerSpec::Any(c)  => {
                dbg.field("Any", &c);
            },
            &PeerSpec::List(ref v) => {
                for p in v {
                    dbg.field("Peer", &p);
                }
            }
        };

        dbg.finish()
    }
}
