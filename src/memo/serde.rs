
use serde::*;
use serde::ser::*;
use serde::de::*;
use super::*;
use memo::*;
use std::fmt;
use network::Network;
use memoref::serde::*;
use memorefhead::serde::*;

impl Serialize for Memo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element( &self.id )?;
        seq.serialize_element( &self.subject_id )?;
        seq.serialize_element( &self.inner.parents )?;
        seq.serialize_element( &self.inner.body )?;
        seq.end()

    }
}

pub struct MemoSeed<'a> { pub net: &'a Network }

impl<'a> DeserializeSeed for MemoSeed<'a> {
    type Value = Memo;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MemoSeed<'a>{
    type Value = Memo;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct Memo")
    }

    fn visit_seq<V> (self, mut visitor: V) -> Result<Memo, V::Error>
        where V: SeqVisitor
    {
       let id: MemoId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(0, &self));
           }
       };
       let subject_id: SubjectId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(1, &self));
           }
       };
       let parents: MemoRefHead = match visitor.visit_seed(MemoRefHeadSeed{ net: self.net })? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(2, &self));
           }
       };
       let body: MemoBody = match visitor.visit_seed(MemoBodySeed{ net: self.net })? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(3, &self));
           }
       };

       Ok(Memo::new( id, subject_id, parents, body ))
    }
}



/*
    SlabPresence(SlabPresence),
    Relation(HashMap<u8,(SubjectId,MemoRefHead)>),
    Edit(HashMap<String, String>),
    FullyMaterialized     { v: HashMap<String, String>, r: HashMap<u8,(SubjectId,MemoRefHead)> },
    PartiallyMaterialized { v: HashMap<String, String>, r: HashMap<u8,(SubjectId,MemoRefHead)> },
    Peering(MemoId,SlabRef,PeeringStatus),
    MemoRequest(Vec<MemoId>,SlabRef)
*/

pub struct MemoBodySeed<'a> { net: &'a Network }
pub struct MemoBodyVariantSeed<'a> { net: &'a Network }

enum MemoBodyVariant {
    SlabPresence,
    Relation,
    Edit,
    FullyMaterialized,
    PartiallyMaterialized,
    Peering,
    MemoRequest
}

impl<'a> DeserializeSeed for MemoBodySeed<'a> {
    type Value = MemoBody;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {

        const VARIANTS: &'static [&'static str] = &[
            "SlabPresence",
            "Relation",
            "Edit",
            "FullyMaterialized",
            "PartiallyMaterialized",
            "Peering",
            "MemoRequest"
        ];

        deserializer.deserialize_enum("MemoBody", VARIANTS, self)
    }
}
impl<'a> Visitor for MemoBodySeed<'a> {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("Sequence of MemoRefs")
    }
    fn visit_enum<V>(self, visitor: V) -> Result<MemoBody, V::Error>
        where V: EnumVisitor
    {

        match try!(visitor.visit_variant_seed(self)) {
            (MemoBodyVariant::SlabPresence, variant) => variant.visit_newtype(),
        //    (MemoBodyVariant::Relation,     variant) => variant.visit_newtype().map(MemoBody::Relation),
        //    (MemoBodyVariant::Edit, variant) => variant.visit_newtype().map(MemoBody::Edit),
        //    (MemoBodyVariant::FullyMaterialized, variant) => variant.visit_newtype().map(MemoBody::FullyMaterialized),
        //    (MemoBodyVariant::PartiallyMaterialized, variant) => variant.visit_newtype().map(MemoBody::PartiallyMaterialized),
        //    (MemoBodyVariant::Peering, variant) => variant.visit_newtype().map(MemoBody::Peering),
        //    (MemoBodyVariant::MemoRequest, variant) => variant.visit_newtype().map(MemoBody::MemoRequest),
        }
    }
}

impl<'a> DeserializeSeed for MemoBodyVariantSeed<'a> {
    fn deserialize<D>(deserializer: D) -> Result<MemoBodyVariant, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;
        deserializer.deserialize(FieldVisitor)
    }
}

impl Visitor for MemoBodyVariant {
    type Value = Field;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`Ok` or `Err`")
    }

    fn visit_str<E>(self, value: &str) -> Result<Field, E>
        where E: Error
    {
        match value {
            "Ok" => Ok(Field::Ok),
            "Err" => Ok(Field::Err),
            _ => Err(Error::unknown_variant(value, VARIANTS)),
        }
    }
}
