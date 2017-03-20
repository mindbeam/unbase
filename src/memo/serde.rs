
use serde::*;
use serde::ser::*;
use serde::de::*;
use super::*;
use memo::*;
use std::fmt;

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
impl Deserialize for Memo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct MemoVisitor;
        impl Visitor for MemoVisitor {
            type Value = Memo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
               formatter.write_str("struct Memo")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Memo, V::Error>
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
               let parents: MemoRefHead = match visitor.visit()? {
                   Some(value) => value,
                   None => {
                       return Err(de::Error::invalid_length(2, &self));
                   }
               };
               let body: MemoBody = match visitor.visit()? {
                   Some(value) => value,
                   None => {
                       return Err(de::Error::invalid_length(3, &self));
                   }
               };

               Ok(Memo::new(id, subject_id, parents, body))
            }
        }


        const FIELDS: &'static [&'static str] = &["id", "subject_id","parents","body"];
        deserializer.deserialize_struct("Memo", FIELDS, MemoVisitor)

    }
}
