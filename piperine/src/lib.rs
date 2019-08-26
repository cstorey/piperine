use std::fmt;
use std::iter::{IntoIterator, Peekable};

use serde::{de, forward_to_deserialize_any};

pub const START: &str = "__start__";
pub const END: &str = "__end__";
pub const MAPPING: &str = "sequence";
pub const SEQUENCE: &str = "mapping";

#[derive(Debug)]
pub enum Error {
    Argh,
}

pub struct Parser<I: Iterator> {
    input: Peekable<I>,
}

pub fn from_pairs<'de, T, S: 'de, I>(input: S) -> Result<T, de::value::Error>
where
    S: IntoIterator<Item = (String, String), IntoIter = I>,
    I: 'de + Iterator<Item = (String, String)>,
    T: de::Deserialize<'de>,
{
    let mut parser = Parser::from_pairs(input.into_iter());
    let t = de::Deserialize::deserialize(&mut parser)?;

    Ok(t)
}

impl<'de, I: 'de + Iterator> Parser<I> {
    fn from_pairs(input: I) -> Self {
        let input = input.peekable();
        Parser { input }
    }
}

impl<'de, 'a, I: Iterator + 'de> de::Deserializer<'de> for &'a mut Parser<I>
where
    I: Iterator<Item = (String, String)>,
{
    type Error = de::value::Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let pair = self.input.peek();
        eprintln!("Parser::deserialize_any: {:?}", pair);

        unimplemented!("Parser::deserialize_any")
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let pair = self.input.peek();
        eprintln!("Parser::deserialize_unit: {:?}", pair);

        visitor.visit_unit()
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        eprintln!("deserialize_struct: name:{:?}; fields:{:?}", _name, _fields);
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        eprintln!("→ deserialize_map");
        let a: MapAccess<'_, I> = MapAccess::new(self);
        visitor.visit_map(a)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        eprintln!("→ deserialize_identifier");
        self.deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        eprintln!("→ deserialize_identifier");
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (name, _value) = self.input.peek().ok_or_else(|| unimplemented!("EOF"))?;
        eprintln!("→ Parser::deserialize_str: {:?}", (name, _value));

        visitor.visit_str(name)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf option unit_struct newtype_struct seq tuple
        tuple_struct enum ignored_any
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Argh => write!(fmt, "Argh!")?,
        };

        Ok(())
    }
}

struct MapAccess<'a, I: Iterator> {
    de: &'a mut Parser<I>,
}

impl<'a, I: Iterator> MapAccess<'a, I> {
    fn new(de: &'a mut Parser<I>) -> Self {
        MapAccess { de }
    }
}

impl<'de, 'a, I: 'de> de::MapAccess<'de> for MapAccess<'a, I>
where
    I: Iterator<Item = (String, String)>,
{
    type Error = de::value::Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // I'm fairly sure I'll have to come back to this decision.
        if let Some((name, value)) = self.de.input.peek() {
            eprintln!("Access as MapAccess::next_key_seed: {:?}", (name, value));

            // What I really want to do here is just be able to pull out the name
            // as an atom, make a deserializer for that alone.
            Ok(Some(seed.deserialize(&mut ParseScalar(name.clone()))?))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // I'm fairly sure I'll have to come back to this decision.
        if let Some((name, value)) = self.de.input.next() {
            eprintln!(
                "Access as MapAccess::next_value_seed: {:?}",
                (&name, &value)
            );

            // What I really want to do here is just be able to pull out the name
            // as an atom, make a deserializer for that alone.
            Ok(seed.deserialize(&mut ParseScalar(value.clone()))?)
        } else {
            Err(unimplemented!())
        }
    }
}

struct ParseScalar(String);

impl<'de, 'a> de::Deserializer<'de> for &'a mut ParseScalar {
    type Error = de::value::Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        eprintln!("→ ParseScalar::deserialize_any: {:?}", self.0);
        visitor.visit_str(&self.0)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf option unit_struct newtype_struct seq tuple struct
        tuple_struct enum ignored_any str string identifier unit map
    }
}
