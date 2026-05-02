//! wat-side mirror of edn-format::Value.
//!
//! `Value` is the dynamic fallback type — used when reading
//! untagged EDN, or when a tag has no SymbolTable entry. Tagged
//! values that DO have type entries return their typed wat-side
//! representation directly (typed reads bypass `Value`).

use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),

    /// Already in wat keyword-path form (`:wat::core::foo`).
    /// Transcoded from EDN's dot-form during read.
    Keyword(String),

    Symbol(String),

    Vec(Vec<Value>),
    Map(BTreeMap<Value, Value>),
    Set(BTreeSet<Value>),

    /// Unknown tag — preserved for round-trip.
    /// `tag` is in wat keyword-path form.
    Tagged {
        tag: String,
        body: Box<Value>,
    },

    /// Built-in EDN tag #inst.
    Inst(chrono::DateTime<chrono::Utc>),

    /// Built-in EDN tag #uuid.
    Uuid(uuid::Uuid),
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum EdnError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Unknown tag: {0}")]
    UnknownTag(String),

    #[error("Field missing: {0}")]
    MissingField(String),

    #[error("Variant arity wrong: {variant} expected {expected} args, got {got}")]
    WrongVariantArity {
        variant: String,
        expected: usize,
        got: usize,
    },

    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),

    #[error("Other: {0}")]
    Other(String),
}

// ─── Eq + Ord for use as map/set keys ────────────────────────────

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SCRATCH: deterministic but minimal ordering for keying.
        // Real implementation should match edn-format's value ordering
        // for byte-equivalent round-trips of maps/sets.
        use std::cmp::Ordering;

        let tag = |v: &Value| match v {
            Value::Nil => 0,
            Value::Bool(_) => 1,
            Value::I64(_) => 2,
            Value::F64(_) => 3,
            Value::String(_) => 4,
            Value::Keyword(_) => 5,
            Value::Symbol(_) => 6,
            Value::Vec(_) => 7,
            Value::Map(_) => 8,
            Value::Set(_) => 9,
            Value::Tagged { .. } => 10,
            Value::Inst(_) => 11,
            Value::Uuid(_) => 12,
        };

        match tag(self).cmp(&tag(other)) {
            Ordering::Equal => match (self, other) {
                (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
                (Value::I64(a), Value::I64(b)) => a.cmp(b),
                (Value::String(a), Value::String(b)) => a.cmp(b),
                (Value::Keyword(a), Value::Keyword(b)) => a.cmp(b),
                (Value::Symbol(a), Value::Symbol(b)) => a.cmp(b),
                (Value::F64(a), Value::F64(b)) => {
                    a.partial_cmp(b).unwrap_or(Ordering::Equal)
                }
                _ => Ordering::Equal,
            },
            other => other,
        }
    }
}
