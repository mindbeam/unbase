use super::*;
use serde::*;
use serde::ser::*;
use serde::de::*;
use memoref::*;
use network::slabref::serde::*;

impl Serialize for MemoRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let shared = &self.shared.lock().unwrap();
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.id)?;
        seq.serialize_element(&self.subject_id)?;
        match &shared.ptr {
            &MemoRefPtr::Remote      => seq.serialize_element(&false),
            &MemoRefPtr::Resident(_) => seq.serialize_element(&true),
        };
        seq.serialize_element(&shared.peers)?;
        seq.end()
    }
}

pub struct MemoRefSeed<'a> { net: &'a Network }
struct MemoRefVisitor<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MemoRefSeed<'a> {
    type Value = MemoRef;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(MemoRefVisitor{ net: self.net })
    }
}

impl<'a> Visitor for MemoRefVisitor<'a> {
    type Value = MemoRef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct MemoRef")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<MemoRef, V::Error>
       where V: SeqVisitor
    {
        let memo_id: MemoId = match visitor.visit()? {
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
        let has_memo: bool = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(2, &self));
           }
        };
        let peers: Vec<MemoPeer> = match visitor.visit_seed( MemoPeerVecSeed{ net: self.net })? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(3, &self));
           }
        };

       let memoref = MemoRef {
           id: memo_id,
           subject_id: Some(subject_id),
           shared: Arc::new(Mutex::new(
               MemoRefShared {
                   peers: peers,
                   ptr: match has_memo {
                       true  => MemoRefPtr::Remote,
                       false => MemoRefPtr::Remote
                   }
               }
           ))
       };

       Ok(memoref)
    }
}

pub struct MemoPeerSeed<'a> { net: &'a Network }
struct MemoPeerVisitor<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MemoPeerSeed<'a> {
    type Value = MemoPeer;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(MemoPeerVisitor{ net: self.net })
    }
}

impl<'a> Visitor for MemoPeerVisitor<'a> {
    type Value = MemoPeer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct MemoRef")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {
        let slabref: SlabRef = match visitor.visit_seed( SlabRefSeed{ net: self.net })? {
            Some(value) => value,
            None => {
                return Err(de::Error::invalid_length(0, &self));
            }
        };
        let status: PeeringStatus = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(de::Error::invalid_length(1, &self));
           }
        };

       Ok(MemoPeer{
           slabref: slabref,
           status: status
       })
    }
}

pub struct MemoPeerVecSeed<'a> { net: &'a Network }
struct MemoPeerVecVisitor<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MemoPeerVecSeed<'a> {
    type Value = Vec<MemoPeer>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(MemoPeerVecVisitor{ net: self.net })
    }
}

impl<'a> Visitor for MemoPeerVecVisitor<'a> {
    type Value = Vec<MemoPeer>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("Sequence of MemoPeers")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {

        let mut memopeers : Vec<MemoPeer> = Vec::new();

        while let Some(memopeer) = visitor.visit_seed( MemoPeerSeed{ net: self.net })? {
            memopeers.push(memopeer);
        };

        Ok(memopeers)
    }
}
