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

## Crate location and language decision

**Updated 2026-05-02 — wat-fmt is wat code, not Rust, and the
crate is fully self-contained.**

User direction (full reasoning in "Why wat, not Rust" below):
wat-fmt is implemented in wat itself with a minimal Rust shim
for parser invocation and CLI integration. RuboCop is the model
— the linter/formatter for a language is itself written in
that language, enabling user-extensibility and shared
infrastructure with downstream tools (wat-lint).

**Single self-contained location:** `wat-rs/crates/wat-fmt/`.

Both the Rust shim AND the wat code live inside the crate.
Other crates (wat-cli, wat-lint, user crates) depend on
wat-fmt; the wat code travels with the dependency. Users who
want wat-cli get the formatter for free as a transitive
dependency.

**Self-contained matters because:**
- The crate IS the deliverable. No "wat-fmt depends on wat
  files in a sibling directory" coupling.
- Distribution is one Cargo unit. `cargo add wat-fmt` is enough.
- The wat code is a private implementation detail of the
  crate; consumers see only the Rust + wat APIs that the
  crate exports.
- Other downstream crates (wat-lint, future wat-extending
  tooling) depend on wat-fmt as a unit, not on a scattered
  set of wat files.

## Crate structure (sketch)

```
wat-rs/crates/wat-fmt/
  Cargo.toml
  src/                             # Rust shim (minimal)
    lib.rs                         # public Rust API:
                                   #   format(input: &str) -> Result<String>
                                   # parses input, loads/embeds the
                                   # wat code, invokes :wat::fmt::format
                                   # on the AST via the wat-vm, returns
                                   # the formatted string
    invoke.rs                      # wat-vm invocation helpers (the
                                   # bridge from Rust input/output to
                                   # wat function calls)
    parser_ext.rs                  # comment-preserving parser variant
                                   # (Rust; lives next to lexer/parser)
    embed.rs                       # include_bytes! of the wat/ tree
                                   # so the crate is a single binary
                                   # unit; no runtime filesystem
                                   # dependency
  wat/                             # the actual formatter (wat code)
    format.wat                     # top-level entry:
                                   #   (:wat::fmt::format ast) -> :String
    rules/
      define.wat                   # Rule 14 implementation
      lambda.wat                   # Rule 14b
      defmacro.wat                 # Rule 14c
      let-star.wat                 # Rule 13
      conditional.wat              # Rule 16 (if / cond / match)
      try.wat                      # Rule 19 (Result/try, Option/try)
      expect.wat                   # Rule 19b
      vec.wat                      # Rule 20
      bundle.wat                   # Rule 21
      hashmap.wat                  # Rule 22
      hashset.wat                  # Rule 22b
      symbols.wat                  # Rules 23, 24 (FQDN, type sigils)
      type-annotations.wat         # Rules 25, 26
      literals.wat                 # Rules 27, 28, 29
      quasiquote.wat               # Rule 30
      multiline-string.wat         # Rule 31
    primitives/
      indent.wat                   # indent computation (column-aware)
      width.wat                    # line-length tracking; wrap decisions
      comment.wat                  # leading / trailing / section-break
      string-builder.wat           # string concat / padding helpers
    ast/
      walk.wat                     # generic AST walker
      inspect.wat                  # AST inspection helpers
                                   # (head?, args, has-form?, etc.)
  tests/
    golden/                        # input.wat + expected.wat pairs
                                   # (one per Rule's canonical example)
    properties.rs                  # round-trip + semantic-preservation
                                   # property tests via the Rust API
```

**How the wat code reaches the wat-vm:**

The Rust shim uses `include_bytes!` (or equivalent) at compile
time to embed the entire `wat/` tree into the crate binary. At
runtime, when `wat_fmt::format` is called, the shim:

1. Initializes a wat-vm instance
2. Loads the embedded wat code into the wat-vm's symbol table
3. Parses the input via wat-rs's parser (with the comment-
   preserving variant)
4. Invokes `:wat::fmt::format` on the resulting AST
5. Returns the formatted String to the Rust caller

No filesystem dependency at runtime. The wat code travels with
the Rust crate as compiled bytes; the crate is one shippable
unit.

**Each rule file is one or two wat functions** that take a
HolonAST node and return a `:wat::core::String` (or compose with
the parent rule's emitter). Easy to navigate; easy to extend;
each rule's wat code IS its specification (the canonical
example in STYLE-RULES.md becomes a test case).

**The wat-vm runs the formatter.** When `:wat::fmt::format` is
invoked from the Rust shim, the wat-vm walks the AST in wat
code, applies the per-form rules, builds the output string. The
wat-vm interpretation cost is acceptable for v1 (format is rare;
correctness > speed); v2 may cache or compile hot paths if
performance matters.

## Dependency story

```
wat-rs (parser, wat-vm, type checker)
  ↑
wat-rs/crates/wat-fmt (Rust shim + embedded wat code)
  ↑                  ↑
wat-cli              wat-lint (when it ships)
                     (depends on wat-fmt; can call :wat::fmt::*
                      from its own wat code; gets the embedded
                      wat code via wat-fmt's transitive presence
                      in the wat-vm's symbol table)
```

Users who depend on wat-cli get wat-fmt as a transitive
dependency — the formatter ships with the CLI for free.
Users who want only the formatter as a library depend on
wat-fmt directly. Users who want the linter depend on wat-lint
which itself depends on wat-fmt; both wat-coded surfaces become
available in the wat-vm.

## Public API

**Two surfaces:**

### Rust API (the shim's `lib.rs` exports)

For wat-cli and other Rust consumers:

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

The Rust shim parses input with wat-rs's parser, then invokes
`:wat::fmt::format` on the resulting AST via the wat-vm.

### wat API (what the wat code exposes)

For wat-lint and wat-extending consumers:

```scheme
(:wat::fmt::format
  (ast :wat::holon::HolonAST)
  -> :wat::core::String)

(:wat::fmt::indent-of
  (ast :wat::holon::HolonAST)
  (column :wat::core::i64)
  -> :wat::core::i64)

(:wat::fmt::would-wrap?
  (ast :wat::holon::HolonAST)
  (column :wat::core::i64)
  -> :wat::core::bool)
```

Plus per-rule entry points (e.g., `:wat::fmt::rules::define`,
`:wat::fmt::rules::let-star`) that downstream tools can call
to ask wat-fmt about specific form's layout decisions.

That's the entire API. No configuration struct (per philosophy
below).

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
entire file at once; no incremental formatting.

**With the wat-not-Rust pivot, performance budget is wider:**
- Pure-Rust formatter: ~5-20ms for a 10k-line file
- wat-coded formatter via wat-vm interpretation: 5-50x slower
  is plausible (rough order of magnitude; would need real
  benchmarks)
- Target: ≤ 200ms for a 10k-line file on save. Above that,
  optimize.

If/when performance matters, the natural optimization paths:
- Skip files that already pass `--check` (fast hash compare);
  only re-format on AST diff
- Memoize per-form output keyed on AST hash (most reformats
  re-emit identical sub-trees)
- Compile hot wat rules to Rust at build time (v2 only; not
  v1)

## Why wat, not Rust

**Decision flipped 2026-05-02.** The earlier draft of this
section argued for Rust; user pushed back via the wat-lint
implication. RuboCop is the model: a Ruby linter/formatter is
itself written in Ruby because that maximizes language-shared
machinery between the linter and its target language.

For wat the alignment is even cleaner because of homoiconicity
— wat IS the AST language by construction; you don't need
reflection, you just `match` on `HolonAST`.

**Why this wins:**

1. **wat-lint can compose with wat-fmt's primitives directly.**
   Lint rules ask "would this wrap?", "what's the indent here?",
   "is this symbol over the line limit after formatting?" —
   all by calling `:wat::fmt::*` functions natively. No FFI;
   no context switch; same language.

2. **User extensibility.** Drop a custom format / lint rule as a
   `.wat` file in the right place; wat-fmt picks it up. Users
   write wat code to extend wat tooling. The community can
   contribute rules without touching Rust.

3. **Single source of truth.** The 19 STYLE-RULES locked in
   STYLE-RULES.md are realized as wat code in `wat-rs/wat/fmt/`.
   The spec (the rule's English description + canonical example)
   sits next to the implementation (the wat function). No
   spec/code drift.

4. **Aligns with substrate philosophy.** wat's whole pitch is
   "wat is the meta-language for everything wat." Implementing
   wat-fmt in Rust would have hardcoded one piece of the wat
   ecosystem in a non-wat language. wat-fmt-in-wat keeps the
   meta-language pure.

5. **Strange loop.** wat code that reformats wat code, called by
   the wat-vm, parsed by wat-rs's parser. The whole pipeline is
   wat-aware.

**Substrate primitives the formatter composes from** (verified
2026-05-02):

- String building: `:wat::core::string::concat`,
  `:wat::core::string::join`, `:wat::core::show`
- String inspection: `:wat::core::string::starts`, `::ends`,
  `::contains?`, `::length`, `::trim`, `::split`, `::to`
- AST inspection / walking: `:wat::core::first` / `:second` /
  `:third` / `:rest`, `:length`, `:filter`, `:map`,
  `:foldl`, `:foldr`
- AST pattern matching: `:wat::form::matches?` (per arc 098)
- HolonAST: closed under itself; every AST node IS a HolonAST
- `:wat::core::match` over HolonAST shapes

Plus `:wat::core::eval` and friends for self-introspection.

**What stays in Rust:**

- The bare parser (lexer + AST construction) — the bootstrap
  has to be Rust; you can't parse wat with code that's parsed
  from wat. Already in `wat-rs/src/lexer.rs` and
  `wat-rs/src/parser.rs`.
- The comment-preserving parser variant (extension of the above;
  lives in `wat-rs/crates/wat-fmt/src/parser_ext.rs` initially;
  graduates to `wat-rs/src/` when wat-lint also needs it).
- The Rust API shim (the `lib.rs` that wat-cli calls).
- The wat-vm itself (the runtime that interprets the wat code).

Everything else — the format rules, the indent calculations,
the line-length logic, the comment placement, the
emitter — is wat code in `wat-rs/wat/fmt/`.

**Performance trade-off:** wat-vm is interpreted, so format-time
on big files will be slower than a pure-Rust formatter. v1
accepts this — format is a rare operation; correctness > speed.
v2 might compile hot paths if profiling shows a real bottleneck.

**What might motivate revisiting:** if wat-vm proves too slow
for format-on-save in editors (sub-100ms target on 10k-line
files), v2 could either (a) memoize / cache aggressively, (b)
compile the rules to Rust at build time. Both are downstream
optimizations; the wat code stays the source of truth.

## What goes into wat-rs proper vs wat-fmt vs the wat code

**`wat-rs/src/`** (Rust runtime + parser):
- The standard tokenizer (strip-comments mode)
- HolonAST definition
- Type checker
- wat-vm evaluator

**`wat-rs/crates/wat-fmt/`** (self-contained crate):
- `src/lib.rs` — public Rust API for wat-cli and library consumers
- `src/invoke.rs` — wat-vm invocation helper
- `src/parser_ext.rs` — comment-preserving parser variant
  (Rust; lives here because it's an extension of the wat-rs
  parser; could be promoted to `wat-rs/src/` if a second
  consumer like wat-lint needs the variant directly)
- `src/embed.rs` — `include_bytes!` of the `wat/` tree
- `wat/format.wat` — top-level wat entry point
- `wat/rules/*.wat` — per-form rule implementations
- `wat/primitives/*.wat` — indent / width / comment /
  string-builder helpers
- `wat/ast/*.wat` — AST walking + inspection helpers
- `tests/` — golden files + property tests

**`wat-rs/crates/wat-lint/`** (when it ships — fully isolated
sibling):
- Same self-contained shape as wat-fmt
- Depends on wat-fmt the crate (gets the wat-coded format
  primitives via wat-fmt's embedded wat code; can call
  `:wat::fmt::*` from its own lint rules)
- Has its own `wat/` tree with lint-rule files

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
