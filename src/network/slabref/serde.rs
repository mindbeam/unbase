use serde::*;
use serde::ser::*;
use serde::de::*;
use super::*;
use subject::*;

impl Serialize for SlabRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.slab_id.to_string())?;
        seq.serialize_element(&"127.0.0.1:12345".to_string())?;
        seq.end()

    }
}


pub struct SlabRefSeed<'a> { pub net: &'a Network }
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
       let id: SlabId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(0, &self));
           }
       };
       let address: TransportAddress = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(1, &self));
           }
       };

       let presence = SlabPresence {
           slab_id: id,
           transport_address: address,
           anticipated_lifetime: SlabAnticipatedLifetime::Unknown
       };

       Ok( self.net.assert_slabref_from_presence(&presence) )
    }
}
