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
impl Deserialize for SlabRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct SlabRefVisitor;
        impl Visitor for SlabRefVisitor {
            type Value = SlabRef;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
               formatter.write_str("struct SlabRef")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<SlabRef, V::Error>
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

               // Ooof, looks like we need a deserialization wrapper
               Ok(SlabRef::new_from_presence(presence, network))
            }
        }


        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Duration", FIELDS, SlabRefVisitor)

    }
}
