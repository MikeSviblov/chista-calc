//! Шифры v1: AES-CBC, PKCS7, ФИКСИРОВАННЫЙ нулевой IV (детерминизм; НЕ безопасно для реального шифрования).
use super::arity;
use crate::error::{CalcError, Pos, Reason, Result};
use crate::registry::Registry;
use crate::value::Value;

use aes::{Aes128, Aes192, Aes256};
use cbc::{Decryptor, Encryptor};
use cipher::block_padding::Pkcs7;
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};

fn parse_key(key_hex: &str, pos: Pos) -> Result<Vec<u8>> {
    let key = hex::decode(key_hex)
        .map_err(|e| CalcError::RangeError { msg: Reason::BadKeyHex(e.to_string()), pos })?;
    match key.len() {
        16 | 24 | 32 => Ok(key),
        _ => Err(CalcError::RangeError { msg: Reason::KeyLength, pos }),
    }
}

fn encrypt(alg: &str, key_hex: &str, plaintext: &str, pos: Pos) -> Result<String> {
    match alg.to_lowercase().as_str() {
        "aes" | "rijndael" => {}
        _ => return Err(CalcError::RangeError { msg: Reason::UnknownCipher(alg.to_string()), pos }),
    }
    let key = parse_key(key_hex, pos)?;
    let iv = [0u8; 16];
    let data = plaintext.as_bytes();
    let ct = match key.len() {
        16 => Encryptor::<Aes128>::new(key.as_slice().into(), &iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data),
        24 => Encryptor::<Aes192>::new(key.as_slice().into(), &iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data),
        32 => Encryptor::<Aes256>::new(key.as_slice().into(), &iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data),
        _ => unreachable!("parse_key гарантирует длину 16/24/32"),
    };
    Ok(hex::encode(ct))
}

fn decrypt(alg: &str, key_hex: &str, ct_hex: &str, pos: Pos) -> Result<String> {
    match alg.to_lowercase().as_str() {
        "aes" | "rijndael" => {}
        _ => return Err(CalcError::RangeError { msg: Reason::UnknownCipher(alg.to_string()), pos }),
    }
    let key = parse_key(key_hex, pos)?;
    let ct = hex::decode(ct_hex)
        .map_err(|e| CalcError::RangeError { msg: Reason::BadCiphertextHex(e.to_string()), pos })?;
    let iv = [0u8; 16];
    let pt = match key.len() {
        16 => Decryptor::<Aes128>::new(key.as_slice().into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ct)
            .map_err(|_| CalcError::RangeError { msg: Reason::DecryptFailed, pos })?,
        24 => Decryptor::<Aes192>::new(key.as_slice().into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ct)
            .map_err(|_| CalcError::RangeError { msg: Reason::DecryptFailed, pos })?,
        32 => Decryptor::<Aes256>::new(key.as_slice().into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ct)
            .map_err(|_| CalcError::RangeError { msg: Reason::DecryptFailed, pos })?,
        _ => unreachable!("parse_key гарантирует длину 16/24/32"),
    };
    String::from_utf8(pt).map_err(|_| CalcError::RangeError { msg: Reason::DecryptNotUtf8, pos })
}

pub fn register(r: &mut Registry) {
    r.register("Encrypt", |a, pos| {
        arity(a, 3, "Encrypt", pos)?;
        let alg = a[0].as_str(pos)?;
        let key = a[1].as_str(pos)?;
        let data = a[2].as_str(pos)?;
        Ok(Value::Str(encrypt(alg, key, data, pos)?))
    });
    r.register("Decrypt", |a, pos| {
        arity(a, 3, "Decrypt", pos)?;
        let alg = a[0].as_str(pos)?;
        let key = a[1].as_str(pos)?;
        let data = a[2].as_str(pos)?;
        Ok(Value::Str(decrypt(alg, key, data, pos)?))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry; use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    fn s(x: &str) -> Value { Value::Str(x.into()) }
    #[test]
    fn aes128_roundtrip() {
        let key = s("000102030405060708090a0b0c0d0e0f"); // 16 bytes
        let enc = call("Encrypt", &[s("aes"), key.clone(), s("hello world")]);
        assert!(matches!(&enc, Value::Str(h) if !h.is_empty() && h.chars().all(|c| c.is_ascii_hexdigit())));
        let dec = call("Decrypt", &[s("aes"), key, enc]);
        assert_eq!(dec, s("hello world"));
    }
    #[test]
    fn aes256_roundtrip() {
        let key = s("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"); // 32 bytes
        let enc = call("Encrypt", &[s("aes"), key.clone(), s("The quick brown fox")]);
        let dec = call("Decrypt", &[s("aes"), key, enc]);
        assert_eq!(dec, s("The quick brown fox"));
    }
    #[test]
    fn aes192_roundtrip() {
        let key = s("000102030405060708090a0b0c0d0e0f1011121314151617"); // 24 bytes
        let enc = call("Encrypt", &[s("aes"), key.clone(), s("secret message here")]);
        let dec = call("Decrypt", &[s("aes"), key, enc]);
        assert_eq!(dec, s("secret message here"));
    }
    #[test]
    fn roundtrip_boundary_lengths() {
        let key = s("000102030405060708090a0b0c0d0e0f");
        for pt in ["", "0123456789abcdef", "0123456789abcdefg", "многобайтовый юникод 😀"] {
            let enc = call("Encrypt", &[s("aes"), key.clone(), s(pt)]);
            let dec = call("Decrypt", &[s("aes"), key.clone(), enc]);
            assert_eq!(dec, s(pt), "round-trip failed for {pt:?}");
        }
        // empty plaintext still produces exactly one 16-byte padding block (32 hex)
        if let Value::Str(h) = call("Encrypt", &[s("aes"), key, s("")]) {
            assert_eq!(h.len(), 32);
        } else { panic!() }
    }
    #[test]
    fn rijndael_alias_same_as_aes() {
        let key = s("000102030405060708090a0b0c0d0e0f");
        let enc_aes = call("Encrypt", &[s("aes"), key.clone(), s("data")]);
        let enc_rij = call("Encrypt", &[s("rijndael"), key.clone(), s("data")]);
        assert_eq!(enc_aes, enc_rij);
    }
    #[test]
    fn deterministic_known_vector() {
        // AES-128-CBC zero-IV PKCS7 of "hello world" under this key is fixed; lock it in.
        let key = s("000102030405060708090a0b0c0d0e0f");
        let enc = call("Encrypt", &[s("aes"), key, s("hello world")]);
        // locked known vector: AES-128-CBC, zero IV, PKCS7 pad of "hello world" under this key
        assert_eq!(enc, s("9276fdf384f38518fa6c8310f191678d"));
    }
    #[test]
    fn cipher_errors() {
        assert!(err("Encrypt", &[s("aes"), s("short"), s("data")]));       // bad key length
        assert!(err("Encrypt", &[s("aes"), s("zzzz"), s("data")]));        // non-hex key
        assert!(err("Encrypt", &[s("nope"), s("000102030405060708090a0b0c0d0e0f"), s("data")])); // unknown cipher
        assert!(err("Decrypt", &[s("aes"), s("000102030405060708090a0b0c0d0e0f"), s("zz")]));    // non-hex/invalid ct
        assert!(err("Decrypt", &[s("aes"), s("000102030405060708090a0b0c0d0e0f"), s("00")]));    // ct not block-aligned / bad padding
    }
}
