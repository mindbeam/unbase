use super::*;
use super::super::*;

use memo::serde::MemoSeed;
use util::serde::DeserializeSeed;
use util::serde::*;

impl StatefulSerialize for Packet {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element( &self.from_slab_id )?;
        seq.serialize_element( &self.from_slab_peering_status )?;
        seq.serialize_element( &self.to_slab_id )?;
        seq.serialize_element( &SerializeWrapper( &self.memo, helper ) )?;
        seq.end()
    }
}

// can't use this because we don't have the same deserialize seed for all fields
/*impl StatefulSerialize for Packet {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut sv = serializer.serialize_struct("Memoref", 4)?;
        sv.serialize_field("from_slab_id",    &self.from_slab_id )?;
        sv.serialize_field("peering_status",  &self.from_slab_peering_status )?;
        sv.serialize_field("to_slab_id",      &self.to_slab_id )?;
        sv.serialize_field("memo",            &SerializeWrapper( &self.memo, helper ) )?;
        sv.end()
    }
}*/

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
               return Err(DeError::invalid_length(0, &self));
           }
       };
       let from_slab_peering_status: PeeringStatus = match visitor.visit()?{
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(1, &self));
           }
       };
       let to_slab_id: SlabId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(2, &self));
           }
       };
       let memo: Memo = match visitor.visit_seed( MemoSeed { net: self.net } )? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(3, &self));
           }
       };

       Ok(Packet{
           from_slab_id: from_slab_id,
           to_slab_id:   to_slab_id,
           from_slab_peering_status: from_slab_peering_status,
           memo:         memo
       })
   }
}
