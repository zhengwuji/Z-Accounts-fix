//! `.zsb` encrypted bundle — PBKDF2-HMAC-SHA256 + AES-256-GCM (via ring).
//! Plus the ZCode `enc:v1` credential-value cipher (SHA-256 secret key +
//! AES-256-GCM, `enc:v1:iv.tag.ct` in URL-safe base64).

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde_json::{json, Value};

pub const FORMAT: &str = "zsw-accounts-bundle";
const KDF_ALG: &str = "pbkdf2-hmac-sha256";
const CIPHER: &str = "aes-256-gcm";
const DEFAULT_ITERS: u32 = 310_000;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

// ----------------------------------------------------- zcode enc:v1 values
//
// ZCode desktop encrypts sensitive credential values at rest:
//   `enc:v1:` + b64url(iv 12B) + "." + b64url(tag 16B) + "." + b64url(ct)
// key = SHA256(secret), secret = env ZCODE_CREDENTIAL_SECRET (trimmed), else
// `zcode-credential-fallback:{platform}:{homedir}:{username}`.

mod zcode_b64 {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD as E;
    use base64::Engine;
    pub fn enc(b: &[u8]) -> String {
        E.encode(b)
    }
    pub fn dec(s: &str) -> Result<Vec<u8>, String> {
        E.decode(s).map_err(|e| e.to_string())
    }
}

fn credential_secret() -> String {
    if let Ok(s) = std::env::var("ZCODE_CREDENTIAL_SECRET") {
        let t = s.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "?".into());
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".into());
    format!("zcode-credential-fallback:win32:{home}:{user}")
}

fn credential_key() -> [u8; 32] {
    use ring::digest;
    let d = digest::digest(&digest::SHA256, credential_secret().as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(d.as_ref());
    out
}

pub mod zcode_cred {
    use super::*;

    pub fn is_encrypted(v: &str) -> bool {
        v.starts_with("enc:v1:")
    }

    /// Decrypt one `enc:v1:` value; non-encrypted values pass through.
    pub fn decrypt_value(v: &str) -> String {
        if !is_encrypted(v) {
            return v.to_string();
        }
        let Some(body) = v.strip_prefix("enc:v1:") else {
            return v.to_string();
        };
        let parts: Vec<&str> = body.split('.').collect();
        if parts.len() != 3 {
            return v.to_string();
        }
        let (Ok(iv), Ok(tag), Ok(ct)) = (
            zcode_b64::dec(parts[0]),
            zcode_b64::dec(parts[1]),
            zcode_b64::dec(parts[2]),
        ) else {
            return v.to_string();
        };
        if iv.len() != 12 || tag.len() != 16 {
            return v.to_string();
        }
        use ring::aead;
        let key = credential_key();
        let Ok(unbound) = aead::UnboundKey::new(&aead::AES_256_GCM, &key) else {
            return v.to_string();
        };
        let opening = aead::LessSafeKey::new(unbound);
        let Ok(nonce_arr) = <[u8; 12]>::try_from(iv.as_slice()) else {
            return v.to_string();
        };
        let mut buf = ct;
        buf.extend_from_slice(&tag);
        match opening.open_in_place(
            aead::Nonce::assume_unique_for_key(nonce_arr),
            aead::Aad::empty(),
            &mut buf,
        ) {
            Ok(plain) => String::from_utf8_lossy(plain).to_string(),
            Err(_) => v.to_string(),
        }
    }

    /// Encrypt one value into `enc:v1:` form (matching the original's
    /// at-rest snapshot format).
    pub fn encrypt_value(plain: &str) -> Result<String, String> {
        use ring::aead;
        use ring::rand::{SecureRandom, SystemRandom};
        let key = credential_key();
        let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key).map_err(|e| e.to_string())?;
        let sealing = aead::LessSafeKey::new(unbound);
        let rng = SystemRandom::new();
        let mut iv = [0u8; 12];
        rng.fill(&mut iv).map_err(|e| e.to_string())?;
        let mut buf = plain.as_bytes().to_vec();
        sealing
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(iv),
                aead::Aad::empty(),
                &mut buf,
            )
            .map_err(|e| e.to_string())?;
        let ct_len = buf.len() - 16;
        let tag = buf.split_off(ct_len);
        Ok(format!(
            "enc:v1:{}.{}.{}",
            zcode_b64::enc(&iv),
            zcode_b64::enc(&tag),
            zcode_b64::enc(&buf)
        ))
    }

    /// Decrypt every encrypted string value in a credentials object.
    pub fn decrypt_creds(v: &Value) -> Value {
        let mut out = v.clone();
        if let Some(obj) = out.as_object_mut() {
            for (_, val) in obj.iter_mut() {
                if let Some(s) = val.as_str() {
                    if is_encrypted(s) {
                        *val = Value::String(decrypt_value(s));
                    }
                }
            }
        }
        out
    }

    /// Encrypt every string value (matching the original's snapshot format).
    pub fn encrypt_creds(v: &Value) -> Result<Value, String> {
        let mut out = v.clone();
        if let Some(obj) = out.as_object_mut() {
            for (_, val) in obj.iter_mut() {
                if let Some(s) = val.as_str() {
                    if !s.is_empty() && !is_encrypted(s) {
                        *val = Value::String(encrypt_value(s)?);
                    }
                }
            }
        }
        Ok(out)
    }
}

pub fn seal(plain: &[u8], password: &str) -> Result<Value, String> {
    use ring::aead;
    use ring::pbkdf2;
    use ring::rand::{SecureRandom, SystemRandom};

    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill(&mut salt).map_err(|e| e.to_string())?;
    rng.fill(&mut nonce).map_err(|e| e.to_string())?;

    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(DEFAULT_ITERS).unwrap(),
        &salt,
        password.as_bytes(),
        &mut key,
    );
    let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key).map_err(|e| e.to_string())?;
    let sealing = aead::LessSafeKey::new(unbound);
    let mut buf = plain.to_vec();
    sealing
        .seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut buf)
        .map_err(|e| e.to_string())?;
    let ct_len = buf.len() - TAG_LEN;
    let tag_bytes = buf.split_off(ct_len);

    Ok(json!({
        "format": FORMAT,
        "version": 1,
        "kdf": { "alg": KDF_ALG, "iters": DEFAULT_ITERS },
        "cipher": CIPHER,
        "salt": B64.encode(salt),
        "nonce": B64.encode(nonce),
        "tag": B64.encode(tag_bytes.as_slice()),
        "data": B64.encode(buf.as_slice()),
    }))
}

pub fn open(doc: &Value, password: &str) -> Result<Vec<u8>, String> {
    let kdf = doc
        .get("kdf")
        .and_then(|v| v.as_object())
        .ok_or("missing kdf")?;
    let iters = kdf
        .get("iters")
        .and_then(|v| v.as_u64())
        .filter(|&n| n > 0 && n <= 20_000_000)
        .unwrap_or(DEFAULT_ITERS as u64) as u32;
    let alg = kdf.get("alg").and_then(|v| v.as_str()).unwrap_or("");
    if alg != KDF_ALG {
        return Err(format!("unsupported kdf: {alg}"));
    }
    let cipher = doc.get("cipher").and_then(|v| v.as_str()).unwrap_or("");
    if cipher != CIPHER {
        return Err(format!("unsupported cipher: {cipher}"));
    }
    let salt = B64.decode(doc.get("salt").and_then(|v| v.as_str()).ok_or("missing salt")?)
        .map_err(|_| "bad salt")?;
    let nonce_b = B64.decode(doc.get("nonce").and_then(|v| v.as_str()).ok_or("missing nonce")?)
        .map_err(|_| "bad nonce")?;
    let tag = B64.decode(doc.get("tag").and_then(|v| v.as_str()).ok_or("missing tag")?)
        .map_err(|_| "bad tag")?;
    let data = B64.decode(doc.get("data").and_then(|v| v.as_str()).ok_or("missing data")?)
        .map_err(|_| "bad data")?;
    if nonce_b.len() != NONCE_LEN || tag.len() != TAG_LEN {
        return Err("bad nonce/tag length".into());
    }

    use ring::aead;
    use ring::pbkdf2;
    let mut key = [0u8; 32];
    let iters_nz = std::num::NonZeroU32::new(iters).ok_or("bad kdf iterations")?;
    pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256, iters_nz, &salt, password.as_bytes(), &mut key);
    let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key).map_err(|e| e.to_string())?;
    let opening = aead::LessSafeKey::new(unbound);
    let mut buf = data;
    buf.extend_from_slice(&tag);
    let plain = opening
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce_b.try_into().map_err(|_| "bad nonce")?),
            aead::Aad::empty(),
            &mut buf,
        )
        .map_err(|_| "wrong password or corrupted file".to_string())?;
    Ok(plain.to_vec())
}
