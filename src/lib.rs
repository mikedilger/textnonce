// Copyright 2015-2016 textnonce Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate rand;
extern crate time;
extern crate rustc_serialize;
#[cfg(feature = "serde")]
extern crate serde;

use std::mem;
use std::ptr;
use std::fmt;
use rand::{OsRng,Rng};
use std::ops::Deref;
use rustc_serialize::base64::{self,ToBase64};

/// A nonce is a cryptographic concept of an arbitrary number that is never used more than once.
///
/// `TextNonce` is a nonce because the first 16 characters represents the current time, which
/// will never have been generated before, nor will it be generated again, across the period of
/// time in which Timespec is valid.
///
/// `TextNonce` additionally includes bytes of randomness, making it difficult to predict.
/// This makes it suitable to be used for session IDs.
///
/// It is also text-based, using only characters in the base64 character set.
#[derive(Clone, PartialEq, Debug)]
pub struct TextNonce(pub String);

impl TextNonce {

    /// Generate a new `TextNonce` with 16 characters of time and 16 characters of
    /// randomness
    pub fn new() -> TextNonce {
        TextNonce::sized(32).unwrap()
    }

    /// Generate a new `TextNonce`. `length` must be at least 16, and divisible by 4.
    /// The first 16 characters come from the time component, and all characters
    /// after that will be random.
    pub fn sized(length: usize) -> Result<TextNonce,String> {
        TextNonce::sized_configured(length, base64::Config {
            char_set: base64::CharacterSet::Standard,
            newline: base64::Newline::LF,
            pad: false,
            line_length: None
        })
    }

    /// Generate a new `TextNonce` using the UrlSafe variant of base64 (using '_' and '-')
    /// `length` must be at least 16, and divisible by 4.  The first 16 characters come
    /// from the time component, and all characters after that will be random.
    pub fn sized_urlsafe(length: usize) ->  Result<TextNonce,String> {
        TextNonce::sized_configured(length, base64::Config {
            char_set: base64::CharacterSet::UrlSafe,
            newline: base64::Newline::LF,
            pad: false,
            line_length: None
        })
    }

    /// Generate a new `TextNonce` specifying the Base64 configuration to use.
    /// `length` must be at least 16, and divisible by 4.  The first 16 characters come
    /// from the time component, and all characters after that will be random.
    pub fn sized_configured(length: usize, config: base64::Config) -> Result<TextNonce,String> {
        if length<16 { return Err("length must be >= 16".to_owned()); }
        if length % 4 != 0 { return Err("length must be divisible by 4".to_owned()); }

        let bytelength: usize = (length / 4) * 3;

        let mut raw: Vec<u8> = Vec::with_capacity(bytelength);
        unsafe { raw.set_len(bytelength); }

        // Get the first 12 bytes from the current time
        // (Timespec is actually 16 due to alignment, but we will only use 12)
        // Big-endian and Little-endian machines will have these in different orders,
        // However this is not of any concern to us.
        let time = ::time::get_time();
        unsafe {
            let timep: *const u8 = mem::transmute(&time);
            ptr::copy_nonoverlapping(timep, raw.as_mut_ptr(), 12);
        }

        // Get the last bytes from random data
        match OsRng::new() {
            Ok(mut g) => g.fill_bytes(&mut raw[12..bytelength]),
            Err(_) => ::rand::thread_rng().fill_bytes(&mut raw[12..bytelength])
        };

        // base64 encode
        Ok(TextNonce(raw.to_base64( config )))
    }

    pub fn into_string(self) -> String {
        let TextNonce(s) = self;
        s
    }
}

impl fmt::Display for TextNonce {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for TextNonce {
    type Target = str;
    fn deref<'a>(&'a self) -> &'a str {
        &*self.0
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for TextNonce {
    fn serialize<S>(&self, serializer: &mut S)  -> Result<(), S::Error>
        where S: serde::ser::Serializer
    {
        serializer.serialize_newtype_struct("TextNonce", &self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::de::Deserialize for TextNonce {
    fn deserialize<D>(deserializer: &mut D) -> Result<TextNonce, D::Error>
        where D: serde::de::Deserializer
    {
        struct Visitor<D: serde::de::Deserializer>(::std::marker::PhantomData<D>);

        impl <D: serde::de::Deserializer> serde::de::Visitor for Visitor<D> {
            type Value = TextNonce;

            #[inline]
            fn visit_newtype_struct<E>(&mut self, e: &mut E) -> Result<Self::Value, E::Error>
                where E: serde::de::Deserializer
            {
                Ok(TextNonce(try!(<String as serde::Deserialize>::deserialize(e))))
            }

            #[inline]
            fn visit_seq<V>(&mut self, mut visitor: V) -> Result<TextNonce, V::Error>
                where V: serde::de::SeqVisitor
            {
                let field0 =  match try!(visitor.visit::<String>()) {
                    Some(value) => { value },
                    None => return Err(serde::de::Error::end_of_stream()),
                };
                try!(visitor.end());
                Ok(TextNonce(field0))
            }
        }

        deserializer.deserialize_newtype_struct("TextNonce",
                                                Visitor::<D>(::std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    extern crate bincode;

    use super::TextNonce;
    use std::collections::HashSet;

    #[test]
    fn new() {
        // Test 100 nonces:
        let mut map = HashSet::new();
        for _ in 0..100 {
            let n = TextNonce::new();
            let TextNonce(s) = n;

            // Verify their length
            assert_eq!(s.len(), 32);

            // Verify their character content
            assert_eq!( s.chars()
                        .filter(|x| { x.is_digit(10) || x.is_alphabetic() || *x=='+' || *x=='/' })
                        .count(),
                        32 );

            // Add to the map
            map.insert(s);
        }
        assert_eq!( map.len(), 100 );
    }

    #[test]
    fn sized() {
        let n = TextNonce::sized(48);
        assert!( n.is_ok() );
        let TextNonce(s) = n.unwrap();
        assert_eq!(s.len(), 48);

        let n = TextNonce::sized(47);
        assert!( n.is_err() );
        let n = TextNonce::sized(12);
        assert!( n.is_err() );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() {
        let n = TextNonce::sized(48);
        let serialized = bincode::serde::serialize(&n, bincode::SizeLimit::Infinite)
            .unwrap();
        let deserialized = bincode::serde::deserialize(&serialized).unwrap();
        assert_eq!(n, deserialized);
    }
}
