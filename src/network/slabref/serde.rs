use serde::*;
use serde::ser::*;
use serde::de::*;
use super::*;

impl Serialize for SlabRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.presence)?;
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
      /* let id: SlabId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(0, &self));
           }
       };*/
       let presence: SlabPresence = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(0, &self));
           }
       };

      /* let presence = SlabPresence {
           slab_id: id,
           transport_address: address,
           anticipated_lifetime: SlabAnticipatedLifetime::Unknown
       };*/

       println!("MARK SERDE");
       Ok( self.net.assert_slabref_from_presence(&presence) )
    }
}
