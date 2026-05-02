//! HolonAST custom EDN handler — delegates to holon-rs's
//! `canonical_edn_holon` (proven byte-stable, used by signed-load
//! digests).
//!
//! Per crate-shape.md (beat 8): the structural default would also
//! work for HolonAST, but registering the existing canonical
//! function is easier than verifying byte-equivalence — and the
//! signed-load infrastructure already depends on its output, so
//! using it directly keeps every wat-emitted HolonAST blob
//! signature-compatible with that path.

use crate::value::{EdnError, Value};
use wat::runtime::RustDepsBuilder;

/// Register the HolonAST CustomEdn handler on the wat type registry.
///
/// Called once at startup from `lib.rs::register`.
pub fn register_holon_ast_handler(builder: &mut RustDepsBuilder) {
    // SCRATCH: actual registration shape depends on wat-rs's
    // CustomEdn API which doesn't exist yet. Intent:
    //
    //   builder.register_custom_edn(
    //       ":wat::holon::HolonAST",
    //       Box::new(holon_ast_decode),
    //       Box::new(holon_ast_encode),
    //   );
    //
    // For now, a placeholder that touches the builder so it isn't
    // flagged as unused.
    let _ = builder;
}

/// Decode an EDN body into a HolonAST value.
/// Routes to holon-rs's canonical reader.
#[allow(dead_code)]
fn holon_ast_decode(
    _body: edn_format::Value,
    _st: &wat::runtime::SymbolTable,
) -> Result<Value, EdnError> {
    // SCRATCH: would call into holon::canonical_edn_holon::decode
    // and wrap the resulting HolonAST in a Value (likely as a Tagged
    // value pointing at the wat::holon::HolonAST type).
    //
    // Pseudocode:
    //   let bytes = edn_format::emit_bytes(&body);
    //   let ast = holon::canonical_edn_holon::decode(&bytes)
    //       .map_err(|e| EdnError::Parse(e.to_string()))?;
    //   Ok(holon_ast_to_value(ast))
    Err(EdnError::Other(
        "holon_ast_decode pending holon-rs canonical_edn_holon surface".into(),
    ))
}

/// Encode a HolonAST value to EDN.
/// Routes to holon-rs's canonical writer.
#[allow(dead_code)]
fn holon_ast_encode(
    _value: &Value,
    _st: &wat::runtime::SymbolTable,
) -> Result<edn_format::Value, EdnError> {
    // SCRATCH: would extract the HolonAST from the Value, call
    // holon::canonical_edn_holon::encode, and parse the result back
    // into edn-format::Value for emission.
    //
    // Pseudocode:
    //   let ast = value_to_holon_ast(value)?;
    //   let bytes = holon::canonical_edn_holon::encode(&ast);
    //   edn_format::parse_bytes(&bytes)
    //       .map_err(|e| EdnError::Parse(e.to_string()))
    Err(EdnError::Other(
        "holon_ast_encode pending holon-rs canonical_edn_holon surface".into(),
    ))
}
