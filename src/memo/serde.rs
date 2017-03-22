
use serde::*;
use serde::ser::*;
use serde::de::*;
use super::*;
use std::fmt;
use network::Network;
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

enum MemoBodyVariant {
    SlabPresence,
    Relation,
    Edit,
    FullyMaterialized,
    PartiallyMaterialized,
    Peering,
    MemoRequest
}

const MEMOBODY_VARIANTS: &'static [&'static str] = &[
    "SlabPresence",
    "Relation",
    "Edit",
    "FullyMaterialized",
    "PartiallyMaterialized",
    "Peering",
    "MemoRequest"
];

impl<'a> DeserializeSeed for MemoBodySeed<'a> {
    type Value = MemoBody;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_enum("MemoBody", MEMOBODY_VARIANTS, self)
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

        match try!(visitor.visit_variant()) {
            (MemoBodyVariant::SlabPresence, variant) => variant.visit_newtype().map(MemoBody::SlabPresence),
            (MemoBodyVariant::Relation,     variant) => variant.visit_newtype_seed(MBRelationSeed{ net: self.net }).map(Ok)?,
            _ => panic!("meow")
        //    (MemoBodyVariant::Edit, variant) => variant.visit_newtype().map(MemoBody::Edit),
        //    (MemoBodyVariant::FullyMaterialized, variant) => variant.visit_newtype().map(MemoBody::FullyMaterialized),
        //    (MemoBodyVariant::PartiallyMaterialized, variant) => variant.visit_newtype().map(MemoBody::PartiallyMaterialized),
        //    (MemoBodyVariant::Peering, variant) => variant.visit_newtype().map(MemoBody::Peering),
        //    (MemoBodyVariant::MemoRequest, variant) => variant.visit_newtype().map(MemoBody::MemoRequest),
        }
    }
}

impl Deserialize for MemoBodyVariant {
    fn deserialize<D>(deserializer: D) -> Result<MemoBodyVariant, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(MemoBodyVariantVisitor)
    }
}
struct MemoBodyVariantVisitor;
impl Visitor for MemoBodyVariantVisitor
{
    type Value = MemoBodyVariant;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("RelationSlotId")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        match value {
            "SlabPresence"            => Ok(MemoBodyVariant::SlabPresence),
            "Relation"                => Ok(MemoBodyVariant::Relation),
            "Edit"                    => Ok(MemoBodyVariant::Edit),
            "FullyMaterialized"       => Ok(MemoBodyVariant::FullyMaterialized),
            "PartiallyMaterialized"   => Ok(MemoBodyVariant::PartiallyMaterialized),
            "Peering"                 => Ok(MemoBodyVariant::Peering),
            "MemoRequest"             => Ok(MemoBodyVariant::MemoRequest),
            _ => Err(serde::de::Error::unknown_field(value, MEMOBODY_VARIANTS)),
        }
    }
}

struct MBRelationSeed<'a> { net: &'a Network }
impl<'a> DeserializeSeed for MBRelationSeed<'a> {
    type Value = MemoBody;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}
impl<'a> Visitor for MBRelationSeed<'a> {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`Ok` or `Err`")
    }

    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor,
    {
        let mut values = HashMap::new();

        while let Some((slot, (subject_id,mrh))) = try!(visitor.visit_seed(RelationSlotIdSeed, SubjectMRHSeed{ net: self.net })) {
            values.insert(slot, (subject_id,mrh));
        }

        Ok(MemoBody::Relation(values))
    }
}

struct SubjectMRHSeed<'a> { net: &'a Network }
impl<'a> DeserializeSeed for SubjectMRHSeed<'a> {
    type Value = (SubjectId,MemoRefHead);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }

}

impl<'a> Visitor for SubjectMRHSeed<'a> {
    type Value = (SubjectId,MemoRefHead);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Subject+MRH tuple")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where V: SeqVisitor
    {

        let subject_id : SubjectId = match visitor.visit()? {
            Some(value) => value,
            None => {
                return Err(de::Error::invalid_length(0, &self));
            }
        };
        let mrh : MemoRefHead = match visitor.visit_seed(MemoRefHeadSeed{ net: self.net })? {
            Some(value) => value,
            None => {
                return Err(de::Error::invalid_length(1, &self));
            }
        };

        Ok((subject_id,mrh))
    }
}

/*
struct MemoBodyBasicVisitor;

impl de::Visitor for MemoBodyBasicVisitor {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("MemoBody")
    }

    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor
    {
        (MemoBodyVariant::SlabPresence, variant) => variant.visit_newtype().map(Ok)?, //(MBSlabPresenceVisitor{net: self.net })
        (MemoBodyVariant::Edit, variant) => variant.visit_newtype().map(MemoBody::Edit),
        _ => Err(serde::de::Error::unknown_field(value, MEMOBODY_VARIANTS)),

    }

    // Similar for other methods:
    //   - visit_i16
    //   - visit_u8
    //   - visit_u16
    //   - visit_u32
    //   - visit_u64
}
*/
