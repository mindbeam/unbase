
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


/*
    SlabPresence(SlabPresence),
    Relation(HashMap<u8,(SubjectId,MemoRefHead)>),
    Edit(HashMap<String, String>),
    FullyMaterialized     { v: HashMap<String, String>, r: HashMap<u8,(SubjectId,MemoRefHead)> },
    PartiallyMaterialized { v: HashMap<String, String>, r: HashMap<u8,(SubjectId,MemoRefHead)> },
    Peering(MemoId,SlabRef,PeeringStatus),
    MemoRequest(Vec<MemoId>,SlabRef)
*/
/*
impl Serialize for MemoBody {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use super::MemoBody::*;
        match *self {
            SlabPresence(ref p) => {
                serializer.serialize_newtype_struct("SlabPresence",p)
            },
            Relation(ref rhm) => {
                serializer.serialize_newtype_struct("Relation",rhm)
            },
            Edit(ref e) => {
                serializer.serialize_newtype_struct("Edit",e)
            },
            FullyMaterialized{ ref r, ref v }  => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element( &r )?;
                seq.serialize_element( &v )?;
                seq.end()
            },
            _ => { panic!("woof") }
        }

    }
}
*/

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

pub struct MemoBodySeed<'a> { net: &'a Network }

enum MBVariant {
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
            (MBVariant::SlabPresence,      variant) => variant.visit_newtype().map(MemoBody::SlabPresence),
            (MBVariant::Relation,          variant) => variant.visit_newtype_seed(RelationMRHSeed{ net: self.net }).map(MemoBody::Relation),
            (MBVariant::Edit,              variant) => variant.visit_newtype().map(MemoBody::Edit),
            (MBVariant::FullyMaterialized, variant) => variant.visit_newtype_seed(MBFullyMaterializedSeed{ net: self.net }),
        //    (MBVariant::PartiallyMaterialized, variant) => variant.visit_newtype().map(MemoBody::PartiallyMaterialized),
            (MBVariant::Peering,           variant) => variant.visit_newtype_seed(MBPeeringSeed{ net: self.net }),
        //    (MBVariant::MemoRequest, variant) => variant.visit_newtype().map(MemoBody::MemoRequest),
            _ => panic!("meow")

        }
    }
}

impl Deserialize for MBVariant {
    fn deserialize<D>(deserializer: D) -> Result<MBVariant, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(MBVariantVisitor)
    }
}
struct MBVariantVisitor;
impl Visitor for MBVariantVisitor
{
    type Value = MBVariant;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("RelationSlotId")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        match value {
            "SlabPresence"            => Ok(MBVariant::SlabPresence),
            "Relation"                => Ok(MBVariant::Relation),
            "Edit"                    => Ok(MBVariant::Edit),
            "FullyMaterialized"       => Ok(MBVariant::FullyMaterialized),
            "PartiallyMaterialized"   => Ok(MBVariant::PartiallyMaterialized),
            "Peering"                 => Ok(MBVariant::Peering),
            "MemoRequest"             => Ok(MBVariant::MemoRequest),
            _ => Err(serde::de::Error::unknown_field(value, MEMOBODY_VARIANTS)),
        }
    }
}

struct RelationMRHSeed<'a> { net: &'a Network }
impl<'a> DeserializeSeed for RelationMRHSeed<'a> {
    type Value = HashMap<u8,(SubjectId,MemoRefHead)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}

impl<'a> Visitor for RelationMRHSeed<'a> {
    type Value = HashMap<u8,(SubjectId,MemoRefHead)>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("MemoBody::Relation")
    }

    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor,
    {
        let mut values = HashMap::new();

        while let Some((slot, (subject_id,mrh))) = try!(visitor.visit_seed(RelationSlotIdSeed, SubjectMRHSeed{ net: self.net })) {
            values.insert(slot, (subject_id,mrh));
        }

        Ok(values)
    }
}

struct MBFullyMaterializedSeed<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MBFullyMaterializedSeed<'a> {
    type Value = MemoBody;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}
impl<'a> Visitor for MBFullyMaterializedSeed<'a> {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("MemoBody::FullyMaterialized")
    }
    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor,
    {

        let mut relations = None;
        let mut values    = None;
        while let Some(key) = visitor.visit_key()? {
            match key {
                'r' => relations = Some(visitor.visit_value_seed(RelationMRHSeed{ net: self.net })?),
                'v' => values    = visitor.visit_value()?,
                _   => {}
            }
        }
        if relations.is_some() && values.is_some() {
            Ok(MemoBody::FullyMaterialized{ r: relations.unwrap(), v: values.unwrap() })
        }else{
            Err(de::Error::invalid_length(0, &self))
        }
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


struct MBPeeringSeed<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MBPeeringSeed<'a> {
    type Value = MemoBody;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}
impl<'a> Visitor for MBPeeringSeed<'a> {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("MemoBody::Peering")
    }
    fn visit_seq<V> (self, mut visitor: V) -> Result<MemoBody, V::Error>
        where V: SeqVisitor
    {
       let memo_id: MemoId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(0, &self));
           }
       };
       let presence: SlabPresence = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(1, &self));
           }
       };
       let peering_status: PeeringStatus = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(2, &self));
           }
       };

       Ok(MemoBody::Peering( memo_id, presence, peering_status ))
    }
}
