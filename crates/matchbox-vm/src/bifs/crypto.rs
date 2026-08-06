#[cfg(feature = "bif-crypto")]
use crate::types::{BxVM, BxValue};
#[cfg(feature = "bif-crypto")]
use digest::Digest;
#[cfg(feature = "bif-crypto")]
use hmac::{Hmac, Mac};
#[cfg(feature = "bif-crypto")]
use md5::Md5;
#[cfg(feature = "bif-crypto")]
use rand::RngExt;
#[cfg(feature = "bif-crypto")]
use sha1::Sha1;
#[cfg(feature = "bif-crypto")]
use sha2::{Sha224, Sha256, Sha384, Sha512};
#[cfg(feature = "bif-crypto")]
use std::fs;
#[cfg(feature = "bif-crypto")]
use std::path::Path;

#[cfg(feature = "bif-crypto")]
const DEFAULT_HASH_ALGORITHM: &str = "MD5";
#[cfg(feature = "bif-crypto")]
const DEFAULT_HASH_ENCODING: &str = "utf-8";
#[cfg(feature = "bif-crypto")]
const DEFAULT_HMAC_ALGORITHM: &str = "HmacMD5";

#[cfg(feature = "bif-crypto")]
enum HashAlgorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Quick,
}

#[cfg(feature = "bif-crypto")]
enum HmacAlgorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

#[cfg(feature = "bif-crypto")]
enum HashSource {
    Text(String),
    Bytes(Vec<u8>),
}

#[cfg(feature = "bif-crypto")]
pub fn hash_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("hash() expects at least 1 argument".to_string());
    }

    let algorithm = args
        .get(1)
        .map(|value| vm.to_string(*value))
        .unwrap_or_else(|| DEFAULT_HASH_ALGORITHM.to_string());
    let encoding = args
        .get(2)
        .map(|value| vm.to_string(*value))
        .unwrap_or_else(|| DEFAULT_HASH_ENCODING.to_string());
    let iterations = args
        .get(3)
        .map(|value| value.as_number().max(1.0) as usize)
        .unwrap_or(1)
        .max(1);

    let algorithm = parse_hash_algorithm(&algorithm)?;
    let source = resolve_hash_source(vm, args[0])?;

    let result = match algorithm {
        HashAlgorithm::Quick => {
            let mut text = match source {
                HashSource::Text(text) => text,
                HashSource::Bytes(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            };

            for _ in 0..iterations {
                text = quick_hash(&text);
            }

            text
        }
        _ => {
            let mut input = match source {
                HashSource::Text(text) => encode_text(&text, &encoding)?,
                HashSource::Bytes(bytes) => bytes,
            };

            for _ in 0..iterations {
                input = match algorithm {
                    HashAlgorithm::Md5 => md5_digest(&input).to_vec(),
                    HashAlgorithm::Sha1 => sha1_digest(&input).to_vec(),
                    HashAlgorithm::Sha224 => sha224_digest(&input).to_vec(),
                    HashAlgorithm::Sha256 => sha256_digest(&input).to_vec(),
                    HashAlgorithm::Sha384 => sha384_digest(&input).to_vec(),
                    HashAlgorithm::Sha512 => sha512_digest(&input).to_vec(),
                    HashAlgorithm::Quick => unreachable!(),
                };
            }

            hex_encode(&input)
        }
    };

    Ok(BxValue::new_ptr(vm.string_new(result)))
}

#[cfg(feature = "bif-crypto")]
pub fn hash40_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return hash_bif(vm, args);
    }

    let mut hash_args = Vec::with_capacity(args.len() + 1);
    hash_args.push(args[0]);
    hash_args.push(BxValue::new_ptr(vm.string_new("sha1".to_string())));
    hash_args.extend_from_slice(args.get(2..).unwrap_or_default());
    hash_bif(vm, &hash_args)
}

#[cfg(feature = "bif-crypto")]
pub fn hmac_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("hmac() expects at least 2 arguments".to_string());
    }

    let algorithm = args
        .get(2)
        .map(|value| vm.to_string(*value))
        .unwrap_or_else(|| DEFAULT_HMAC_ALGORITHM.to_string());
    let encoding = args
        .get(3)
        .map(|value| vm.to_string(*value))
        .unwrap_or_else(|| DEFAULT_HASH_ENCODING.to_string());

    let algorithm = parse_hmac_algorithm(&algorithm)?;
    let message = resolve_message_bytes(vm, args[0], &encoding)?;
    let key = resolve_key_bytes(vm, args[1], &encoding)?;

    let result = match algorithm {
        HmacAlgorithm::Md5 => hmac_md5(&key, &message)?,
        HmacAlgorithm::Sha1 => hmac_sha1(&key, &message)?,
        HmacAlgorithm::Sha224 => hmac_sha224(&key, &message)?,
        HmacAlgorithm::Sha256 => hmac_sha256(&key, &message)?,
        HmacAlgorithm::Sha384 => hmac_sha384(&key, &message)?,
        HmacAlgorithm::Sha512 => hmac_sha512(&key, &message)?,
    };

    Ok(BxValue::new_ptr(vm.string_new(hex_encode(&result))))
}

#[cfg(feature = "bif-crypto")]
fn parse_hash_algorithm(name: &str) -> Result<HashAlgorithm, String> {
    match normalize_algorithm_name(name).as_str() {
        "" | "md5" | "bxmxcompat" => Ok(HashAlgorithm::Md5),
        "sha" | "sha1" => Ok(HashAlgorithm::Sha1),
        "sha224" => Ok(HashAlgorithm::Sha224),
        "sha256" => Ok(HashAlgorithm::Sha256),
        "sha384" => Ok(HashAlgorithm::Sha384),
        "sha512" => Ok(HashAlgorithm::Sha512),
        "quick" | "quick64" => Ok(HashAlgorithm::Quick),
        other => Err(format!(
            "Unsupported hash algorithm: {}. Supported algorithms are MD5, SHA-1, SHA-224, SHA-256, SHA-384, SHA-512, QUICK, and bxmX_COMPAT.",
            other
        )),
    }
}

#[cfg(feature = "bif-crypto")]
fn parse_hmac_algorithm(name: &str) -> Result<HmacAlgorithm, String> {
    match normalize_algorithm_name(name).as_str() {
        "" | "hmacmd5" => Ok(HmacAlgorithm::Md5),
        "hmacsha1" => Ok(HmacAlgorithm::Sha1),
        "hmacsha224" => Ok(HmacAlgorithm::Sha224),
        "hmacsha256" => Ok(HmacAlgorithm::Sha256),
        "hmacsha384" => Ok(HmacAlgorithm::Sha384),
        "hmacsha512" => Ok(HmacAlgorithm::Sha512),
        other => Err(format!(
            "Unsupported hmac algorithm: {}. Supported algorithms are HmacMD5, HmacSHA1, HmacSHA224, HmacSHA256, HmacSHA384, and HmacSHA512.",
            other
        )),
    }
}

#[cfg(feature = "bif-crypto")]
fn normalize_algorithm_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(feature = "bif-crypto")]
fn resolve_hash_source(vm: &mut dyn BxVM, value: BxValue) -> Result<HashSource, String> {
    if vm.is_string_value(value) {
        let text = vm.to_string(value);
        let path = Path::new(&text);
        if path.exists() && path.is_file() {
            return fs::read(path)
                .map(HashSource::Bytes)
                .map_err(|e| format!("Failed to read file for hashing: {}", e));
        }
        return Ok(HashSource::Text(text));
    }

    if let Ok(bytes) = vm.to_bytes(value) {
        return Ok(HashSource::Bytes(bytes));
    }

    Ok(HashSource::Text(vm.to_string(value)))
}

#[cfg(feature = "bif-crypto")]
fn resolve_message_bytes(vm: &mut dyn BxVM, value: BxValue, encoding: &str) -> Result<Vec<u8>, String> {
    if let Ok(bytes) = vm.to_bytes(value) {
        return Ok(bytes);
    }

    encode_text(&vm.to_string(value), encoding)
}

#[cfg(feature = "bif-crypto")]
fn resolve_key_bytes(vm: &mut dyn BxVM, value: BxValue, encoding: &str) -> Result<Vec<u8>, String> {
    resolve_message_bytes(vm, value, encoding)
}

#[cfg(feature = "bif-crypto")]
fn encode_text(text: &str, encoding: &str) -> Result<Vec<u8>, String> {
    match normalize_encoding_name(encoding).as_str() {
        "utf8" => Ok(text.as_bytes().to_vec()),
        "utf16" => {
            let mut bytes = Vec::with_capacity(2 + text.len() * 2);
            bytes.extend_from_slice(&[0xFE, 0xFF]);
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&unit.to_be_bytes());
            }
            Ok(bytes)
        }
        "utf16le" => {
            let mut bytes = Vec::with_capacity(text.len() * 2);
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            Ok(bytes)
        }
        "utf16be" => {
            let mut bytes = Vec::with_capacity(text.len() * 2);
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&unit.to_be_bytes());
            }
            Ok(bytes)
        }
        "ascii" | "usascii" => {
            if text.is_ascii() {
                Ok(text.as_bytes().to_vec())
            } else {
                Err(format!("Unsupported non-ASCII input for encoding {}", encoding))
            }
        }
        other => Err(format!("Unsupported encoding: {}", other)),
    }
}

#[cfg(feature = "bif-crypto")]
fn normalize_encoding_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(feature = "bif-crypto")]
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

#[cfg(feature = "bif-crypto")]
fn quick_hash(input: &str) -> String {
    const HSTART: u64 = 0xBB40E64DA205B064;
    const HMULT: u64 = 7664345821815920749;
    let byte_table = generate_hash_lookup_table();

    let mut hash = HSTART;
    for ch in input.chars() {
        let code = ch as u32;
        hash = (hash.wrapping_mul(HMULT)) ^ byte_table[(code & 0xff) as usize];
        hash = (hash.wrapping_mul(HMULT)) ^ byte_table[((code >> 8) & 0xff) as usize];
    }

    let signed = hash as i64;
    let magnitude = if signed < 0 {
        signed.wrapping_neg() as u64
    } else {
        signed as u64
    };

    format!("{:x}", magnitude)
}

#[cfg(feature = "bif-crypto")]
fn generate_hash_lookup_table() -> [u64; 256] {
    let mut table = [0u64; 256];
    let mut h = 0x544B2FBACAAF1684u64;
    let mut i = 0;

    while i < 256 {
        let mut j = 0;
        while j < 31 {
            h = (h >> 7) ^ h;
            h = (h << 11) ^ h;
            h = (h >> 10) ^ h;
            j += 1;
        }
        table[i] = h;
        i += 1;
    }

    table
}

#[cfg(feature = "bif-crypto")]
fn md5_digest(input: &[u8]) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(input);
    hasher.finalize().into()
}

#[cfg(feature = "bif-crypto")]
fn sha1_digest(input: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(input);
    hasher.finalize().into()
}

#[cfg(feature = "bif-crypto")]
fn sha224_digest(input: &[u8]) -> [u8; 28] {
    let mut hasher = Sha224::new();
    hasher.update(input);
    hasher.finalize().into()
}

#[cfg(feature = "bif-crypto")]
fn sha256_digest(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

#[cfg(feature = "bif-crypto")]
fn sha384_digest(input: &[u8]) -> [u8; 48] {
    let mut hasher = Sha384::new();
    hasher.update(input);
    hasher.finalize().into()
}

#[cfg(feature = "bif-crypto")]
fn sha512_digest(input: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(input);
    hasher.finalize().into()
}

#[cfg(feature = "bif-crypto")]
fn hmac_md5(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    type HmacMd5 = Hmac<Md5>;
    let mut mac = HmacMd5::new_from_slice(key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(feature = "bif-crypto")]
fn hmac_sha1(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(feature = "bif-crypto")]
fn hmac_sha224(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    type HmacSha224 = Hmac<Sha224>;
    let mut mac = HmacSha224::new_from_slice(key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(feature = "bif-crypto")]
fn hmac_sha256(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(feature = "bif-crypto")]
fn hmac_sha384(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    type HmacSha384 = Hmac<Sha384>;
    let mut mac = HmacSha384::new_from_slice(key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(feature = "bif-crypto")]
fn hmac_sha512(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    type HmacSha512 = Hmac<Sha512>;
    let mut mac = HmacSha512::new_from_slice(key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(feature = "bif-crypto")]
const DEFAULT_ENCRYPTION_ALGORITHM: &str = "AES";
#[cfg(feature = "bif-crypto")]
const DEFAULT_ENCRYPTION_ENCODING: &str = "UU";
#[cfg(feature = "bif-crypto")]
const DEFAULT_ENCRYPTION_KEY_SIZE: usize = 256;
#[cfg(feature = "bif-crypto")]
const DEFAULT_ENCRYPTION_ITERATIONS: usize = 1000;

#[cfg(feature = "bif-crypto")]
fn default_key_size_for_algorithm(algorithm: &str) -> usize {
    match normalize_algorithm_name(algorithm).as_str() {
        "aes" => 256,
        "des" => 56,
        "desede" | "tripledes" => 168,
        "blowfish" => 128,
        "arcfour" | "rc4" => 128,
        "hmacmd5" => 128,
        "hmacsha1" => 160,
        "hmacsha224" => 224,
        "hmacsha256" => 256,
        "hmacsha384" => 384,
        "hmacsha512" => 512,
        _ => DEFAULT_ENCRYPTION_KEY_SIZE,
    }
}

#[cfg(feature = "bif-crypto")]
pub fn generate_secret_key(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let algorithm = args
        .first()
        .map(|v| vm.to_string(*v))
        .unwrap_or_else(|| DEFAULT_ENCRYPTION_ALGORITHM.to_string());
    let key_size_bits = args
        .get(1)
        .map(|v| v.as_number() as usize)
        .unwrap_or_else(|| default_key_size_for_algorithm(&algorithm));

    let key_size_bytes = (key_size_bits + 7) / 8;
    let mut key_bytes = vec![0u8; key_size_bytes];
    let mut rng = rand::rng();
    for byte in key_bytes.iter_mut() {
        *byte = rng.random();
    }

    Ok(BxValue::new_ptr(vm.string_new(base64_encode_bytes(&key_bytes))))
}

#[cfg(feature = "bif-crypto")]
pub fn generate_pbkdf_key(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 3 {
        return Err("generatePBKDFKey() expects at least 3 arguments: (algorithm, passphrase, salt)".to_string());
    }

    let algorithm = vm.to_string(args[0]);
    let passphrase = vm.to_string(args[1]);
    let salt = vm.to_string(args[2]);
    let iterations = args
        .get(3)
        .map(|v| v.as_number() as usize)
        .unwrap_or(DEFAULT_ENCRYPTION_ITERATIONS)
        .max(1);
    let key_size_bits = args
        .get(4)
        .map(|v| v.as_number() as usize)
        .unwrap_or(DEFAULT_ENCRYPTION_KEY_SIZE);

    let normalized = normalize_algorithm_name(&algorithm);
    let normalized = match normalized.as_str() {
        "pbkdf2withsha1" => "pbkdf2withhmacsha1".to_string(),
        "pbkdf2withsha224" => "pbkdf2withhmacsha224".to_string(),
        "pbkdf2withsha256" => "pbkdf2withhmacsha256".to_string(),
        "pbkdf2withsha384" => "pbkdf2withhmacsha384".to_string(),
        "pbkdf2withsha512" => "pbkdf2withhmacsha512".to_string(),
        other => other.to_string(),
    };

    let key_bytes = pbkdf2_derive(&normalized, passphrase.as_bytes(), salt.as_bytes(), iterations, key_size_bits)?;

    Ok(BxValue::new_ptr(vm.string_new(base64_encode_bytes(&key_bytes))))
}

#[cfg(feature = "bif-crypto")]
pub fn encrypt_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("encrypt() expects at least 2 arguments: (string, key)".to_string());
    }
    let _object = vm.to_string(args[0]);
    let _key = vm.to_string(args[1]);
    let algorithm = args
        .get(2)
        .map(|v| vm.to_string(*v))
        .unwrap_or_else(|| DEFAULT_ENCRYPTION_ALGORITHM.to_string());
    let _encoding = args
        .get(3)
        .map(|v| vm.to_string(*v))
        .unwrap_or_else(|| DEFAULT_ENCRYPTION_ENCODING.to_string());

    Err(format!(
        "encrypt() with algorithm '{}' is not yet supported. Cipher implementations (AES, DES, etc.) require additional crates.",
        algorithm
    ))
}

#[cfg(feature = "bif-crypto")]
pub fn decrypt_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("decrypt() expects at least 2 arguments: (string, key)".to_string());
    }
    let _encrypted = vm.to_string(args[0]);
    let _key = vm.to_string(args[1]);
    let algorithm = args
        .get(2)
        .map(|v| vm.to_string(*v))
        .unwrap_or_else(|| DEFAULT_ENCRYPTION_ALGORITHM.to_string());

    Err(format!(
        "decrypt() with algorithm '{}' is not yet supported. Cipher implementations (AES, DES, etc.) require additional crates.",
        algorithm
    ))
}

#[cfg(feature = "bif-crypto")]
fn pbkdf2_derive(
    algorithm: &str,
    password: &[u8],
    salt: &[u8],
    iterations: usize,
    key_size_bits: usize,
) -> Result<Vec<u8>, String> {
    let hash_len = match algorithm {
        "pbkdf2withhmacsha1" => 20,
        "pbkdf2withhmacsha224" => 28,
        "pbkdf2withhmacsha256" => 32,
        "pbkdf2withhmacsha384" => 48,
        "pbkdf2withhmacsha512" => 64,
        _ => {
            return Err(format!(
                "Unsupported PBKDF2 algorithm: {}. Supported: PBKDF2WithHmacSHA1, PBKDF2WithHmacSHA224, PBKDF2WithHmacSHA256, PBKDF2WithHmacSHA384, PBKDF2WithHmacSHA512",
                algorithm
            ));
        }
    };

    let key_size_bytes = (key_size_bits + 7) / 8;
    let num_blocks = (key_size_bytes + hash_len - 1) / hash_len;
    let mut dk = Vec::with_capacity(num_blocks * hash_len);

    for block_num in 1..=num_blocks {
        let u = pbkdf2_f(password, salt, iterations, block_num as u32, algorithm)?;
        let mut t = u.clone();
        dk.append(&mut t);
    }

    dk.truncate(key_size_bytes);
    Ok(dk)
}

#[cfg(feature = "bif-crypto")]
fn pbkdf2_f(
    password: &[u8],
    salt: &[u8],
    iterations: usize,
    block_num: u32,
    algorithm: &str,
) -> Result<Vec<u8>, String> {
    let mut salt_with_index = salt.to_vec();
    salt_with_index.extend_from_slice(&block_num.to_be_bytes());

    let mut u = pbkdf2_hmac(password, &salt_with_index, algorithm)?;
    let mut result = u.clone();

    for _ in 1..iterations {
        u = pbkdf2_hmac(password, &u, algorithm)?;
        for (r, u_byte) in result.iter_mut().zip(u.iter()) {
            *r ^= u_byte;
        }
    }

    Ok(result)
}

#[cfg(feature = "bif-crypto")]
fn pbkdf2_hmac(key: &[u8], data: &[u8], algorithm: &str) -> Result<Vec<u8>, String> {
    match algorithm {
        "pbkdf2withhmacsha1" => {
            type H = Hmac<Sha1>;
            let mut mac = H::new_from_slice(key).map_err(|e| format!("HMAC error: {}", e))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "pbkdf2withhmacsha224" => {
            type H = Hmac<Sha224>;
            let mut mac = H::new_from_slice(key).map_err(|e| format!("HMAC error: {}", e))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "pbkdf2withhmacsha256" => {
            type H = Hmac<Sha256>;
            let mut mac = H::new_from_slice(key).map_err(|e| format!("HMAC error: {}", e))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "pbkdf2withhmacsha384" => {
            type H = Hmac<Sha384>;
            let mut mac = H::new_from_slice(key).map_err(|e| format!("HMAC error: {}", e))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "pbkdf2withhmacsha512" => {
            type H = Hmac<Sha512>;
            let mut mac = H::new_from_slice(key).map_err(|e| format!("HMAC error: {}", e))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        _ => Err(format!("Unsupported PBKDF2 HMAC algorithm: {}", algorithm)),
    }
}

#[cfg(feature = "bif-crypto")]
fn base64_encode_bytes(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
