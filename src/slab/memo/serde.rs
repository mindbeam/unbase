
use serde::*;
use serde::ser::*;
use serde::de::*;
use super::*;
use std::fmt;
use network::Network;
use memorefhead::serde::*;

use slab::slabref::serde::SlabRefSeed;
use slab::MemoOrigin;
use util::serde::*;

impl StatefulSerialize for Memo {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element( &self.id )?;
        seq.serialize_element( &self.subject_id )?;
        seq.serialize_element( &SerializeWrapper( &self.inner.body, helper ) )?;
        seq.serialize_element( &SerializeWrapper( &self.inner.parents, helper ) )?;
        seq.end()
    }
}

impl StatefulSerialize for MemoBody {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use super::MemoBody::*;
        match *self {
            SlabPresence{ ref p, ref r } =>{
                let mut sv = serializer.serialize_struct_variant("MemoBody", 0, "SlabPresence", 2)?;
                sv.serialize_field("p", &SerializeWrapper(&p, helper))?;
                sv.serialize_field("r", &SerializeWrapper(r, helper))?;
                sv.end()
            }
            Relation(ref rhm) => {
                //let mut sv = serializer.serialize_struct_variant("MemoBody", 1, "Relation", 1)?;
                //sv.serialize_field("r", &SerializeWrapper(rhm, helper))?;
                //sv.end()
                serializer.serialize_newtype_variant("MemoBody", 1, "Relation", &SerializeWrapper(rhm, helper) )
            },
            Edit(ref e) => {
                let mut sv = serializer.serialize_struct_variant("MemoBody", 2, "Edit", 1)?;
                sv.serialize_field("e", e )?;
                sv.end()
            },
            FullyMaterialized{ ref r, ref v }  => {
                let mut sv = serializer.serialize_struct_variant("MemoBody", 3, "FullyMaterialized", 2)?;
                sv.serialize_field("r", &SerializeWrapper(r, helper))?;
                sv.serialize_field("v", v)?;
                sv.end()
            },
            PartiallyMaterialized{ ref r, ref v }  => {
                let mut sv = serializer.serialize_struct_variant("MemoBody", 3, "PartiallyMaterialized", 2)?;
                sv.serialize_field("r", &SerializeWrapper(r, helper))?;
                sv.serialize_field("v", v)?;
                sv.end()
            },
            Peering( ref memo_id, ref slabpresence, ref peeringstatus ) =>{
                let mut sv = serializer.serialize_struct_variant("MemoBody", 4, "Peering", 3)?;
                sv.serialize_field("i", memo_id )?;
                sv.serialize_field("p", VecSeed(&SerializeWrapper(&slabpresence,helper)) )?;
                sv.serialize_field("s", peeringstatus)?;
                sv.end()
            }
            MemoRequest( ref memo_ids, ref slabref ) =>{
                let mut sv = serializer.serialize_struct_variant("MemoBody", 5, "MemoRequest", 2)?;
                sv.serialize_field("i", memo_ids )?;
                sv.serialize_field("s", &SerializeWrapper(slabref, helper))?;
                sv.end()
            }
        }

    }
}


impl StatefulSerialize for (SubjectId,MemoRefHead) {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element( &self.0 )?;
        seq.serialize_element( &SerializeWrapper( &self.1, helper ) )?;
        seq.end()
    }
}

pub struct MemoSeed<'a> {
    dest_slab: &'a Slab,
    from_presence: SlabPresence,
    from_slab_peering_status: MemoPeeringStatus
}

impl<'a> DeserializeSeed for MemoSeed<'a> {
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MemoSeed<'a>{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct Memo")
    }

    fn visit_seq<V> (self, mut visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor
    {
       let id: MemoId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(0, &self));
           }
       };
       let subject_id: SubjectId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(1, &self));
           }
       };
       let body: MemoBody = match visitor.visit_seed(MemoBodySeed{ net: self.net })? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(2, &self));
           }
       };
       let parents: MemoRefHead = match visitor.visit_seed(MemoRefHeadSeed{ net: self.net })? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(3, &self));
           }
       };

       let from = self.dest_slab.assert_slabref_from_presence(&self.from_presence);
       let memoorigin = MemoOrigin::OtherSlab(&from, self.from_slab_peering_status);
       let memoref = self.dest_slab.put_memo(&memoorigin,
           Memo::new(
               id,
               subject_id,
               parents,
               body,
               self.dest_slab
           )
       );

       //Ok(Memo::new( id, subject_id, parents, body ))
       Ok(())
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
       formatter.write_str("MemoBody")
    }
    fn visit_enum<V>(self, visitor: V) -> Result<MemoBody, V::Error>
        where V: EnumVisitor
    {

        match try!(visitor.visit_variant()) {
            (MBVariant::SlabPresence,      variant) => variant.visit_newtype_seed(MBSlabPresenceSeed{ net: self.net }),
            (MBVariant::Relation,          variant) => variant.visit_newtype_seed(RelationMRHSeed{ net: self.net }).map(MemoBody::Relation),
            (MBVariant::Edit,              variant) => variant.visit_newtype().map(MemoBody::Edit),
            (MBVariant::FullyMaterialized, variant) => variant.visit_newtype_seed(MBFullyMaterializedSeed{ net: self.net }),
        //  (MBVariant::PartiallyMaterialized, variant) => variant.visit_newtype().map(MemoBody::PartiallyMaterialized),
            (MBVariant::Peering,           variant) => variant.visit_newtype_seed(MBPeeringSeed{ net: self.net }),
            (MBVariant::MemoRequest,       variant) => variant.visit_newtype_seed(MBMemoRequestSeed{ net: self.net }),
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
       formatter.write_str("MemoBody Variant")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: DeError
    {
        match value {
            "SlabPresence"            => Ok(MBVariant::SlabPresence),
            "Relation"                => Ok(MBVariant::Relation),
            "Edit"                    => Ok(MBVariant::Edit),
            "FullyMaterialized"       => Ok(MBVariant::FullyMaterialized),
            "PartiallyMaterialized"   => Ok(MBVariant::PartiallyMaterialized),
            "Peering"                 => Ok(MBVariant::Peering),
            "MemoRequest"             => Ok(MBVariant::MemoRequest),
            _ => Err(serde::DeError::unknown_field(value, MEMOBODY_VARIANTS)),
        }
    }
}

#[derive(Clone)]
pub struct MBMemoRequestSeed<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MBMemoRequestSeed<'a> {
    type Value = MemoBody;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MBMemoRequestSeed<'a> {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("MemoBody::MemoRequest")
    }
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: MapVisitor
    {
        let mut memo_ids : Option<Vec<MemoId>> = None;
        let mut slabref  : Option<SlabRef> = None;
        while let Some(key) = visitor.visit_key()? {
            match key {
                'i' => memo_ids = visitor.visit_value()?,
                's' => slabref  = Some(visitor.visit_value_seed(SlabRefSeed{ net: self.net })?),
                _   => {}
            }
        }

        if memo_ids.is_some() && slabref.is_some() {

            Ok(MemoBody::MemoRequest( memo_ids.unwrap(), slabref.unwrap() ))
        }else{
            Err(DeError::invalid_length(0, &self))
        }
    }
}

struct RelationMRHSeed<'a> { net: &'a Network }
impl<'a> DeserializeSeed for RelationMRHSeed<'a> {
    type Value = HashMap<RelationSlotId,(SubjectId,MemoRefHead)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}

impl<'a> Visitor for RelationMRHSeed<'a> {
    type Value = HashMap<RelationSlotId,(SubjectId,MemoRefHead)>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("MemoBody::Relation")
    }

    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor,
    {
        let mut values : Self::Value = HashMap::new();

        while let Some(slot) = visitor.visit_key()? {
             let (subject_id ,mrh ) = visitor.visit_value_seed(SubjectMRHSeed{ net: self.net })?;
             values.insert(slot, (subject_id,mrh));
        }

        Ok(values)
    }
}

struct MBSlabPresenceSeed <'a> { net: &'a Network }
impl<'a> DeserializeSeed for MBSlabPresenceSeed<'a> {
    type Value = MemoBody;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}

impl<'a> Visitor for MBSlabPresenceSeed<'a> {
    type Value = MemoBody;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("MemoBody::SlabPresence")
    }
    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor,
    {

        let mut presence  = None;
        let mut root_index_seed : Option<Option<MemoRefHead>>   = None;
        while let Some(key) = visitor.visit_key()? {
            match key {
                'p' => presence        = visitor.visit_value()?,
                'r' => root_index_seed = Some(visitor.visit_value_seed(OptionSeed(MemoRefHeadSeed{ net: self.net }))?),
                _   => {}
            }
        }
        if presence.is_some() &&root_index_seed.is_some() {
            Ok(MemoBody::SlabPresence{ p: presence.unwrap(), r: root_index_seed.unwrap() })
        }else{
            Err(DeError::invalid_length(0, &self))
        }
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
            Err(DeError::invalid_length(0, &self))
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
                return Err(DeError::invalid_length(0, &self));
            }
        };
        let mrh : MemoRefHead = match visitor.visit_seed(MemoRefHeadSeed{ net: self.net })? {
            Some(value) => value,
            None => {
                return Err(DeError::invalid_length(1, &self));
            }
        };

        Ok((subject_id,mrh))
    }
}

// TODO convert this to a non-seed deserializer
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
    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor,
    {
        let _ = self.net;
        let mut memo_ids : Option<MemoId> = None;
        let mut presence : Option<SlabPresence> = None;
        let mut status   : Option<PeeringStatus> = None;
        while let Some(key) = visitor.visit_key()? {
            match key {
                'i' => memo_ids  = visitor.visit_value()?,
                'p' => presence  = visitor.visit_value()?,
                's' => status    = visitor.visit_value()?,
                _   => {}
            }
        }

        if memo_ids.is_some() && presence.is_some() && status.is_some() {

            Ok(MemoBody::Peering( memo_ids.unwrap(), presence.unwrap(), status.unwrap() ))
        }else{
            Err(DeError::invalid_length(0, &self))
        }

    }
}
