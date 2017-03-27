use std::fmt;
use serde::de::*;

pub struct VecSeed<S>(pub S);

impl<S> DeserializeSeed for VecSeed<S>
    where S: DeserializeSeed + Clone
{
    type Value = Vec<S::Value>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<S> Visitor for VecSeed<S>
    where S: DeserializeSeed + Clone
{
    type Value = Vec<S::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("sequence")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {

        let mut out : Vec<S::Value> = Vec::new();

        while let Some(v) = visitor.visit_seed( self.0.clone() )? {
            out.push(v);
        };

        Ok(out)
    }
}
/// optional one.
pub struct OptionSeed<S>(pub S);

impl<S> Visitor for OptionSeed<S>
    where S: DeserializeSeed
{
    type Value = Option<S::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        self.0.deserialize(deserializer).map(Some)
    }
}

impl<S> DeserializeSeed for OptionSeed<S>
    where S: DeserializeSeed
{
    type Value = Option<S::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_option(self)
    }
}
