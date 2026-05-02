//! wat_edn — EDN reader/writer for wat.
//!
//! SCRATCH SKETCH from arc 003-edn-typed-wire (2026-04-26).
//! See ../../README.md for layout and the design notes for the
//! trajectory that produced this.
//!
//! # Architecture
//!
//! This crate is a ROUTER, not a registry. It consults the wat
//! SymbolTable at dispatch time for type metadata. Structural
//! walkers handle struct / enum / parametric types uniformly.
//! No bundled handlers for application types.
//!
//! HolonAST gets a CustomEdn handler that delegates to holon-rs's
//! `canonical_edn_holon` (proven byte-stable, used by signed-load
//! digests).
//!
//! # Wire format
//!
//! - Tags use dot-namespace form (`wat.holon/Atom`); the wat-side
//!   transcoder converts `::` ↔ `.` at the boundary.
//! - Generics use `<T_U>` (underscore separator since EDN treats
//!   `,` as whitespace).
//! - Sum types use Style A (per-variant tagging, e.g. `wat.core/Some<T>`).
//!
//! # Public surface (via #[wat_dispatch] under :rust::wat_edn)
//!
//! - `read-str`        :String -> :Result<:wat::edn::Value, :EdnError>
//! - `read-str-as<T>`  :String -> :Result<T, :EdnError>
//! - `write-str`       :T -> :String

mod dispatch;
mod holon_ast;
mod transcode;
mod value;

pub use dispatch::{read_str, read_str_as, write_str};
pub use transcode::{
    edn_keyword_to_path, path_to_edn_keyword, path_to_tag, tag_to_path,
};
pub use value::{EdnError, Value};

// SCRATCH NOTE: the exact #[wat_dispatch] attribute path and
// RustDepsBuilder / SymbolTable types should be verified against
// wat-rs's actual macro surface before lifting. The shape below
// matches what the project memory describes (Arc 060+ era).

use wat::main_macros::wat_dispatch;
use wat::runtime::{RustDepsBuilder, SymbolTable, WatSource};

#[wat_dispatch(path = ":rust::wat_edn::read-str", scope = "shared")]
pub fn wat_read_str(input: String, st: &SymbolTable) -> Result<Value, EdnError> {
    dispatch::read_str(&input, st)
}

#[wat_dispatch(path = ":rust::wat_edn::write-str", scope = "shared")]
pub fn wat_write_str(value: Value, st: &SymbolTable) -> Result<String, EdnError> {
    dispatch::write_str(&value, st)
}

/// Wire wat_edn's capabilities into the wat runtime.
/// Called from a consumer's `wat::main!` macro deps list.
pub fn register(builder: &mut RustDepsBuilder) {
    wat_read_str_register(builder);
    wat_write_str_register(builder);
    holon_ast::register_holon_ast_handler(builder);
}

/// No wat-side source files; everything is Rust shims + structural
/// dispatch consulting the SymbolTable.
pub fn wat_sources() -> &'static [WatSource] {
    &[]
}
