//! Boundary transcoder: wat keyword-paths ↔ EDN dot-form.
//!
//! Per dot-namespace-decision.md (beat 4 of arc 003):
//!
//! ```text
//!   wat path :a::b::c::Name  <->  EDN tag a.b.c/Name
//!   wat path :a::b::c::Name  <->  EDN keyword :a.b.c.Name
//! ```
//!
//! Conversion is mechanical, total, reversible. Inside generic
//! brackets, leading `:` on type args is stripped and `,` becomes
//! `_` (since EDN treats commas as whitespace).

/// Convert a wat keyword-path to an EDN tag (prefix.name form).
/// Last `::` becomes `/`; preceding `::` become `.`.
///
/// ```text
///   :wat::holon::Atom                    -> wat.holon/Atom
///   :enterprise::observer::Engram        -> enterprise.observer/Engram
///   :wat::core::Vec<:i64>                -> wat.core/Vec<i64>
///   :wat::core::HashMap<:String,:i64>    -> wat.core/HashMap<String_i64>
/// ```
pub fn path_to_tag(path: &str) -> String {
    let stripped = path.strip_prefix(':').unwrap_or(path);
    let cleaned = strip_inner_colons_in_generics(stripped);

    if let Some(idx) = cleaned.rfind("::") {
        let prefix = cleaned[..idx].replace("::", ".");
        let name = &cleaned[idx + 2..];
        format!("{}/{}", prefix, name)
    } else {
        cleaned
    }
}

/// Convert an EDN tag (prefix/name form) back to a wat keyword-path.
///
/// ```text
///   wat.holon/Atom                  -> :wat::holon::Atom
///   enterprise.observer/Engram      -> :enterprise::observer::Engram
/// ```
pub fn tag_to_path(tag: &str) -> String {
    let with_double_colons = tag.replace('.', "::").replace('/', "::");
    format!(":{}", with_double_colons)
}

/// EDN keyword body to wat keyword-path. Both forms have a leading `:`;
/// the body uses `.` in EDN, `::` in wat.
///
/// ```text
///   enterprise.observer.foo  ->  :enterprise::observer::foo
/// ```
///
/// Input is the keyword body without the leading `:`.
pub fn edn_keyword_to_path(kw_body: &str) -> String {
    format!(":{}", kw_body.replace('.', "::"))
}

/// wat keyword-path to EDN keyword form (with leading `:`).
///
/// ```text
///   :enterprise::observer::foo  ->  :enterprise.observer.foo
/// ```
pub fn path_to_edn_keyword(path: &str) -> String {
    let body = path.strip_prefix(':').unwrap_or(path);
    format!(":{}", body.replace("::", "."))
}

/// Inside generic angle brackets, strip leading `:` on type args
/// and convert internal `::` to `.`. Replace `,` with `_`.
///
/// Outside angle brackets, this function is identity.
fn strip_inner_colons_in_generics(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth: u32 = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                depth += 1;
                out.push(c);
            }
            '>' => {
                depth = depth.saturating_sub(1);
                out.push(c);
            }
            ':' if depth > 0 => {
                if matches!(chars.peek(), Some(':')) {
                    chars.next();
                    out.push('.');
                }
                // else: leading `:` of a type arg — drop silently
            }
            ',' if depth > 0 => out.push('_'),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_tag_simple() {
        assert_eq!(path_to_tag(":wat::holon::Atom"), "wat.holon/Atom");
    }

    #[test]
    fn path_to_tag_with_generic_simple() {
        assert_eq!(path_to_tag(":wat::core::Vec<:i64>"), "wat.core/Vec<i64>");
    }

    #[test]
    fn path_to_tag_multi_arg_generic() {
        assert_eq!(
            path_to_tag(":wat::core::HashMap<:String,:i64>"),
            "wat.core/HashMap<String_i64>"
        );
    }

    #[test]
    fn path_to_tag_nested_generic() {
        assert_eq!(
            path_to_tag(":wat::core::HashMap<:String,:Vec<:i64>>"),
            "wat.core/HashMap<String_Vec<i64>>"
        );
    }

    #[test]
    fn path_to_tag_qualified_generic_arg() {
        assert_eq!(
            path_to_tag(":wat::core::Vec<:wat::holon::HolonAST>"),
            "wat.core/Vec<wat.holon.HolonAST>"
        );
    }

    #[test]
    fn round_trip_bare() {
        let path = ":enterprise::observer::TradeSignal";
        assert_eq!(tag_to_path(&path_to_tag(path)), path);
    }

    #[test]
    fn keyword_round_trip() {
        let path = ":enterprise::observer::asset";
        let edn = path_to_edn_keyword(path);
        assert_eq!(edn, ":enterprise.observer.asset");
        assert_eq!(edn_keyword_to_path(&edn[1..]), path);
    }

    #[test]
    fn application_namespace() {
        assert_eq!(
            path_to_tag(":enterprise::observer::Engram"),
            "enterprise.observer/Engram"
        );
    }
}
