use util::serde::*;
use network::TransportAddress;
use super::*;


impl<'a> StatefulSerialize for &'a SlabPresence {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut sv = serializer.serialize_struct("SlabPresence", 3)?;
        sv.serialize_field("slab_id",  &self.slab_id)?;

        sv.serialize_field("address", match self.address {
            TransportAddress::Local   => helper.return_address,
            TransportAddress::UDP(_)  => &self.address,
            _ => return Err(SerError::custom("Address does not support serialization"))
        })?;
        sv.serialize_field("lifetime", &self.lifetime ) ?;
        sv.end()
    }
}
impl StatefulSerialize for SlabPresence {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut sv = serializer.serialize_struct("SlabPresence", 3)?;
        sv.serialize_field("slab_id",  &self.slab_id)?;

        sv.serialize_field("address", match self.address {
            TransportAddress::Local   => helper.return_address,
            TransportAddress::UDP(_)  => &self.address,
            _ => return Err(SerError::custom("Address does not support serialization"))
        })?;
        sv.serialize_field("lifetime", &self.lifetime ) ?;
        sv.end()
    }
}

impl StatefulSerialize for SlabRef {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        // TODO: Should actually be a sequence of slab presences
        // to allow for slabs with multiple transports
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element( &SerializeWrapper(&*(self.0.presence.lock().unwrap()),helper) )?;
        seq.end()
    }
}


pub struct SlabRefSeed<'a> { pub dest_slab: &'a Slab }
impl<'a> DeserializeSeed for SlabRefSeed<'a> {
    type Value = SlabRef;

    fn deserialize<D> (self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {

        deserializer.deserialize_seq( self )
    }
}

impl<'a> Visitor for SlabRefSeed<'a> {
    type Value = SlabRef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct SlabRef")
    }

    fn visit_seq<V> (self, mut visitor: V) -> Result<SlabRef, V::Error>
        where V: SeqVisitor
    {
       let presence: SlabPresence = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(0, &self));
           }
       };
       let slabref;
       {
           slabref = self.dest_slab.inner().slabref_from_presence(&presence).expect("slabref from slabrefseed presence");
       }

       Ok( slabref )
    }
}
