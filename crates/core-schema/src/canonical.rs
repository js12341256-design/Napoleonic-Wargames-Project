//! Canonical JSON serialization (PROMPT.md §5.2).
//!
//! Rules:
//! - keys sorted lexicographically at every level,
//! - no trailing whitespace, UTF-8, `\n` line endings,
//! - integers without decimal points,
//! - booleans as `true` / `false`,
//! - no `null`s — optional fields are omitted entirely.
//!
//! This module produces canonical bytes from any `serde_json::Value`.
//! Saved and hashed state must use [`to_canonical_string`] or
//! [`canonical_hash`]; never rely on default `serde_json` output.

use std::fmt::Write;

/// Errors raised when canonicalizing.
#[derive(Debug, thiserror::Error)]
pub enum CanonicalJsonError {
    #[error("floats are forbidden in canonical state (PROMPT.md §2.2): {0}")]
    FloatForbidden(f64),

    #[error("nulls are forbidden in canonical state (PROMPT.md §5.2)")]
    NullForbidden,

    #[error("serialization failure: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Serialize a value to canonical JSON.
///
/// `T: Serialize` is first converted to `serde_json::Value`, then
/// recursively rewritten with sorted keys and no nulls.
pub fn to_canonical_string<T: serde::Serialize>(value: &T) -> Result<String, CanonicalJsonError> {
    let v: serde_json::Value = serde_json::to_value(value)?;
    let mut out = String::new();
    write_value(&v, &mut out)?;
    Ok(out)
}

/// Serialize a value to canonical JSON bytes.
pub fn to_canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CanonicalJsonError> {
    Ok(to_canonical_string(value)?.into_bytes())
}

/// 32-byte BLAKE3 hash of the canonical representation, hex-encoded.
pub fn canonical_hash<T: serde::Serialize>(value: &T) -> Result<String, CanonicalJsonError> {
    let bytes = to_canonical_bytes(value)?;
    let hash = blake3_hex(&bytes);
    Ok(hash)
}

fn write_value(v: &serde_json::Value, out: &mut String) -> Result<(), CanonicalJsonError> {
    use serde_json::Value;
    match v {
        Value::Null => Err(CanonicalJsonError::NullForbidden),
        Value::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
            Ok(())
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                write!(out, "{i}").unwrap();
                Ok(())
            } else if let Some(u) = n.as_u64() {
                write!(out, "{u}").unwrap();
                Ok(())
            } else {
                // Float landed in canonical state; reject loudly.
                Err(CanonicalJsonError::FloatForbidden(
                    n.as_f64().unwrap_or(0.0),
                ))
            }
        }
        Value::String(s) => {
            write_json_string(s, out);
            Ok(())
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
            Ok(())
        }
        Value::Object(map) => {
            // Drop nulls; sort the remaining keys.
            let mut keys: Vec<&String> = map
                .iter()
                .filter(|(_, v)| !matches!(v, Value::Null))
                .map(|(k, _)| k)
                .collect();
            keys.sort_unstable();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json_string(k, out);
                out.push(':');
                write_value(&map[*k], out)?;
            }
            out.push('}');
            Ok(())
        }
    }
}

fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keys_are_sorted() {
        let v = json!({ "z": 1, "a": 2, "m": 3 });
        let s = to_canonical_string(&v).unwrap();
        assert_eq!(s, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn nulls_dropped_in_objects() {
        let v = json!({ "a": 1, "b": null, "c": 3 });
        let s = to_canonical_string(&v).unwrap();
        assert_eq!(s, r#"{"a":1,"c":3}"#);
    }

    #[test]
    fn null_at_top_level_errors() {
        let v: serde_json::Value = serde_json::Value::Null;
        let r = to_canonical_string(&v);
        assert!(matches!(r, Err(CanonicalJsonError::NullForbidden)));
    }

    #[test]
    fn floats_rejected() {
        let v = json!({ "x": 1.5 });
        let r = to_canonical_string(&v);
        assert!(matches!(r, Err(CanonicalJsonError::FloatForbidden(_))));
    }

    #[test]
    fn nested_keys_sorted() {
        let v = json!({
            "outer": { "z": 1, "a": 2 },
            "alpha": [{ "y": 1, "x": 2 }]
        });
        let s = to_canonical_string(&v).unwrap();
        assert_eq!(s, r#"{"alpha":[{"x":2,"y":1}],"outer":{"a":2,"z":1}}"#);
    }

    #[test]
    fn hash_is_stable() {
        let a = json!({ "a": 1, "b": 2 });
        let b = json!({ "b": 2, "a": 1 });
        let ha = canonical_hash(&a).unwrap();
        let hb = canonical_hash(&b).unwrap();
        assert_eq!(ha, hb);
    }

    #[test]
    fn hash_is_64_hex() {
        let a = json!({ "a": 1 });
        let h = canonical_hash(&a).unwrap();
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn strings_escape_control_chars() {
        let v = json!({ "x": "a\nb\tc" });
        let s = to_canonical_string(&v).unwrap();
        assert_eq!(s, r#"{"x":"a\nb\tc"}"#);
    }

    #[test]
    fn arrays_preserve_order() {
        let v = json!([3, 1, 2]);
        let s = to_canonical_string(&v).unwrap();
        assert_eq!(s, "[3,1,2]");
    }
}
