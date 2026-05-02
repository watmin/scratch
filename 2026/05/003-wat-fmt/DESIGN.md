# wat-fmt — DESIGN

Architecture, crate layout, CLI integration, public API, error
model, round-trip properties, configuration philosophy.

Style rules live in a separate file (`STYLE-RULES.md`) so the
user can mark them up without wading through architecture.

---

## Why a formatter is easy in wat (relatively)

Lisp formatters are easier than other languages' formatters
because **the AST is the canonical structure**. There's no
operator precedence, no associativity ambiguity, no sigil
placement debate. wat-fmt = walk the AST + emit text per
per-node rules + handle comments.

The hard problem is comment preservation; everything else
falls out of "format from the AST."

## The pipeline

```
source bytes
  ↓ tokenizer (with comment-token preservation)
tokens
  ↓ parser (attaches comments to adjacent AST nodes)
HolonAST + attached comments
  ↓ emitter (walks tree, applies per-node format rules)
formatted source bytes
```

**No regex. No line-by-line rewriting. No whitespace heuristics.**
The AST is the source of truth; formatting is a total function
over it. Comments ride along as structural attachments to AST
nodes.

## Comment preservation strategy

The wat tokenizer (in `wat-rs/src/`) currently strips `;;` line
comments before parsing — standard Lisp tokenizer behavior. For
wat-fmt, we need comments to survive through to format-time.

Two implementation choices:

**Choice A — Add a comment-preserving tokenizer variant.** The
existing tokenizer keeps its strip-comments mode; wat-fmt
invokes a sibling that emits comment tokens. The parser is
extended to consume comment tokens and attach them to adjacent
AST nodes.

**Choice B — Re-tokenize at format time alongside the AST walk.**
Format-time pipeline opens the source again, tokenizes with
comments, and interleaves them with the AST walk's emission.

Recommendation: **Choice A**. Cleaner. Comments become part of
the structural representation; the AST + comments is a complete
record of the source. The cost is extending the parser; the
benefit is that downstream consumers (wat-lint, future tools)
also get comments-as-structure for free.

## Comment attachment rules

Each comment attaches to one AST node. Three positions:

1. **Leading comment** — `;;` line(s) immediately before a form,
   no blank line between. Attaches to the form. Re-emitted on
   the line(s) before.
2. **Trailing inline comment** — `;;` at end of a form's line.
   Attaches to that form. Re-emitted with two-space gap before
   the `;;`.
3. **Section-break comment** — `;;` line(s) surrounded by blank
   lines. Treated as a top-level form-equivalent; attaches to
   nothing structurally; preserves blank lines around itself.

## Crate location

`wat-rs/crates/wat-fmt/` — workspace member crate alongside the
existing 8 crates:
- `wat-cli`, `wat-edn`, `wat-holon-lru`, `wat-lru`, `wat-macros`,
  `wat-sqlite`, `wat-telemetry`, `wat-telemetry-sqlite`

wat-cli depends on wat-fmt and exposes the CLI surface.

## Crate structure (sketch)

```
wat-rs/crates/wat-fmt/
  Cargo.toml
  src/
    lib.rs              # public API
    tokenizer.rs        # comment-preserving tokenizer (or thin
                        # wrapper around wat-rs/src/parser)
    parser.rs           # AST builder with comment attachment
    ast.rs              # AST + Comment types (or re-export
                        # wat-rs's HolonAST + Comment extension)
    emitter.rs          # walk AST + emit formatted text
    rules/
      indent.rs         # indentation logic
      special_forms.rs  # let, define, lambda, cond, if, match
                        # (the user-iteration-pending rules)
      collections.rs    # vec, HashMap, HashSet, Bundle
      atoms.rs          # Atom, Bind, type annotations
      comments.rs       # leading / trailing / section-break
                        # placement
    width.rs            # line-length-aware wrapping decisions
  tests/
    golden/             # input.wat + expected.wat pairs
    properties.rs       # round-trip stability + semantic
                        # preservation property tests
```

## Public API

Minimal, total, opinionated:

```rust
pub fn format(input: &str) -> Result<String, FormatError>;

pub enum FormatError {
    Parse(ParseError),    // input wasn't valid wat
    // Format errors are NOT possible — the format rules are
    // total over a parsed AST. Once parsed, formatting always
    // succeeds.
}

// For --check mode:
pub fn is_formatted(input: &str) -> Result<bool, FormatError>;
// equivalent to: format(input).map(|out| out == input)

// For --diff mode:
pub fn format_diff(input: &str) -> Result<String, FormatError>;
// returns unified diff between input and format(input)
```

That's the entire crate API. No configuration struct (per
philosophy below).

## CLI integration

Subcommand pattern (`wat fmt ...`) per user direction. Confirmed
because it opens the path for future siblings: `wat check`
(promoting from the wards' /check skill), `wat lint`, `wat run`,
etc.

```
wat fmt PATH                 # format in place
wat fmt --check PATH         # exit 0 if formatted, 1 if not, 2 if parse error
wat fmt --diff PATH          # show diff vs current; never write
wat fmt --stdin              # read stdin, write stdout
wat fmt PATH/                # recurse into directory (only .wat files)
wat fmt PATH1 PATH2 PATH3    # multiple paths
```

**Exit code contract** (matters for editor plugins later):

- `0` — formatted (or successfully formatted in non-check mode)
- `1` — needs format (in check mode); already-existed-was-wrong
- `2` — parse error (input doesn't tokenize/parse as wat)
- `3` — IO error (can't read/write file)

Editor plugins call `wat fmt --check --stdin` and use exit code
to decide whether to display "needs format" warnings.

## Round-trip properties

Two properties the test suite must enforce:

**Property 1 — Idempotent.**
```
∀ src such that format(src) is Ok:
  format(format(src).unwrap()) == format(src)
```
Running format twice produces the same output as running it
once. Means we can safely run format-on-save, format-in-CI,
format-as-pre-commit-hook without churn.

**Property 2 — Semantic preservation.**
```
∀ src such that format(src) is Ok:
  parse(format(src).unwrap()) == parse(src)
```
Formatting never changes meaning. The AST after format equals
the AST before format. Comments that were attached to specific
AST nodes stay attached to the same AST nodes (modulo their
re-emission per the placement rules).

These get property-test coverage via golden files and
randomized AST → emit → parse round-trips.

## Configuration philosophy

**Start with NO configuration.** One canonical style. Per
proposal 058's discipline (ship only what's earned by cited
use), configuration earns its way in only when a real consumer
can articulate WHY their case can't use the canonical style.

Predictions:
- Line length is the most likely first config (user already
  noted "120 for now... it'll be longer with reasons")
- Special-form indentation might want config if multiple
  competing house styles emerge
- Comment placement might want config (some prefer trailing,
  some prefer leading)

But none of these ship in v1. wat-fmt v1 = one style, one set
of rules, no knobs. The simplest tool to integrate (no config
files, no flags-that-affect-output, no subtle differences
across users).

## Error model

Formatting is a total function over parsed wat. The only failure
mode is "input didn't parse." Three principles:

1. **Parse errors point at the source location.** wat-fmt
   inherits the existing parser's diagnostic quality; it doesn't
   add new error types.

2. **No partial output.** If format fails, no file is written
   in `--write` mode. The original is untouched.

3. **`--diff` mode shows nothing on parse error.** Just the
   parse diagnostic. No partial diff against half-parsed input.

## Performance

Not a concern for v1. Format is a rare operation (on save, on
commit, on PR). Correctness matters more than speed. Format the
entire file at once; no incremental formatting. If a 10k-line
file takes 100ms to format, that's fine.

If/when performance matters, the natural hot path is: skip
files that already pass `--check` (fast hash compare), only
re-format on AST diff.

## Why Rust, not wat itself

wat is homoiconic; HolonAST is closed under itself; in
principle wat-fmt could BE a wat program over wat. Tempting.

Rejected for v1:

- The parser is in Rust; format-time benefits from
  parser-internal access (token positions, error spans, etc.)
- wat doesn't yet have a String pretty-printing primitive of
  the shape this needs; would have to build that first
- wat-cli is Rust; integrating a Rust crate is one less FFI
  boundary

Could revisit later: if wat grows a pretty-printer primitive
naturally (for some other reason), wat-fmt v2 might collapse
into wat code. v1 ships in Rust.

## What goes into wat-rs proper vs wat-fmt

Stays in wat-rs proper:
- The standard tokenizer (strip-comments mode)
- HolonAST definition
- Type checker
- Evaluator

Lives in wat-fmt:
- Comment-preserving tokenizer variant (could move to wat-rs
  later if other consumers need it)
- Comment attachment logic
- Format rules
- Per-node emitters

If wat-lint is the next thing, it likely wants the
comment-preserving tokenizer too — at that point, the variant
graduates to wat-rs proper as a parser mode.

## Open architectural questions

A. **Where does the comment-preserving tokenizer live initially?**
   In wat-fmt (cleanest if it's the only consumer for now), or
   directly in wat-rs (cleanest if wat-lint will use it
   imminently). Decision: start in wat-fmt; promote to wat-rs
   when wat-lint is opened.

B. **Token-level format directives?** Some formatters honor
   things like `;; @format-off` / `;; @format-on` to skip a
   region. Probably YES eventually, but NOT in v1. Add when a
   real consumer needs to preserve hand-formatted ASCII art or
   unusual layouts.

C. **What about `(comment ...)` form-as-AST-node?** OG wat had
   this; current wat doesn't seem to (as a substrate primitive).
   If `(comment ...)` ships as a real form, wat-fmt's comment
   attachment logic gets simpler — comments become regular AST
   nodes with their own format rule. Open question; not blocking.

D. **How aggressive about line-wrapping vs leaving long lines?**
   wat-fmt should wrap at 120 cols. But what about a 121-col
   line that wraps awkwardly vs a 130-col line that stays
   readable? Strict (always wrap at 120) vs permissive (wrap if
   the wrap reads well). Recommendation: **strict** — formatters
   should be predictable; readability concerns are rule-design
   problems, not format-time decisions.

## What's NOT in scope

- **Linting** — separate tool (wat-lint, slice 5 / next arc).
  wat-fmt fixes formatting; wat-lint flags suspect patterns.
- **Refactoring** — wat-fmt doesn't rename, restructure, or
  reorganize. Format-time is for whitespace/layout only.
- **Auto-fix for wat-lint warnings** — wat-lint may eventually
  have an `--fix` mode that calls into wat-fmt's emitter for
  the formatting half, but the lint rules and the format rules
  stay separate.
- **Format-on-comment-only changes** — if the AST is identical
  but the formatter would re-emit comments differently, that's
  a no-op or a comment-specific edit, not a "reformat the whole
  file" operation. v1: just reformat the file. Optimize later.
