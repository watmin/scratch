# wat-doc — DESIGN

Architecture: substrate change for first-class docstrings,
extractor crate, four output formats, cross-references, wat-lint
integration. The four questions applied throughout.

Specific output-format details (HTML theme, Markdown flavor,
EDN schema specifics, code-example syntax) are TO BE REFINED
via chat (per user's "C after A" sequencing). This document
locks the load-bearing architecture; details follow.

---

## The four questions are the design compass

Per the established discipline (carried from wat-fmt / wat-lint /
wat-cov arcs):

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape
  per concept.
- **Honest?** What's named matches what's there; docs cannot
  drift from forms because they live INSIDE the forms.
- **Good UX?** A user can do the right thing without ceremony.

Obvious + Simple + Honest must hold before Good UX is even
considered.

## The load-bearing decision — docstrings are first-class

User direction (verbatim):

> *"we could just add doc strings as first class citizens in our
> forms.... that's probably the honest thing to do?...."*

Through the four questions:

**Honest?** ✅✅ — load-bearing argument. A comment-above-form
is a TOOLING CONVENTION; a docstring-in-form is a SUBSTRATE
GUARANTEE. The doc IS part of the form's structure — it cannot
drift from the form because it lives inside the form. The
substrate type-checks it. Two parsers (with/without comment
preservation) see the same thing. Honest by construction.

**Simple?** ✅ slight edge — adds one tiny substrate concept
("string in position N is the docstring"), but eliminates the
entire comment-attachment machinery wat-doc would need from
comments-above. Net: simpler tooling.

**Obvious?** ✅ slight edge — the form's shape literally tells
you "this slot is for documentation." Comments-above requires
knowing the convention.

**Good UX?** wat is a Lisp; its paradigm family is Clojure /
Scheme / CL, not Rust / C. The host-language tiebreaker (which
normally points to Rust) does NOT apply here — wat's
grammatical lineage (per latin-in-wat: morphology over position)
says the form should carry its own meaning. Docstring-IN-form
is morphological. Docstring-as-comment is positional.

**Decision:** docstring as first-class.

## The substrate change

**Three-arg form for `define` / `lambda` / `defmacro`** —
backwards-compatible (existing 2-arg forms still valid):

```scheme
;; Old shape (still valid; backwards-compat):
(:wat::core::define (sig) body)

;; New shape (optional docstring between signature and body):
(:wat::core::define (sig) "Doc string." body)
```

### Concrete examples

**`define` with docstring:**
```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Compute the thing. First sentence is the summary.

   Long-form explanation continues across lines per Rule 31."
  (:wat::core::* x x))
```

**`lambda` with docstring:**
```scheme
(:wat::core::lambda
  ((x :T)
   -> :U)
  "Doc string for this anonymous function."
  body)
```

**`defmacro` with docstring:**
```scheme
(:wat::core::defmacro
  (:my-macro
    (x :AST<T>)
    -> :AST<U>)
  "Doc string. Explains what the macro expands to."
  `(template ,x))
```

### Type-checker handling

The type checker's recognition rule:

```
A `define` / `lambda` / `defmacro` form has either:
- 2 args: (sig body)
- 3 args: (sig docstring body)
  where docstring is :wat::core::String

If 3 args and the middle is NOT a String, type error.
If 3 args and the middle IS a String, the docstring slot is
recognized; the body is args[2].
```

No runtime semantics impact. The docstring is metadata; the
runtime evaluator ignores it. Only tooling (wat-doc, wat-lint)
reads the docstring slot.

### Why between signature and body, not inside the signature

Three reasons:

1. **Signature stays purely about types.** Cleaner separation
   between "what the function takes/returns" and "what it
   does."
2. **Lambda symmetry.** Lambda has no name in the signature;
   placing the docstring outside the signature works
   identically for define/lambda/defmacro. Inside the signature,
   lambda would have an awkward "first arg vs first docstring"
   ambiguity.
3. **The docstring documents the WHOLE form, not just the
   signature.** It belongs between sig and body — adjacent to
   both — rather than nested inside one or the other.

### Why optional, not required

Backwards compatibility: existing wat code without docstrings
keeps working. Adoption is gradual; users add docstrings where
they help; wat-lint can later require docstrings on public
forms (see lint integration below).

## STYLE-RULES.md amendment cascade

Three rules in the wat-fmt arc need amendments:

### Rule 14 (`:define`) — add docstring slot

After the signature, before the body, the docstring (if
present) goes on its own line indented 2:

```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Docstring on its own line indented 2."
  body)
```

Multi-line docstrings follow Rule 31 (atomic; preserved
verbatim):

```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Docstring summary.

   Long-form explanation continues here. The continuation
   lines are part of the string content per Rule 31."
  body)
```

### Rule 14b (`:lambda`) — same pattern

```scheme
(:wat::core::lambda
  ((x :T)
   -> :U)
  "Lambda docstring."
  body)
```

### Rule 14c (`:defmacro`) — same pattern

```scheme
(:wat::core::defmacro
  (:my-macro (x :AST<T>) -> :AST<U>)
  "Macro docstring."
  `(template ,x))
```

These amendments are SMALL. The rest of each rule (signature
shape, body indent, vertical principle) is unchanged.

## wat-doc crate architecture

Same self-contained pattern as the other foundation crates:

```
wat-rs/crates/wat-doc/
  Cargo.toml                       # depends on:
                                   #   wat (path = "../..")
                                   #   wat-macros (path = "../wat-macros")
                                   #   wat-fmt (path = "../wat-fmt")
                                   #     -- shared parser + comment access
                                   #   wat-edn (path = "../wat-edn")
                                   #     -- for --json output
  src/
    lib.rs                         # public Rust API:
                                   #   document(input: &str) -> Result<DocTree, DocError>
                                   #   document_crate(...) -> ...
                                   # PLUS arc-013 contract:
                                   #   wat_sources() / register()
    extract.rs                     # AST walker; pulls (form,
                                   #   signature, docstring) triples
    cross_ref.rs                   # FQDN resolution; link generation
    formatters/
      edn.rs                       # EDN schema for the doc tree
      markdown.rs                  # Markdown emission
      html.rs                      # HTML site (codox-style)
      json.rs                      # via wat-edn
    invoke.rs                      # wat-vm bridge
  wat/
    doc/
      extract.wat                  # wat-coded extraction filters
                                   #   (which forms count as public)
      cross-ref.wat                # wat-coded cross-ref rules
      filters.wat                  # include/exclude patterns
  wat-tests/
    doc/                           # wat-level tests
  tests/
    test.rs                        # Rust harness
    golden/                        # input.wat + expected outputs
                                   #   (one per output format)
```

## Output formats

Four formats. Same EDN data model under the hood; different
renderers.

### EDN (canonical)

Same developer-first principle: structured data; humans can
read; machines can parse.

```edn
{:doc-tree
 {:crate "wat-fmt"
  :modules
  [{:fqdn ":wat::fmt::format"
    :kind :function
    :signature "(format (ast :HolonAST) -> :String)"
    :docstring "Format the given AST per the canonical wat-fmt rules."
    :file "wat/fmt/format.wat"
    :line 12
    :cross-refs [":wat::holon::HolonAST" ":wat::core::String"]}
   ...]}}
```

### Markdown

Embeddable in README files; per-form sections; cross-references
as Markdown links. Suitable for GitHub-rendered project docs.

### HTML

Browseable static site (codox-style):
- Index page with crate navigation
- Per-module pages
- Per-form sections with signature + docstring + cross-references
- Searchable (client-side; no server)
- Cross-references are working links

Templating: askama or tera (Rust templating). Asset bundling:
minimal (single CSS file, no JS framework).

### JSON

Via wat-edn. Same data; ecosystem-friendly format. For
non-wat tooling (e.g., a JS-based docs viewer).

## Cross-reference resolution

When a docstring mentions an FQDN like `:wat::holon::Bind`, the
formatter renders it as a link to that form's documentation
page.

**Mechanism:**
1. Parse the docstring text; identify FQDN-shaped tokens
2. Look up each FQDN in the doc tree
3. If found, render as link in HTML / Markdown
4. If not found, render as plain text + warn (potentially a
   wat-lint rule: `documentation/broken-cross-ref`)

**FQDN detection:** a token starting with `:` and matching
`:[a-zA-Z_][a-zA-Z0-9_]*(::[a-zA-Z_][a-zA-Z0-9_]*)*` is an
FQDN candidate. Walk the regex; check against the doc tree.

**Code-example syntax in docstrings:** TBD (refine via C). Two
candidates:
- Markdown-style fenced code blocks: ` ```scheme ... ``` `
- Indented code blocks (4-space indent)

The Markdown style is more familiar; lean toward it.

## CLI integration

Same wat-cli subcommand pattern:

```
wat doc <crate-or-path>            # generate docs (default: edn)
wat doc <path> --markdown          # emit Markdown
wat doc <path> --html              # emit HTML site
wat doc <path> --json              # emit JSON via wat-edn
wat doc <path> --output ./docs/    # output directory
wat doc --check <path>             # exit nonzero on missing
                                   # docstrings (uses wat-lint)
```

Exit code contract:
- `0` — success
- `1` — missing docstrings detected (in `--check` mode)
- `2` — parse / runtime error
- `3` — IO error

## wat-lint integration

New lint rule:

### `documentation/missing-public-docstring`

**What it counts:** for each public form (`:define` / `:defmacro`
/ `:typealias` / `:struct` / `:enum` / `:lambda`-as-let-binding),
is a docstring present?

**Threshold:** any public form without a docstring → L2
candidate (configurable to L1 via `wat-lint.edn`).

**Rune categories:**
- `documentation(self-evident)` — name + signature is
  sufficient documentation; no docstring needed
- `documentation(transitional)` — code is in active migration;
  docstring will come in a follow-up

**Finding shape:**
```edn
{:rule "documentation/missing-public-docstring"
 :severity :L2-candidate
 :file "wat/foo.wat"
 :line 12
 :context {:form-name ":wat::foo::bar"
           :form-kind :define}
 :message ":wat::foo::bar has no docstring"
 :hint "add a docstring as the second arg to define"}
```

### `documentation/broken-cross-ref` (future)

Flags FQDN mentions in docstrings that don't resolve to a form
in the doc tree. Helps catch typos and stale references.

## Backwards compatibility

The substrate change is OPT-IN:

- Existing 2-arg forms (`(define sig body)`) continue to work
  unchanged
- Code without docstrings doesn't break; just generates docs
  with no description (or wat-lint flags as missing)
- Migration is gradual: add docstrings where they help; existing
  comments-above stay as ordinary comments

**Transitional behavior:** wat-doc CAN read leading `;;`
comments as fallback documentation when no docstring exists.
This eases migration. Rule of precedence:

1. If docstring slot is filled → use docstring
2. Else if leading `;;` comment block exists → use comment
3. Else → no documentation; wat-lint flags as missing

The fallback is transitional; long-term, the docstring slot is
the canonical home and comments-above are ordinary comments
(not documentation).

## Per-crate vs project-wide doc trees

**Per-crate** (default): each crate's documentation is a
self-contained tree. Cross-references within the crate work;
cross-references to other crates link to those crates' docs
if they exist; otherwise plain text.

**Project-wide composition** (future): when a project depends
on multiple crates, wat-doc can compose them into one unified
doc site. Like Rust's `cargo doc --workspace`. Defer to v2 or
slice 4 polish.

## Performance

Documentation generation is rare (build-time, not runtime).
Performance is not a concern for v1. Format the entire doc tree
at once; no incremental docs.

If future demand surfaces (large multi-crate workspaces with
slow doc builds), the natural optimization:
- Skip crates whose source hasn't changed since last doc build
- Cache the EDN doc tree per crate; only re-render formatters

## What goes into wat-rs proper vs wat-doc

**`wat-rs/src/`** (Rust runtime + parser + type checker):
- Add 3-arg form recognition to define / lambda / defmacro
- Type checker: optional String at position [1]; reject
  non-String at that position
- No runtime semantics changes (docstring is metadata)

**`wat-rs/crates/wat-doc/`** (self-contained crate):
- All extraction logic (Rust shim + wat code)
- All format emitters
- Cross-reference resolution
- CLI integration

**`wat-rs/crates/wat-fmt/`** (sibling — small amendment):
- Rule 14 / 14b / 14c amended in STYLE-RULES.md
- Per-form emitters in slice 1 add docstring rendering
  (one if-check per affected form)

**`wat-rs/crates/wat-lint/`** (sibling — adds new rule):
- `documentation/missing-public-docstring` rule
- `documentation/broken-cross-ref` rule (future)

## Open architectural questions

Three flagged for slice-time decisions or chat refinement (per
"C after A"):

A. **Code-example syntax in docstrings.** Markdown fenced code
   blocks (` ```scheme `) vs indented (4-space) blocks. Lean
   Markdown for familiarity; confirm via C.

B. **HTML site theme + navigation.** Codox provides a
   functional default; wat-doc could match that or design our
   own. Defer to slice 3 implementation.

C. **Multi-crate doc composition.** Per-crate (default) is
   slice 1-3; project-wide composition is slice 4 polish or
   v2.

## What's NOT in scope

- **Substrate-level documentation requirements.** wat-lint
  flags missing docstrings; the substrate doesn't require them.
  Enforcement is configurable per project.
- **Tutorial / guide content.** wat-doc generates API reference;
  long-form tutorials are written by humans in Markdown / HTML
  separately.
- **LLM-generated documentation.** Per the LLM-out discipline,
  wat-doc emits structured data; LLM-generated improvements
  are downstream tooling the user invokes explicitly.
- **Cross-language docs.** wat-doc documents wat. Rust
  documentation comes from rustdoc.
