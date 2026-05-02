# wat-kwargs — DESIGN

The pattern in detail; concrete examples; the four-questions
analysis; integration points with wat-fmt / wat-lint / wat-doc;
the helper macro that reduces the boilerplate.

---

## The four questions are the design compass

Per the established discipline:

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape
  per concept.
- **Honest?** What's named matches what's there; the macro
  doesn't pretend to be a function.
- **Good UX?** A user can do the right thing without ceremony.

For this pattern: trades small "Obvious / Simple" cost for big
"Good UX" win. Net positive when the function has more than
~3 args or when args have similar types that could be confused
positionally.

## The pattern

### Layer 1 — typed positional substrate function

```scheme
(:wat::core::define
  (:wat::remote::Program/remote-mtls
    (host        :String)
    (port        :i64)
    (client-cert :Path)
    (client-key  :Path)
    (ca-cert     :Path)
    -> :Result<:Program<:I, :O>, :ConnectError>)
  "The typed substrate for constructing an mTLS remote program.
   Positional args; types declared in signature.
   See :wat::remote::Program/with-mtls for the kwarg surface
   that human consumers should prefer."
  body)
```

Standard typed function. Types declared. Substrate type checker
enforces correctness. Positional args at the substrate level.

### Layer 2 — kwarg macro surface

```scheme
(:wat::core::defmacro
  (:wat::remote::Program/with-mtls & (kwargs :AST<Vector<HolonAST>>)
    -> :AST<HolonAST>)
  "Construct an mTLS remote program with keyword arguments.

   Kwarg-shaped surface for :wat::remote::Program/remote-mtls.
   Required keys: :host :port :client-cert :client-key :ca-cert.

   Example:
     (:wat::remote::Program/with-mtls
       :host        \"api.example.com\"
       :port        443
       :client-cert cert
       :client-key  key
       :ca-cert     ca-cert)"
  ;; macro parses kwargs vec into positional args
  (:wat::kwargs::expand-kwargs
    :wat::remote::Program/remote-mtls
    [:host :port :client-cert :client-key :ca-cert]
    kwargs))
```

Variadic macro captures user-provided keyword-arg pairs;
expands to a positional call to the typed function.

### Consumer site

```scheme
(:wat::remote::Program/with-mtls
  :host        "api.example.com"
  :port        443
  :client-cert cert
  :client-key  key
  :ca-cert     ca-cert)
```

Clean. Self-documenting. Positional swaps prevented by the
slot names. No types visible. The substrate enforces correctness
via the underlying typed function's signature.

## The macro implementation pattern

The macro must:
1. Accept variadic input as `& (kwargs :AST<Vector<HolonAST>>)`
2. Walk the kwargs vec extracting `[:keyword, value, :keyword, value, ...]` pairs
3. Look up each expected keyword's value in the pairs
4. Build the positional call in the function's declared parameter order
5. Emit the positional call as the macro's expansion

**Failure modes to handle:**
- Missing required keyword → macro-time error pointing at the call site
- Extra unknown keyword → macro-time error or warning (pick a discipline)
- Duplicate keyword → macro-time error (last-wins is too quiet)
- Wrong type for a keyword's value → caught by the substrate's type checker
  on the expanded form (no special handling needed at macro time)

Per the four questions on the macro's diagnostics:
- **Obvious?** ✅ — error messages name the keyword that's wrong
- **Honest?** ✅ — surface errors point at the user's call site, not the
  expansion
- **Good UX?** ✅ — explicit "missing required keyword `:client-cert`"
  beats a positional "wrong number of args"

## Auto-generation — `:wat::kwargs::auto-kwargs`

**Updated 2026-05-03 per user direction:**

> *"we could make this completely generic... we could have a
> macro who reads in a function's def and creates a kwarg
> variant for the user?... the func's form is fully specified?.."*

YES. wat is homoiconic; the function's definition IS data; a
macro can introspect it; the kwarg variant is derivable. The
function's signature is the single source of truth; the kwarg
variant is a projection of that source.

**The shape — one line per API:**

```scheme
;; Define the typed function normally
(:wat::core::define
  (:wat::remote::Program/remote-mtls
    (host :String)
    (port :i64)
    (client-cert :Path)
    (client-key :Path)
    (ca-cert :Path)
    -> :Result<...>)
  "Construct an mTLS remote program (typed positional API)."
  body)

;; In a follow-up form, auto-generate the kwarg variant from the
;; function's signature
(:wat::kwargs::auto-kwargs :wat::remote::Program/remote-mtls)
```

**That's it.** The macro:
1. Looks up the function's signature at macro-expansion time
2. Extracts parameter names + order from the signature
3. Auto-derives the macro name (`with-remote-mtls` per the
   `with-X` naming convention, or configurable)
4. Auto-generates the kwarg variant's docstring (referencing
   the underlying function's docstring)
5. Emits the kwarg macro definition

**Always-in-sync:** if the function's signature changes (a
parameter added, removed, renamed), the kwarg variant updates
automatically on next re-expansion. The two surfaces literally
cannot drift because they share one source of truth.

### Per the four questions on auto-generation

This is strictly better than the explicit-list form across all
four questions:

- **Obvious?** ✅✅ — one declaration; intent unmistakable;
  the underlying function's signature is what you're projecting
- **Simple?** ✅✅ — eliminates boilerplate; ONE macro for the
  whole ecosystem; no per-API ceremony
- **Honest?** ✅✅✅ — the function's signature IS the contract;
  the macro just projects it; **impossible** for the two
  surfaces to drift
- **Good UX?** ✅✅ — adopters drop in one line; opting INTO
  the pattern is trivial; opting OUT is just deleting that line

The explicit-list form had ⚠️ "Simple slight cost"; auto-generation
is ✅✅ "Simple win." Strictly stronger pattern.

### Substrate prerequisite — macro-time signature lookup

`:wat::kwargs::auto-kwargs` requires ONE substrate primitive
that may or may not exist today:

```scheme
(:wat::core::sig-of <fn-name>) -> :wat::core::SignatureAST
```

Returns the function's signature as a HolonAST that the macro
can inspect (parameter names, types, order). Per wat being
homoiconic + HolonAST closed-under-itself, this is a natural
primitive — the function's signature is already an AST node;
expose it for inspection.

**Status of this primitive:** TBD. To verify in slice 1.
- If it exists today: auto-kwargs ships immediately
- If it doesn't: small substrate addition; earns its keep via
  this pattern (and would unlock other introspection-based
  macros)

### Override hooks for the auto-derived shape

`auto-kwargs` derives reasonable defaults; users can override:

```scheme
;; Default: macro name derived from function name (with-mtls);
;; default docstring referencing the underlying function
(:wat::kwargs::auto-kwargs :wat::remote::Program/remote-mtls)

;; Override: explicit macro name (e.g., when default doesn't fit)
(:wat::kwargs::auto-kwargs :wat::remote::Program/remote-mtls
  :as :wat::remote::Program/connect-mtls)

;; Override: explicit docstring
(:wat::kwargs::auto-kwargs :wat::remote::Program/remote-mtls
  :doc "Custom docstring overriding the auto-derived one.")

;; Both overrides
(:wat::kwargs::auto-kwargs :wat::remote::Program/remote-mtls
  :as :wat::remote::Program/connect-mtls
  :doc "Custom docstring.")
```

The overrides themselves use kwarg-style — eating our own dog
food.

## The lower-level primitive — `:wat::kwargs::define-with-kwargs`

The explicit-list form remains as the lower-level primitive
that `auto-kwargs` generates calls to. Useful in two cases:

1. When the function isn't available at macro-expansion time
   (e.g., dynamically-resolved or generated)
2. When the auto-derived parameter names need overriding (rare;
   the override hooks above usually cover it)

```scheme
(:wat::kwargs::define-with-kwargs
  :wat::remote::Program/with-mtls           ; macro name to define
  :wat::remote::Program/remote-mtls         ; underlying typed function
  [:host :port :client-cert :client-key :ca-cert]   ; kwargs in positional order
  "Custom docstring.")
```

Mostly invisible. `auto-kwargs` is the user-facing tool;
`define-with-kwargs` is the implementation under it.

## When to adopt vs not adopt

### Adopt the pattern when:
- The function has 4+ args
- Multiple args have the same type (could be positionally swapped)
- The function is intended for direct human use (not deeply nested
  in computation)
- The args are configuration-shaped (constructors, builders,
  options-takers)

### Keep positional when:
- The function is a binary operator (`+`, `-`, `cosine`)
- The function is a simple unary call (`first`, `length`)
- The function is internal-only (rare human callers)
- The args are small in count AND distinct in type (3 args, all
  different types — positional is fine)

Adoption is at the consumer crate's discretion. The pattern
doesn't FORCE itself on every multi-arg function; it's a tool
to reach for when call-site readability matters.

## Naming convention

Per the four questions (Obvious): the relationship between the
two surfaces should be visible from the names.

**Convention:** the typed substrate function is `<base-name>`;
the kwarg macro is `with-<base-name>` or similar prefix.

Examples:
- `Program/remote-mtls` (typed) ↔ `Program/with-mtls` (kwarg)
- `Server/start` (typed) ↔ `Server/with-start` (kwarg)
- `Channel/configure` (typed) ↔ `Channel/with-configure` (kwarg)

The `with-` prefix tells the reader: this is the kwarg-style
constructor; the underlying typed surface lives without the
prefix. Macro's docstring confirms the relationship explicitly.

Alternative conventions discussed (TBD per slice 1 lock):
- `make-X` for the kwarg form, `X` for the typed form
- `X-with-options` for the kwarg form
- Whatever feels cleanest in real wat code

## Per the four questions — full analysis

### Obvious

**⚠️ slight cost** — readers see a macro call; need to know it
expands to a typed function. **Mitigations:**
- Naming convention (with-X) makes the macro's role visible
- Macro's docstring points at the underlying typed function
- wat-doc (slice 4 of arc 006) cross-references both surfaces
- wat-fmt formats kwarg macro calls consistently with struct
  construction (visually similar)

Net: still readable; the slight cost is bounded by good
conventions.

### Simple

**⚠️ slight cost** — two surfaces per API. **Mitigations:**
- Helper macro `:wat::kwargs::define-with-kwargs` reduces
  boilerplate to ~5 lines per API
- Adoption is opt-in per function; only wraps where it pays off
- Both surfaces share one source of truth (the positional
  function's signature)

Net: bounded complexity; simpler than substrate-level keyword
args (which would be cross-cutting work).

### Honest

**✅** — the pattern's discipline IS its honesty:
- Types live in ONE place (the function's signature)
- Both surfaces document their relationship
- The macro is named for what it constructs
- The substrate enforces correctness via the expanded form

The macro doesn't lie about being a function; it's a macro
that constructs a function call. The user's mental model maps
to reality.

### Good UX

**✅✅ load-bearing win:**
- Self-documenting call sites
- Positional swaps for similarly-typed args become impossible
- Fits the "Lispy data-shape" aesthetic that wat already has
- Pattern is teachable (it's just one rule)
- Adopters can stop worrying about long positional argument
  lists

This is where the pattern earns its keep.

## Integration with the foundation tier

### wat-fmt (003)

Need to add a formatting convention for kwarg macro calls.
Probably similar to HashMap construction:

```scheme
;; YES — keys aligned; values follow
(:wat::remote::Program/with-mtls
  :host        "api.example.com"
  :port        443
  :client-cert cert)

;; YES (alt) — no alignment; single space
(:wat::remote::Program/with-mtls
  :host "api.example.com"
  :port 443
  :client-cert cert)
```

Per Rule 22's existing decision (no alignment, single space),
the second form is canonical. **Slice 3 of THIS arc** adds the
amendment to STYLE-RULES.md.

### wat-lint (004)

Optional rule: `kwargs/long-positional-arg-list` flags
functions with N+ args where M+ have the same type, suggesting
a kwarg macro wrapper.

**Default severity: L3-candidate** (informational; not
prescriptive). The user decides whether to adopt the pattern;
the lint just surfaces the opportunity.

**Rune categories:**
- `kwargs(intentional-positional)` — function is an internal
  helper or has a natural positional shape (e.g., point
  coordinates `(x, y, z)`)

**Slice 4 of THIS arc** adds the rule to wat-lint.

### wat-doc (006)

Both surfaces (macro + underlying function) get docstrings per
the wat-doc convention. The macro's docstring SHOULD reference
the underlying function's docstring; wat-doc's cross-reference
resolution links them.

**Recommended docstring shape for kwarg macros:**

```
"<one-line summary of what the macro constructs>

 Kwarg-shaped surface for `:wat::remote::Program/remote-mtls`.
 Required keys: :host :port :client-cert :client-key :ca-cert.

 Example:
   (:wat::remote::Program/with-mtls
     :host \"...\"
     :port 443
     :client-cert ...
     :client-key ...
     :ca-cert ...)"
```

This makes the macro discoverable AND points consumers at the
typed substrate when they need full type information.

## Open architectural questions

Three flagged for slice-time decisions:

A. **Naming convention final lock.** `with-X`, `make-X`, or
   something else? Probably `with-X` per the discussion above
   but lock at slice 1.

B. **Strict vs lax kwarg validation.** Should an unknown kwarg
   trigger a macro-time error (strict) or a warning (lax)?
   Strict feels right but might bite during refactor.

C. **Default values for omitted kwargs.** Should the helper
   macro support per-kwarg defaults? E.g.,
   `[(host) (port 443) (client-cert) ...]` to default `:port`
   to 443 if omitted. Powerful but adds complexity. Maybe v2.

## What's NOT in scope

- **Substrate-level keyword arguments.** This pattern lives at
  the macro layer; it doesn't change the substrate. If
  substrate keyword args ever ship (separate, larger arc), this
  pattern becomes obsolete; until then, it's the pragmatic
  answer.
- **Runtime kwarg dispatch.** All kwarg parsing happens at
  macro-expansion time (compile time). Runtime args are
  positional after expansion.
- **Reflection over kwarg specs.** The helper macro generates
  static expansions; there's no runtime API to query "what
  kwargs does this macro take?" (downstream tooling reads the
  macro's docstring instead).
