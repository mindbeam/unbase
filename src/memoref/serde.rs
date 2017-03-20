use super::*;
use serde::*;
use serde::ser::*;
use serde::de::*;
use memoref::*;

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

impl Deserialize for MemoRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct MemoRefVisitor;
        impl Visitor for MemoRefVisitor {
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
               let ptr: bool = match visitor.visit()? {
                   Some(value) => value,
                   None => {
                       return Err(de::Error::invalid_length(2, &self));
                   }
               };
               let peers: Vec<MemoPeer> = match visitor.visit()? {
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
                           ptr: match ptr {
                               true  => MemoRefPtr::Remote,
                               false => MemoRefPtr::Remote
                           }
                       }
                   ))
               };

               Ok(memoref)
            }
        }


        const FIELDS: &'static [&'static str] = &["id", "subject_id","ptr","peers"];
        deserializer.deserialize_struct("MemoRef", FIELDS, MemoRefVisitor)

    }
}
