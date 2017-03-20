
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

pub struct MemoSeed<'a> { net: &'a Network }
struct MemoVisitor<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MemoSeed<'a> {
    type Value = Memo;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(MemoVisitor{ net: self.net })
    }
}

impl<'a> Visitor for MemoVisitor<'a>{
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

impl Serialize for MemoBody {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(None)?;

        match *self {
            MemoBody::SlabPresence(p) => {
                seq.serialize_element(&0);
                seq.serialize_element(&p);
            },
            MemoBody::Relation(m) => {
                seq.serialize_element(&1);
                for (ref slot_id, &( subject_id, mrh)) in m.iter(){
                    seq.serialize_element(slot_id);
                    seq.serialize_element(&subject_id);
                    seq.serialize_element(&mrh);
                }
            }
            MemoBody::Edit(m) => {
                seq.serialize_element(&2);
                for (ref k, ref v ) in m.iter(){
                    seq.serialize_element(k);
                    seq.serialize_element(v);
                }
            }
            MemoBody::FullyMaterialized(m) =>{
                // blehhh
            }
        }
        seq.end()

    }
}

pub struct MemoBodySeed<'a> { net: &'a Network }
struct MemoBodyVisitor<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MemoBodySeed<'a> {
    type Value = MemoBody;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(MemoBodyVisitor{ net: self.net })
    }
}

impl<'a> Visitor for MemoBodyVisitor<'a> {
    type Value = MemoBody;

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

        Ok( MemoRefHead::new_from_vec(memorefs) )
    }
}
