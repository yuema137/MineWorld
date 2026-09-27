//! A serializer and a deserializer that report `is_human_readable() == false`, and record what
//! they were handed.
//!
//! The contract layer's opaque identities encode themselves as decimal strings for a
//! human-readable format and as a `u64` for every other one (`contracts/src/ids.rs`). No binary
//! `serde` format is a dependency of this workspace, and the property under test is not "bincode
//! produces these bytes" — it is exactly the discriminator: *given a format that says it is not
//! human-readable, does an id go out as a number and come back from one?* So this module is the
//! smallest honest instrument for that question rather than a simulation of a real codec.
//!
//! [`BinaryDeserializer`] additionally refuses `deserialize_any`, which is what makes it
//! discriminative rather than merely permissive: a real binary format is not self-describing, so a
//! `Deserialize` implementation that forgot to branch on `is_human_readable()` and reached for
//! `deserialize_any` would fail here exactly as it would against `bincode` or `postcard`.

use std::fmt;

use serde::de::{DeserializeOwned, Visitor};
use serde::ser::Impossible;
use serde::{Deserializer, Serialize, Serializer};

/// What a value asked the serializer to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Written {
    /// `serialize_u64` — the branch an id must take when the format is not human-readable.
    Unsigned(u64),
    /// `serialize_str` — the branch an id must take only when it is.
    Text(String),
}

/// What went wrong, as `serde` requires an error type to be able to say.
#[derive(Debug)]
pub struct ProbeError(String);

impl fmt::Display for ProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ProbeError {}

impl serde::ser::Error for ProbeError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

impl serde::de::Error for ProbeError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

/// Serializes one scalar and reports which method it was asked for, declaring itself not
/// human-readable.
///
/// Every method a scalar identity would never use refuses, so a change that made an id serialize
/// as something else entirely — a struct, a sequence, a byte string — fails loudly here instead of
/// being recorded as an unexamined success.
pub struct BinarySerializer;

/// The one method this instrument exists for: what did `value` write on a non-human-readable
/// format?
pub fn write_binary<T: Serialize>(value: &T) -> Result<Written, ProbeError> {
    value.serialize(BinarySerializer)
}

macro_rules! refuse {
    ($method:ident, $type:ty) => {
        fn $method(self, _value: $type) -> Result<Written, ProbeError> {
            Err(ProbeError(format!(
                "an opaque identity must not serialize through {}",
                stringify!($method)
            )))
        }
    };
    ($method:ident) => {
        fn $method(self) -> Result<Written, ProbeError> {
            Err(ProbeError(format!(
                "an opaque identity must not serialize through {}",
                stringify!($method)
            )))
        }
    };
}

impl Serializer for BinarySerializer {
    type Ok = Written;
    type Error = ProbeError;
    type SerializeSeq = Impossible<Written, ProbeError>;
    type SerializeTuple = Impossible<Written, ProbeError>;
    type SerializeTupleStruct = Impossible<Written, ProbeError>;
    type SerializeTupleVariant = Impossible<Written, ProbeError>;
    type SerializeMap = Impossible<Written, ProbeError>;
    type SerializeStruct = Impossible<Written, ProbeError>;
    type SerializeStructVariant = Impossible<Written, ProbeError>;

    /// The whole point of the instrument.
    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_u64(self, value: u64) -> Result<Written, ProbeError> {
        Ok(Written::Unsigned(value))
    }

    fn serialize_str(self, value: &str) -> Result<Written, ProbeError> {
        Ok(Written::Text(value.to_owned()))
    }

    refuse!(serialize_bool, bool);
    refuse!(serialize_i8, i8);
    refuse!(serialize_i16, i16);
    refuse!(serialize_i32, i32);
    refuse!(serialize_i64, i64);
    refuse!(serialize_u8, u8);
    refuse!(serialize_u16, u16);
    refuse!(serialize_u32, u32);
    refuse!(serialize_f32, f32);
    refuse!(serialize_f64, f64);
    refuse!(serialize_char, char);
    refuse!(serialize_bytes, &[u8]);
    refuse!(serialize_none);
    refuse!(serialize_unit);

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Written, ProbeError> {
        Err(ProbeError("an opaque identity is not an option".to_owned()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Written, ProbeError> {
        Err(ProbeError(
            "an opaque identity is not a unit struct".to_owned(),
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<Written, ProbeError> {
        Err(ProbeError("an opaque identity is not an enum".to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Written, ProbeError> {
        Err(ProbeError(
            "an opaque identity must write its scalar directly rather than as a newtype: a \
             transparent newtype is what let the rule be bypassed"
                .to_owned(),
        ))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Written, ProbeError> {
        Err(ProbeError("an opaque identity is not an enum".to_owned()))
    }

    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, ProbeError> {
        Err(ProbeError(
            "an opaque identity is not a sequence".to_owned(),
        ))
    }

    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple, ProbeError> {
        Err(ProbeError("an opaque identity is not a tuple".to_owned()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct, ProbeError> {
        Err(ProbeError("an opaque identity is not a tuple".to_owned()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, ProbeError> {
        Err(ProbeError("an opaque identity is not an enum".to_owned()))
    }

    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, ProbeError> {
        Err(ProbeError("an opaque identity is not a map".to_owned()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, ProbeError> {
        Err(ProbeError("an opaque identity is not a struct".to_owned()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, ProbeError> {
        Err(ProbeError("an opaque identity is not an enum".to_owned()))
    }
}

/// Hands one `u64` back, declaring itself not human-readable, and refusing to be self-describing.
pub struct BinaryDeserializer(pub u64);

/// Reads `raw` back as `T` through a format that is not human-readable and not self-describing.
pub fn read_binary<T: DeserializeOwned>(raw: u64) -> Result<T, ProbeError> {
    T::deserialize(BinaryDeserializer(raw))
}

impl<'de> Deserializer<'de> for BinaryDeserializer {
    type Error = ProbeError;

    /// The whole point of the instrument.
    fn is_human_readable(&self) -> bool {
        false
    }

    /// A binary format is not self-describing, so this is the failure a real one would produce.
    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ProbeError> {
        Err(ProbeError(
            "a format that is not human-readable cannot answer deserialize_any: the type must ask \
             for the shape it wrote"
                .to_owned(),
        ))
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ProbeError> {
        visitor.visit_u64(self.0)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u128 f32 f64 char str string bytes byte_buf option
        unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
    }
}
