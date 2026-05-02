# Sum-style resolution — gaze closes Q1

## Why this beat

A reviewer of the scratch flagged:

> "Q1 (sum-type style A vs B) is the big one. The scratch flags it as 'must be picked uniformly … blocker for committing to wire format.' The realistic-blob example accidentally mixes both. This shapes the entire handler-tree code; I wouldn't open the arc without it resolved. Good /gaze candidate. … cast /gaze on Q1 (style A vs B) when you've got a slack moment. That settles the load-bearing decision so when a consumer does surface, the arc opens with all three Qs resolved and ships in one slice."

User: "let's address the issues"

The /gaze ward is about whether code communicates. Abstract design questions aren't its target — code is. So the move is: sketch concrete handler-tree code for both styles, then gaze at the sketches and let the ward report.

Skill source: `holon-lab-trading/.claude/skills/gaze/SKILL.md`. Severity model: Level 1 lies, Level 2 mumbles, Level 3 taste (noted but not flagged). The gaze converges when L1 and L2 are zero.

## The two handler-tree sketches (Rust, since this is where they'd land via `#[wat_dispatch]`)

### Style A — per-variant tag

```rust
fn register_option<T: WatType>(reg: &mut TagRegistry) {
    reg.add(format!("wat.core/Some<{}>", T::tag_name()), |body, ctx| {
        let inner = decode_as::<T>(body, ctx)?;
        Ok(WatValue::Option(Some(inner)))
    });
    reg.add(format!("wat.core/None<{}>", T::tag_name()), |body, _ctx| {
        expect_nil(body)?;
        Ok(WatValue::Option(None))
    });
}

fn register_result<T: WatType, E: WatType>(reg: &mut TagRegistry) {
    reg.add(format!("wat.core/Ok<{}_{}>", T::tag_name(), E::tag_name()), |body, ctx| {
        let inner = decode_as::<T>(body, ctx)?;
        Ok(WatValue::Result(Ok(inner)))
    });
    reg.add(format!("wat.core/Err<{}_{}>", T::tag_name(), E::tag_name()), |body, ctx| {
        let inner = decode_as::<E>(body, ctx)?;
        Ok(WatValue::Result(Err(inner)))
    });
}
```

### Style B — per-type tag with body discriminator

```rust
fn register_option<T: WatType>(reg: &mut TagRegistry) {
    reg.add(format!("wat.core/Option<{}>", T::tag_name()), |body, ctx| {
        let vec = body.as_vec().ok_or(EdnError::ExpectedVec)?;
        let variant = vec.first()
            .and_then(|v| v.as_keyword())
            .ok_or(EdnError::MissingDiscriminator)?;
        match variant.name() {
            "Some" => {
                if vec.len() != 2 {
                    return Err(EdnError::WrongArity {
                        variant: "Some", expected: 1, got: vec.len() - 1
                    });
                }
                let inner = decode_as::<T>(&vec[1], ctx)?;
                Ok(WatValue::Option(Some(inner)))
            }
            "None" => {
                if vec.len() != 1 {
                    return Err(EdnError::WrongArity {
                        variant: "None", expected: 0, got: vec.len() - 1
                    });
                }
                Ok(WatValue::Option(None))
            }
            other => Err(EdnError::UnknownVariant {
                type_: "Option", variant: other.to_string()
            }),
        }
    });
}
```

(Same shape repeats for `Result<T,E>` — ~25 lines per type instead of ~3 per variant.)

## Gaze findings

### Style A

- **Level 1 (lies):** none.
- **Level 2 (mumbles):** none.
- **Spark:** present. Closure body is three lines: decode body as inner type, wrap in variant constructor, return. The tag IS the dispatch — no second layer. No discriminator string to misspell. Function signature, registered tag string, and closure body all say the same thing in three different forms.

### Style B

- **Level 1 (lies):** one. The handler claims to handle `wat.core/Option<T>` but actually accepts ANY keyword as the variant discriminator — the type system cannot enforce that the wire keyword is one of `[Some, None]`. The error variant `UnknownVariant` exists precisely because a typo in the writer (`[:Soem 42]`) becomes a runtime error rather than a wire-format violation caught at the boundary. The tag promises a type; the body negotiates with strings to find out which variant. That mismatch between promise and mechanism is a Level 1 lie.
- **Level 2 (mumbles):** four.
  - The `vec.first().and_then(|v| v.as_keyword()).ok_or(...)` chain — three operations to extract a discriminator that a tag would have given for free.
  - Three error paths (`ExpectedVec`, `MissingDiscriminator`, `WrongArity`, `UnknownVariant`) that exist purely because the wire shape uses vec-with-keyword-head instead of a tag. None describe domain failures — they describe protocol overhead.
  - The `if vec.len() != 2` arity guards repeat per variant. The repetition is the function asking to be split — and Style A is what splitting it looks like.
  - Closure is ~25 lines vs Style A's ~3.
- **Spark:** dim. Structure does NOT mirror the wire form — it negotiates with it. The handler does protocol decoding before semantic decoding.

## Verdict

**Style A converges (zero L1, zero L2). Style B has 1 L1 and 4 L2s.**

Three reinforcing reasons beyond the per-handler comparison:

1. **HolonAST forces A by variant count.** Eleven variants makes Style B's outer wrapper pure noise (`#wat.holon/HolonAST [:Bind ...]` everywhere). If HolonAST goes A and Option/Result goes B, the codebase has a wire-format split — which the prior open-questions doc already flagged as a blocker. Style A applied uniformly avoids that.

2. **The tag IS a content-addressed identifier — that's what EDN tags ARE for.** Style A uses tags for what tags are: dispatch keys. One tag → one handler → one variant. Style B uses tags as type-names and invents a parallel dispatch mechanism (keyword-in-body) on top. Two dispatch layers where one suffices.

3. **Error surface shrinks.** Style A's only failure mode is "body doesn't decode as T" — the genuine domain error. Style B adds three protocol-level failures that have nothing to do with the data being wrong. They're failures of the wire shape being well-formed, which a typed wire format shouldn't have to negotiate at the handler level.

## The Clojure-interop counter-argument, addressed

The case for Style B was: Clojure programmers recognize `[:variant ...args]` as an idiomatic tagged-union body. Real, but weaker than it looks.

Clojure's `data_readers.clj` is exactly the per-tag handler mechanism EDN was designed for. A Clojure consumer registers one entry per tag in either style:
- Style A: register handlers for `wat.core/Some`, `wat.core/None`, `wat.core/Ok`, `wat.core/Err` — one per variant.
- Style B: register handlers for `wat.core/Option`, `wat.core/Result` — but each handler is internally more complex and must dispatch on the body keyword.

Style A asks Clojure to register more handlers (small cost — a few lines of `data_readers.clj`). Style B asks Clojure to handle the body shape inside one handler (larger cost — same body-discrimination logic the Rust handler has).

The interop cost is comparable; the within-codebase cost favors A by a wide margin.

## Resolution

**Q1 closed: Style A, applied uniformly across `Option`, `Result`, `HolonAST`, and any future user-defined enum that surfaces in EDN.**

Implications:
- `string-examples.md` updated — Option/Result examples shown only in Style A; realistic blob switched from `[:Some 64500.0]` to `#wat.core/Some<f64> 64500.0`.
- `open-questions.md` updated — Q1 marked resolved; Q2 and Q3 remain open.
- Handler-tree sketch above is the canonical reference for first-slice implementation when a consumer surfaces.

## Note on applying /gaze to design questions

The reviewer's framing — "ideally with a concrete handler-tree sketch in front of the ward so the cost difference between A and B is visible" — is the right move. /gaze checks communication; design choices that translate to code-shape can be evaluated by sketching that code and gazing at it. Without code in front of the ward, gaze has nothing to scan and the question stays abstract.

This is reusable: future design questions where two paths are equally "abstractly defensible" can be settled by sketching both paths' minimal handler/registration code and letting the ward report. The path with fewer mumbles wins.
