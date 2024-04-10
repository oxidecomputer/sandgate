/*
 * Copyright 2024 Oxide Computer Company
 */

use std::ops::Deref;
use std::result::Result as SResult;

use csnmp::ObjectValue;
use serde::de::value::U32Deserializer;
use serde::de::{DeserializeSeed, Error, SeqAccess, Unexpected};
use serde::Deserializer;

#[derive(Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Value(pub(crate) ObjectValue);

impl Deref for Value {
    type Target = ObjectValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::de::IntoDeserializer<'de> for &'de Value {
    type Deserializer = ValueDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer(&self.0)
    }
}

#[repr(transparent)]
pub struct ValueDeserializer<'a>(&'a ObjectValue);

impl ValueDeserializer<'_> {
    fn as_u64(&self) -> SResult<u64, serde::de::value::Error> {
        match &self.0 {
            ObjectValue::Integer(i) => {
                if *i < 0 {
                    Err(serde::de::value::Error::invalid_value(
                        Unexpected::Signed(*i as i64),
                        &"a u32",
                    ))
                } else {
                    Ok((*i).try_into().unwrap())
                }
            }

            ObjectValue::Counter32(u)
            | ObjectValue::Unsigned32(u)
            | ObjectValue::TimeTicks(u) => Ok((*u).into()),

            ObjectValue::Counter64(u) => Ok(*u),

            _ => Err(serde::de::value::Error::invalid_value(
                Unexpected::Other("other SNMP type"),
                &"a u64",
            )),
        }
    }

    fn as_i64(&self) -> SResult<i64, serde::de::value::Error> {
        match &self.0 {
            ObjectValue::Integer(i) => Ok((*i).into()),

            ObjectValue::Counter32(u)
            | ObjectValue::Unsigned32(u)
            | ObjectValue::TimeTicks(u) => Ok((*u).into()),

            ObjectValue::Counter64(u) => {
                let v: i64 = (*u).try_into().map_err(|_| {
                    serde::de::value::Error::invalid_value(
                        Unexpected::Unsigned(*u),
                        &"an i64",
                    )
                })?;

                Ok(v)
            }

            _ => Err(serde::de::value::Error::invalid_value(
                Unexpected::Other("other SNMP type"),
                &"an i64",
            )),
        }
    }
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.0 {
            ObjectValue::Integer(_) => self.deserialize_i32(v),
            ObjectValue::String(_) => self.deserialize_str(v),
            ObjectValue::ObjectId(_) => self.deserialize_seq(v),
            ObjectValue::Counter32(_)
            | ObjectValue::Unsigned32(_)
            | ObjectValue::TimeTicks(_) => self.deserialize_u32(v),
            ObjectValue::Counter64(_) => self.deserialize_u64(v),
            ObjectValue::IpAddress(_) | ObjectValue::Opaque(_) => {
                self.deserialize_bytes(v)
            }
        }
    }

    fn deserialize_bool<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no bool support"))
    }

    fn deserialize_i8<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(v)
    }

    fn deserialize_i16<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(v)
    }

    fn deserialize_i32<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(v)
    }

    fn deserialize_i64<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_i64(self.as_i64()?)
    }

    fn deserialize_u8<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_u16<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_u32<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_u64<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_u64(self.as_u64()?)
    }

    fn deserialize_f32<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no f32 support"))
    }

    fn deserialize_f64<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no f64 support"))
    }

    fn deserialize_char<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no char support"))
    }

    fn deserialize_str<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.0 {
            ObjectValue::String(buf) => {
                v.visit_str(std::str::from_utf8(buf).map_err(|_| {
                    serde::de::value::Error::invalid_value(
                        Unexpected::Bytes(buf),
                        &"a valid UTF-8 string",
                    )
                })?)
            }
            _ => Err(serde::de::value::Error::invalid_value(
                Unexpected::Other("other SNMP value"),
                &"a valid UTF-8 string",
            )),
        }
    }

    fn deserialize_string<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(v)
    }

    fn deserialize_bytes<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.0 {
            ObjectValue::String(buf) | ObjectValue::Opaque(buf) => {
                v.visit_bytes(buf)
            }
            _ => Err(serde::de::value::Error::invalid_value(
                Unexpected::Other("other SNMP value"),
                &"an opaque or a string",
            )),
        }
    }

    fn deserialize_byte_buf<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(v)
    }

    fn deserialize_option<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no option support"))
    }

    fn deserialize_unit<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no unit support"))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _v: V,
    ) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no unit struct support"))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _v: V,
    ) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no newtype struct support"))
    }

    fn deserialize_seq<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        struct OidSeqAccess<'a> {
            oid: &'a [u32],
            pos: usize,
        }

        impl<'de, 'a> SeqAccess<'de> for OidSeqAccess<'a> {
            type Error = serde::de::value::Error;

            fn next_element_seed<T>(
                &mut self,
                seed: T,
            ) -> SResult<Option<T::Value>, Self::Error>
            where
                T: DeserializeSeed<'de>,
            {
                if self.pos >= self.oid.len() {
                    Ok(None)
                } else {
                    let v = self.oid[self.pos];
                    self.pos += 1;
                    let de = U32Deserializer::new(v);
                    seed.deserialize(de).map(Some)
                }
            }
        }

        match self.0 {
            ObjectValue::ObjectId(oid) => {
                v.visit_seq(OidSeqAccess { oid: oid.as_slice(), pos: 0 })
            }
            _ => Err(serde::de::value::Error::invalid_value(
                Unexpected::Other("other SNMP value"),
                &"an object ID",
            )),
        }
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        _v: V,
    ) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no tuple support"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _v: V,
    ) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no tuple struct support"))
    }

    fn deserialize_map<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no map support"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _v: V,
    ) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no struct support"))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _v: V,
    ) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no enum support"))
    }

    fn deserialize_identifier<V>(self, _v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::value::Error::custom("no identifier support"))
    }

    fn deserialize_ignored_any<V>(self, v: V) -> SResult<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(v)
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ObjectValue::Integer(i) => format_args!("{}", i).fmt(f),
            ObjectValue::String(vu) => {
                format_args!("{:?}", String::from_utf8_lossy(vu)).fmt(f)
            }
            ObjectValue::ObjectId(oid) => format_args!("<oid:{oid}>").fmt(f),
            ObjectValue::IpAddress(ip) => format_args!("{}", ip).fmt(f),
            ObjectValue::Counter32(u) => format_args!("{}", u).fmt(f),
            ObjectValue::Unsigned32(u) => format_args!("{}", u).fmt(f),
            ObjectValue::TimeTicks(u) => format_args!("{}", u).fmt(f),
            ObjectValue::Opaque(buf) => format_args!("{:?}", buf).fmt(f),
            ObjectValue::Counter64(u) => format_args!("{}", u).fmt(f),
        }
    }
}
