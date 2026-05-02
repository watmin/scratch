//! The dispatch loop: read/write EDN, walk SymbolTable metadata.
//!
//! See crate-shape.md (beat 8 of arc 003) for the architectural
//! framing. This module implements the router. Type-specific
//! handlers are NOT here — the SymbolTable is the authority.

use crate::transcode::{path_to_tag, path_to_edn_keyword, tag_to_path};
use crate::value::{EdnError, Value};
use edn_format::{Symbol as EdnSymbol, Value as EdnValue};

// SCRATCH NOTE: the wat::runtime API surface assumed below is
// best-effort, derived from project memory. Verify against the
// actual wat-rs API before lifting. Specifically: `SymbolTable`,
// `TypeEntry`, `StructEntry::fields()`, `ParametricEntry::constructor_name()`,
// `ParametricEntry::type_args()` are stubs likely to need adjustment.

use wat::runtime::{
    EnumEntry, ParametricEntry, StructEntry, SymbolTable, TypeEntry,
};

/// Top-level read entrypoint — return a `Value` (typed if the tag
/// resolves; dynamic Tagged otherwise).
pub fn read_str(input: &str, st: &SymbolTable) -> Result<Value, EdnError> {
    let edn = edn_format::parse_str(input)
        .map_err(|e| EdnError::Parse(e.to_string()))?;
    decode_edn_value(edn, st)
}

/// Typed read — coerce the result to a specific wat type path.
pub fn read_str_as(
    input: &str,
    expected_path: &str,
    st: &SymbolTable,
) -> Result<Value, EdnError> {
    let parsed = read_str(input, st)?;
    coerce_to(parsed, expected_path, st)
}

/// Top-level write entrypoint.
pub fn write_str(value: &Value, st: &SymbolTable) -> Result<String, EdnError> {
    let edn = encode_to_edn(value, st)?;
    Ok(edn_format::emit_str(&edn))
}

// ─── Read direction ─────────────────────────────────────────────

fn decode_edn_value(edn: EdnValue, st: &SymbolTable) -> Result<Value, EdnError> {
    match edn {
        EdnValue::Nil => Ok(Value::Nil),
        EdnValue::Boolean(b) => Ok(Value::Bool(b)),
        EdnValue::Integer(i) => Ok(Value::I64(i)),
        EdnValue::Float(f) => Ok(Value::F64(f.into_inner())),
        EdnValue::String(s) => Ok(Value::String(s)),
        EdnValue::Keyword(k) => {
            // Transcode EDN keyword (dot form) to wat path (:: form)
            let body = k.namespaced_name();
            Ok(Value::Keyword(format!(":{}", body.replace('.', "::"))))
        }
        EdnValue::Symbol(s) => Ok(Value::Symbol(s.namespaced_name())),
        EdnValue::Vector(v) => {
            let items = v
                .into_iter()
                .map(|e| decode_edn_value(e, st))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Vec(items))
        }
        EdnValue::Map(m) => {
            let entries = m
                .into_iter()
                .map(|(k, v)| {
                    Ok((decode_edn_value(k, st)?, decode_edn_value(v, st)?))
                })
                .collect::<Result<_, EdnError>>()?;
            Ok(Value::Map(entries))
        }
        EdnValue::Set(s) => {
            let items = s
                .into_iter()
                .map(|e| decode_edn_value(e, st))
                .collect::<Result<_, EdnError>>()?;
            Ok(Value::Set(items))
        }
        EdnValue::Inst(dt) => Ok(Value::Inst(dt)),
        EdnValue::Uuid(u) => Ok(Value::Uuid(u)),
        EdnValue::TaggedElement(tag, body) => dispatch_tag(&tag, *body, st),
        // SCRATCH: handle BigInt, BigDec, Char as wat-rs requires
        _ => Err(EdnError::Other("unhandled edn value variant".into())),
    }
}

/// The core router. Look up the tag in the SymbolTable, walk metadata.
fn dispatch_tag(
    tag: &EdnSymbol,
    body: EdnValue,
    st: &SymbolTable,
) -> Result<Value, EdnError> {
    let tag_str = tag.namespaced_name();
    let wat_path = tag_to_path(&tag_str);

    match st.lookup_type(&wat_path) {
        Some(entry) => walk_metadata(&entry, body, st),
        None => {
            // Dynamic fallback: preserve as Tagged.
            let body_decoded = decode_edn_value(body, st)?;
            Ok(Value::Tagged {
                tag: wat_path,
                body: Box::new(body_decoded),
            })
        }
    }
}

fn walk_metadata(
    entry: &TypeEntry,
    body: EdnValue,
    st: &SymbolTable,
) -> Result<Value, EdnError> {
    match entry {
        TypeEntry::Struct(s) => decode_struct(s, body, st),
        TypeEntry::Enum(e) => decode_enum(e, body, st),
        TypeEntry::Parametric(p) => decode_parametric(p, body, st),
        TypeEntry::CustomEdn(handler) => handler(body, st),
    }
}

fn decode_struct(
    s: &StructEntry,
    body: EdnValue,
    st: &SymbolTable,
) -> Result<Value, EdnError> {
    let edn_map = match body {
        EdnValue::Map(m) => m,
        other => {
            return Err(EdnError::TypeMismatch {
                expected: "map".into(),
                got: format!("{:?}", other),
            })
        }
    };

    let mut decoded = std::collections::BTreeMap::new();
    for (field_name, field_type_path) in s.fields() {
        // Field keys are bare keywords like :asset, :size — no namespace.
        let key = EdnValue::Keyword(
            edn_format::Keyword::from_namespaced_name(field_name),
        );
        let raw = edn_map.get(&key).cloned().ok_or_else(|| {
            EdnError::MissingField(field_name.to_string())
        })?;
        let coerced = coerce_to_type(raw, field_type_path, st)?;
        decoded.insert(Value::Keyword(format!(":{}", field_name)), coerced);
    }
    Ok(Value::Tagged {
        tag: format!(":{}", s.qualified_name()),
        body: Box::new(Value::Map(decoded)),
    })
}

fn decode_enum(
    _e: &EnumEntry,
    _body: EdnValue,
    _st: &SymbolTable,
) -> Result<Value, EdnError> {
    // SCRATCH: per Style A (sum-style-resolution.md, beat 7), each
    // enum variant has its own EDN tag. So when we get here via
    // dispatch_tag, the variant is already known from the tag.
    // The enum type-entry case is for the OUTER lookup; per-variant
    // dispatch routes through decode_parametric (Some/None/Ok/Err).
    //
    // This path will need wat-rs's actual EnumEntry layout once
    // Arc 048 (user-defined enums) lands. For now, return an
    // "unsupported" error to make the gap visible during testing.
    Err(EdnError::Other(
        "user-defined enum decoding pending wat-rs Arc 048".into(),
    ))
}

fn decode_parametric(
    p: &ParametricEntry,
    body: EdnValue,
    st: &SymbolTable,
) -> Result<Value, EdnError> {
    match (p.constructor_name(), body) {
        // ─── Collections ────────────────────────────────────────
        ("Vec", EdnValue::Vector(items)) => {
            let t_path = p.type_args()[0];
            let coerced = items
                .into_iter()
                .map(|e| coerce_to_type(e, t_path, st))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Vec(coerced))
        }
        ("HashSet", EdnValue::Set(items)) => {
            let t_path = p.type_args()[0];
            let coerced = items
                .into_iter()
                .map(|e| coerce_to_type(e, t_path, st))
                .collect::<Result<_, _>>()?;
            Ok(Value::Set(coerced))
        }
        ("HashMap", EdnValue::Map(entries)) => {
            let k_path = p.type_args()[0];
            let v_path = p.type_args()[1];
            let coerced = entries
                .into_iter()
                .map(|(k, v)| {
                    Ok((
                        coerce_to_type(k, k_path, st)?,
                        coerce_to_type(v, v_path, st)?,
                    ))
                })
                .collect::<Result<_, EdnError>>()?;
            Ok(Value::Map(coerced))
        }

        // ─── Sums (Style A — per-variant tag) ──────────────────
        ("Some", body) => {
            let t_path = p.type_args()[0];
            let inner = coerce_to_type(body, t_path, st)?;
            Ok(Value::Tagged {
                tag: format!(":wat::core::Some<{}>", t_path),
                body: Box::new(inner),
            })
        }
        ("None", EdnValue::Nil) => Ok(Value::Tagged {
            tag: format!(":wat::core::None<{}>", p.type_args()[0]),
            body: Box::new(Value::Nil),
        }),
        ("Ok", body) => {
            let t_path = p.type_args()[0];
            let e_path = p.type_args()[1];
            let inner = coerce_to_type(body, t_path, st)?;
            Ok(Value::Tagged {
                tag: format!(":wat::core::Ok<{}_{}>", t_path, e_path),
                body: Box::new(inner),
            })
        }
        ("Err", body) => {
            let t_path = p.type_args()[0];
            let e_path = p.type_args()[1];
            let inner = coerce_to_type(body, e_path, st)?;
            Ok(Value::Tagged {
                tag: format!(":wat::core::Err<{}_{}>", t_path, e_path),
                body: Box::new(inner),
            })
        }

        (other, body) => Err(EdnError::Other(format!(
            "no decoder for parametric {} with body {:?}",
            other, body
        ))),
    }
}

/// Coerce a raw EDN value to a target wat type path.
/// Used for struct field decoding and parametric arg coercion.
fn coerce_to_type(
    edn: EdnValue,
    target_path: &str,
    st: &SymbolTable,
) -> Result<Value, EdnError> {
    // Primitives: pass through.
    if matches!(
        target_path,
        ":i64" | ":f64" | ":bool" | ":String" | ":Keyword" | ":Instant" | ":Uuid"
    ) {
        return decode_edn_value(edn, st);
    }

    // Tagged: dispatch by tag.
    if let EdnValue::TaggedElement(tag, body) = edn {
        return dispatch_tag(&tag, *body, st);
    }

    // Otherwise: try general decode.
    decode_edn_value(edn, st)
}

fn coerce_to(
    value: Value,
    _target_path: &str,
    _st: &SymbolTable,
) -> Result<Value, EdnError> {
    // SCRATCH: real implementation verifies value matches target type;
    // for now pass through. The dispatch above already typed it.
    Ok(value)
}

// ─── Write direction ────────────────────────────────────────────

fn encode_to_edn(value: &Value, st: &SymbolTable) -> Result<EdnValue, EdnError> {
    match value {
        Value::Nil => Ok(EdnValue::Nil),
        Value::Bool(b) => Ok(EdnValue::Boolean(*b)),
        Value::I64(i) => Ok(EdnValue::Integer(*i)),
        Value::F64(f) => Ok(EdnValue::Float((*f).into())),
        Value::String(s) => Ok(EdnValue::String(s.clone())),
        Value::Keyword(k) => {
            let edn_kw = path_to_edn_keyword(k);
            Ok(EdnValue::Keyword(edn_format::Keyword::from_namespaced_name(
                edn_kw.strip_prefix(':').unwrap_or(&edn_kw),
            )))
        }
        Value::Symbol(s) => Ok(EdnValue::Symbol(
            edn_format::Symbol::from_namespaced_name(s),
        )),
        Value::Vec(items) => {
            let encoded = items
                .iter()
                .map(|v| encode_to_edn(v, st))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(EdnValue::Vector(encoded))
        }
        Value::Map(entries) => {
            let encoded = entries
                .iter()
                .map(|(k, v)| Ok((encode_to_edn(k, st)?, encode_to_edn(v, st)?)))
                .collect::<Result<_, EdnError>>()?;
            Ok(EdnValue::Map(encoded))
        }
        Value::Set(items) => {
            let encoded = items
                .iter()
                .map(|v| encode_to_edn(v, st))
                .collect::<Result<_, EdnError>>()?;
            Ok(EdnValue::Set(encoded))
        }
        Value::Tagged { tag, body } => {
            let edn_tag = path_to_tag(tag);
            let body_edn = encode_to_edn(body, st)?;
            Ok(EdnValue::TaggedElement(
                edn_format::Symbol::from_namespaced_name(&edn_tag),
                Box::new(body_edn),
            ))
        }
        Value::Inst(dt) => Ok(EdnValue::Inst(*dt)),
        Value::Uuid(u) => Ok(EdnValue::Uuid(*u)),
    }
}

#[cfg(test)]
mod tests {
    // SCRATCH: round-trip tests need a stub SymbolTable. Real
    // integration tests come with the first slice — see the
    // acceptance bar in ../README.md.
}
