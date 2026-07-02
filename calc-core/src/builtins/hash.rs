use super::arity;
use crate::error::{CalcError, Pos, Result};
use crate::registry::Registry;
use crate::value::Value;

fn digest_hex<D: digest::Digest>(data: &[u8]) -> String {
    let mut h = D::new();
    h.update(data);
    hex::encode(h.finalize())
}

fn hash_bytes(alg: &str, data: &[u8], pos: Pos) -> Result<String> {
    let out = match alg.to_lowercase().as_str() {
        "md5" => digest_hex::<md5::Md5>(data),
        "sha1" => digest_hex::<sha1::Sha1>(data),
        "sha224" => digest_hex::<sha2::Sha224>(data),
        "sha256" => digest_hex::<sha2::Sha256>(data),
        "sha384" => digest_hex::<sha2::Sha384>(data),
        "sha512" => digest_hex::<sha2::Sha512>(data),
        "sha3_256" => digest_hex::<sha3::Sha3_256>(data),
        "sha3_512" => digest_hex::<sha3::Sha3_512>(data),
        "ripemd160" => digest_hex::<ripemd::Ripemd160>(data),
        "tiger" => digest_hex::<tiger::Tiger>(data),
        "crc32" => format!("{:08x}", crc32fast::hash(data)),
        "adler32" => format!("{:08x}", adler::adler32_slice(data)),
        _ => return Err(CalcError::RangeError { msg: format!("неизвестный алгоритм хеша '{alg}'"), pos }),
    };
    Ok(out)
}

pub fn register(r: &mut Registry) {
    r.register("Hash", |a, pos| {
        arity(a, 2, "Hash", pos)?;
        let alg = a[0].as_str(pos)?;
        let data = a[1].as_str(pos)?;
        Ok(Value::Str(hash_bytes(alg, data.as_bytes(), pos)?))
    });
    r.register("Md5", |a, pos| {
        arity(a, 1, "Md5", pos)?;
        Ok(Value::Str(hash_bytes("md5", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha1", |a, pos| {
        arity(a, 1, "Sha1", pos)?;
        Ok(Value::Str(hash_bytes("sha1", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha224", |a, pos| {
        arity(a, 1, "Sha224", pos)?;
        Ok(Value::Str(hash_bytes("sha224", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha256", |a, pos| {
        arity(a, 1, "Sha256", pos)?;
        Ok(Value::Str(hash_bytes("sha256", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha384", |a, pos| {
        arity(a, 1, "Sha384", pos)?;
        Ok(Value::Str(hash_bytes("sha384", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha512", |a, pos| {
        arity(a, 1, "Sha512", pos)?;
        Ok(Value::Str(hash_bytes("sha512", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha3_256", |a, pos| {
        arity(a, 1, "Sha3_256", pos)?;
        Ok(Value::Str(hash_bytes("sha3_256", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Sha3_512", |a, pos| {
        arity(a, 1, "Sha3_512", pos)?;
        Ok(Value::Str(hash_bytes("sha3_512", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("RipeMD160", |a, pos| {
        arity(a, 1, "RipeMD160", pos)?;
        Ok(Value::Str(hash_bytes("ripemd160", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Tiger", |a, pos| {
        arity(a, 1, "Tiger", pos)?;
        Ok(Value::Str(hash_bytes("tiger", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Crc32", |a, pos| {
        arity(a, 1, "Crc32", pos)?;
        Ok(Value::Str(hash_bytes("crc32", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
    r.register("Adler32", |a, pos| {
        arity(a, 1, "Adler32", pos)?;
        Ok(Value::Str(hash_bytes("adler32", a[0].as_str(pos)?.as_bytes(), pos)?))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry; use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    fn s(x: &str) -> Value { Value::Str(x.into()) }
    #[test]
    fn known_hashes() {
        assert_eq!(call("Md5", &[s("")]), s("d41d8cd98f00b204e9800998ecf8427e"));
        assert_eq!(call("Md5", &[s("abc")]), s("900150983cd24fb0d6963f7d28e17f72"));
        assert_eq!(call("Sha1", &[s("abc")]), s("a9993e364706816aba3e25717850c26c9cd0d89d"));
        assert_eq!(call("Sha256", &[s("abc")]), s("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        assert_eq!(call("Sha512", &[s("abc")]), s("ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"));
        assert_eq!(call("RipeMD160", &[s("abc")]), s("8eb208f7e05d987a9b044a8e98c6b087f15a0bfc"));
        assert_eq!(call("Crc32", &[s("abc")]), s("352441c2"));
        assert_eq!(call("Hash", &[s("sha256"), s("abc")]), s("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        assert_eq!(call("Hash", &[s("md5"), s("abc")]), s("900150983cd24fb0d6963f7d28e17f72"));
    }
    #[test]
    fn hash_errors() {
        assert!(err("Hash", &[s("nope"), s("x")]));
        assert!(err("Hash", &[s("sha256")]));
        assert!(err("Md5", &[Value::Int(5)]));
    }
}
