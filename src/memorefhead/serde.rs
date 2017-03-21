use serde::de::*;
use network::*;
use memoref::serde::*;
use super::*;

pub struct MemoRefHeadSeed<'a> { pub net: &'a Network }

impl<'a> DeserializeSeed for MemoRefHeadSeed<'a> {
    type Value = MemoRefHead;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MemoRefHeadSeed<'a> {
    type Value = MemoRefHead;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("Sequence of MemoRefs")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {

        let mut memorefs : Vec<MemoRef> = Vec::new();

        while let Some(memopeer) = visitor.visit_seed( MemoRefSeed{ net: self.net })? {
            memorefs.push(memopeer);
        };

        Ok( MemoRefHead(memorefs) )
    }
}

pub struct RelationSlotIdSeed;
impl DeserializeSeed for RelationSlotIdSeed {
    type Value = RelationSlotId;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}

impl Visitor for RelationSlotIdSeed {
    type Value = RelationSlotId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("RelationSlotId")
    }
    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(value as Self::Value)
    }
}
