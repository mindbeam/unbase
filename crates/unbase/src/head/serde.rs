use crate::{
    head::Head,
    slab::{
        memoref_serde::*,
        MemoRef,
        SlabHandle,
        SlabRef,
        SubjectId,
    },
    util::serde::{
        DeError,
        SerializeHelper,
        SerializeWrapper,
        StatefulSerialize,
        VecSeed,
    },
};
use serde::{
    de::*,
    ser::*,
};
use std::fmt;

impl StatefulSerialize for Head {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            Head::Null => {
                let sv = serializer.serialize_struct_variant("Head", 0, "Null", 0)?;
                sv.end()
            },
            Head::Anonymous { ref head, .. } => {
                let mut sv = serializer.serialize_struct_variant("Head", 1, "Anonymous", 1)?;
                sv.serialize_field("h", &SerializeWrapper(head, helper))?;
                sv.end()
            },
            Head::Subject { ref subject_id,
                                   ref head,
                                   .. } => {
                let mut sv = serializer.serialize_struct_variant("Head", 2, "Subject", 3)?;
                sv.serialize_field("s", &subject_id)?;
                sv.serialize_field("h", &SerializeWrapper(&head, helper))?;
                sv.end()
            },
        }
    }
}

pub struct HeadSeed<'a> {
    pub dest_slab:      &'a SlabHandle,
    pub origin_slabref: &'a SlabRef,
}

#[derive(Deserialize)]
enum MRHVariant {
    Null,
    Anonymous,
    Subject,
}

impl<'a> DeserializeSeed for HeadSeed<'a> {
    type Value = Head;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        const MRH_VARIANTS: &'static [&'static str] = &["Null", "Anonymous", "Subject"];

        deserializer.deserialize_enum("Head", MRH_VARIANTS, self)
    }
}

impl<'a> Visitor for HeadSeed<'a> {
    type Value = Head;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Head")
    }

    fn visit_enum<V>(self, visitor: V) -> Result<Head, V::Error>
        where V: EnumVisitor
    {
        let foo = match visitor.visit_variant()? {
            (MRHVariant::Null, variant) => variant.visit_newtype_seed(MRHNullSeed {}),
            (MRHVariant::Anonymous, variant) => {
                variant.visit_newtype_seed(MRHAnonymousSeed { dest_slab:      self.dest_slab,
                                                              origin_slabref: self.origin_slabref, })
            },
            (MRHVariant::Subject, variant) => {
                variant.visit_newtype_seed(MRHSubjectSeed { dest_slab:      self.dest_slab,
                                                            origin_slabref: self.origin_slabref, })
            },
        };

        foo
    }
}

struct MRHNullSeed {}

impl<'a> DeserializeSeed for MRHNullSeed {
    type Value = Head;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}
impl<'a> Visitor for MRHNullSeed {
    type Value = Head;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Head::Null")
    }

    fn visit_map<Visitor>(self, _visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor
    {
        Ok(Head::Null)
    }
}

struct MRHAnonymousSeed<'a> {
    dest_slab:      &'a SlabHandle,
    origin_slabref: &'a SlabRef,
}

impl<'a> DeserializeSeed for MRHAnonymousSeed<'a> {
    type Value = Head;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}
impl<'a> Visitor for MRHAnonymousSeed<'a> {
    type Value = Head;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Head::Anonymous")
    }

    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor
    {
        let mut head: Option<Vec<MemoRef>> = None;
        while let Some(key) = visitor.visit_key()? {
            match key {
                'h' => {
                    head = Some(visitor.visit_value_seed(VecSeed(MemoRefSeed { dest_slab:      self.dest_slab,
                                                                        origin_slabref: self.origin_slabref, }))?)
                },
                _ => {},
            }
        }

        if head.is_some() {
            Ok(Head::Anonymous { owning_slab_id: self.dest_slab.my_ref.slab_id,
                                        head:           head.unwrap(), })
        } else {
            Err(DeError::invalid_length(0, &self))
        }
    }
}

struct MRHSubjectSeed<'a> {
    dest_slab:      &'a SlabHandle,
    origin_slabref: &'a SlabRef,
}
impl<'a> DeserializeSeed for MRHSubjectSeed<'a> {
    type Value = Head;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(self)
    }
}
impl<'a> Visitor for MRHSubjectSeed<'a> {
    type Value = Head;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Head::Subject")
    }

    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where Visitor: MapVisitor
    {
        let mut head: Option<Vec<MemoRef>> = None;
        let mut subject_id: Option<SubjectId> = None;
        while let Some(key) = visitor.visit_key()? {
            match key {
                's' => subject_id = Some(visitor.visit_value()?),
                'h' => {
                    head = Some(visitor.visit_value_seed(VecSeed(MemoRefSeed { dest_slab:      self.dest_slab,
                                                                        origin_slabref: self.origin_slabref, }))?)
                },
                _ => {},
            }
        }

        if head.is_some() && subject_id.is_some() {
            Ok(Head::Subject { owning_slab_id: self.dest_slab.my_ref.slab_id,
                                      head:           head.unwrap(),
                                      subject_id:     subject_id.unwrap(), })
        } else {
            Err(DeError::invalid_length(0, &self))
        }
    }
}
