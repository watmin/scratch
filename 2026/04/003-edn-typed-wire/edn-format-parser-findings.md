# edn-format parser findings — verifying the proposed scheme

## Why this beat

User: "3 - yes - let's confirm"

Beat 2 ended with a proposed scheme that relied on `<` `>` `:` all being legal in EDN tag names. The spec said yes to all three. The implementation might say something different. This beat checks the actual parser source.

## Library landscape (briefly, from the search)

- **`edn-rs`** (naomijub) — explicitly HALTED in its own README. Deprecated. Not viable.
- **`edn-format`** (bowbahdoe) — actively maintained, more complete API than edn-rs, exposes `Value::TaggedElement(Symbol, Box<Value>)` for tag handling. **The pick.**
- **`utkarshkukreti/edn.rs`** — older, less complete.
- **`cirru_edn`** — niche (Cirru-specific).

So the relevant parser is `bowbahdoe/edn-format`. Single-file crate, 2776 lines, all in `src/lib.rs`.

## The good — angle brackets are accepted

`is_allowed_atom_character` (lib.rs:459):

```rust
fn is_allowed_atom_character(c: char) -> bool {
    c == '.'
        || c == '*'
        || c == '+'
        || c == '!'
        || c == '-'
        || c == '_'
        || c == '?'
        || c == '$'
        || c == '%'
        || c == '&'
        || c == '='
        || c == '<'
        || c == '>'
        || c == '/'
        || c == ':'
        || c.is_alphabetic()
        || c.is_numeric()
}
```

So `<` `>` lex fine. `Vec<i64>` is one atom. Multi-arg with `_` (no commas, no spaces) is one atom: `HashMap<String_i64>`. So far so good.

## The bad — internal colons are rejected

`interpret_atom` (lib.rs:700-723) — this is where atoms get classified after lexing.

Keyword path:
```rust
&[':', ref rest @ ..] => {
    if rest.contains(&':') {
        Err(ParserError::CannotHaveColonInKeyword)
    } else {
        // ... split on '/', construct keyword ...
    }
}
```

Symbol path:
```rust
chars => {
    if chars.contains(&':') {
        Err(ParserError::CannotHaveColonInSymbol)
    } else {
        // ... split on '/', construct symbol ...
    }
}
```

Both error variants are real enum members:
- `CannotHaveColonInSymbol` (declared at line 376 area)
- `CannotHaveColonInKeyword` (declared nearby)

So **any internal colon in a symbol or keyword body is rejected categorically**, regardless of position.

## The spec disagrees with the lib

Spec verbatim:

> "Symbols begin with a non-numeric character and can contain
> alphanumeric characters and `. * + ! - _ ? $ % & = < >`. If `-`,
> `+` or `.` are the first character, the second character (if any)
> must be non-numeric. Additionally, `: #` are allowed as constituent
> characters in symbols other than as the first character."

So per spec, `:` is legal in symbol bodies (just not as first char). The lib is non-conformant.

The likely reason for the lib's strictness: to avoid ambiguity with the leading-`:` keyword reader. If a parser sees `:foo:bar`, is that one keyword or `:foo` followed by `:bar`? The spec resolves this by saying keywords can't begin with `::` (which catches `::foo`), but `:foo:bar` (one leading `:`, internal `:`) is technically legal per spec — just hard to distinguish at a glance. The lib pre-rejects the whole class.

## What this breaks in our beat-2 scheme

| Wanted in beat 2 | edn-format result | Why |
|---|---|---|
| `:wat::holon::Atom` (EDN keyword) | rejected | internal `::` in keyword body |
| `#wat::holon/Atom` (EDN tag) | rejected | `::` in prefix |
| `#wat/Vec<:i64>` (EDN tag name) | rejected | `:` inside `<>` |

`#wat/Vec<i64>` (no leading colon on the type arg) **does** still parse — angle brackets are atoms-internal, no colons inside. So we keep angle brackets, drop colons inside them.

The bigger break is the `::` namespace separator. Wat's keyword-path system uses `::` everywhere; the lib rejects every form of it.

## Three options surfaced for the user

**A. Live within edn-format's rules.** Use `.` instead of `::` inside EDN tags and keywords. wat reader transcodes `.` ↔ `::` at the boundary. EDN format and wat keyword-path system are different surfaces with different lex rules; the boundary is explicit.

**B. Fork or patch edn-format.** Spec says colons should be allowed; the rejection is a one-`if`-block fix. Upstream PR likely welcome (it's a conformance bug). Cost: another dependency to track.

**C. Roll our own EDN reader/writer.** Holon-rs already has `canonical_edn_holon` for AST serialization. ~1500 lines of new code. Removes a dep but adds bug surface.

User picked A in beat 4. Notes there.
