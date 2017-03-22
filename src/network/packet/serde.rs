use super::*;
use super::super::*;

use memo::serde::MemoSeed;

use serde::*;
use serde::ser::*;
use serde::de::*;

impl Serialize for Packet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element( &self.from_slab_id )?;
        seq.serialize_element( &self.to_slab_id )?;
        seq.serialize_element( &self.memo )?;
        seq.end()

    }
}

pub struct PacketSeed <'a>{ pub net: &'a Network }

impl<'a> DeserializeSeed for PacketSeed<'a>{
    type Value = Packet;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq( self )
    }
}

impl<'a> Visitor for PacketSeed<'a> {
    type Value = Packet;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Packet")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {
       let from_slab_id: SlabId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(0, &self));
           }
       };
       let to_slab_id: SlabId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(1, &self));
           }
       };
       let memo: Memo = match visitor.visit_seed( MemoSeed { net: self.net } )? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(2, &self));
           }
       };

       Ok(Packet{
           from_slab_id: from_slab_id,
           to_slab_id:   to_slab_id,
           memo:         memo
       })
   }
}
