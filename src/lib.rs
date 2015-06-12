
extern crate rand;
extern crate time;
extern crate rustc_serialize;

use std::mem;
use std::ptr;
use std::fmt;
use rand::{OsRng,Rng};
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
#[derive(Clone,Debug)]
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

    /// Generate a new `TextNonce` specifying the Base64 configuration to use.
    /// `length` must be at least 16, and divisible by 4.  The first 16 characters come
    /// from the time component, and all characters after that will be random.
    pub fn sized_configured(length: usize, config: base64::Config) -> Result<TextNonce,String> {
        if length<16 { return Err("length must be >= 16".to_string()); }
        if length % 4 != 0 { return Err("length must be divisible by 4".to_string()); }

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

#[cfg(test)]
mod tests {
    use super::TextNonce;
    use std::collections::HashSet;

    #[test]
    fn new() {
        // Test 100 nonces:
        let mut map = HashSet::new();
        for _ in (0..100) {
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
}
