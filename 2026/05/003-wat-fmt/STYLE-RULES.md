# wat-fmt — STYLE RULES

Numbered for easy reference. **User: mark up directly. Strike,
revise, replace, add. Each rule is independent.**

Status legend:
- ✅ **CONFIRMED** — user accepted in 2026-05-02 conversation
- ❓ **DRAFT** — assistant proposed; user wants to iterate
- 🔧 **OPEN** — needs decision

---

## §1 — Indentation and whitespace

### Rule 1 ✅ — Two-space indent

```scheme
(:wat::core::let* (((x :wat::core::i64) 42))
  (:wat::core::println x))
;; not 4 spaces; not 1 space; not tabs
```

### Rule 2 ✅ — Closing parens stack at end-of-line

Lisp convention. Never on their own line.

```scheme
;; YES
(:wat::core::let* (((x :T) 1)
                   ((y :T) 2))
  (:wat::core::+ x y))

;; NO — Python-style
(:wat::core::let* (((x :T) 1)
                   ((y :T) 2)
)
  (:wat::core::+ x y)
)
```

### Rule 3 ✅ — Trailing newline at EOF

Always. One final `\n`. Not zero, not multiple.

### Rule 4 ✅ — No trailing whitespace

On any line. Stripped on format.

### Rule 5 ✅ — Tabs never emitted

Always spaces. (Even if input has tabs, output has spaces.)

---

## §2 — Line length

### Rule 6 ✅ — 120-column line limit (for now)

User noted: *"it'll be longer with reasons"* — so 120 is the
starting point; expect this to increase as wat develops more
verbose forms (multi-line type annotations, long FQDNs, etc.).

When a form exceeds 120 columns, wrap per the special-form rules
(§4) or the collection rules (§5).

---

## §3 — Comments

### Rule 7 ✅ — Comments preserved verbatim

Modulo trailing whitespace stripped. Otherwise the comment text
is untouched (no rewrapping, no spell-check, no normalization).

### Rule 8 ✅ — Block comments above the form they document

```scheme
;; Compute the cosine similarity between two HolonAST forms.
;; Returns Ok(:f64) on success; Err on dimension mismatch.
(:wat::core::define (:user::cosine-sim (a :HolonAST) (b :HolonAST)
                                       -> :Result<:f64,:Error>)
  ...)
```

No blank line between the leading comment and its form. They
belong together.

### Rule 9 ✅ — Inline comments require two spaces before `;;`

```scheme
(:wat::core::let* (((x :i64) 42))  ;; the magic number
  ...)
;; not   42);; the magic number   (one space)
;; not   42)         ;; ...        (excess gap)
```

### Rule 10 ✅ — Section-break comments

A comment line surrounded by blank lines is treated as a
section break. Preserves blank lines around it; not attached to
any form.

```scheme
(:wat::core::define (foo) ...)

;; ─── Internal helpers ───────────────────

(:wat::core::define (helper) ...)
```

The `;; ───` style is one possible section-break convention; the
formatter treats ANY isolated comment line (blank lines on both
sides) as a section break. The formatter doesn't enforce the
visual style.

---

## §4 — Top-level forms

### Rule 11 ✅ — One blank line between top-level forms

Default: exactly one blank line between two top-level forms.

```scheme
(:wat::core::define (foo) ...)

(:wat::core::define (bar) ...)
```

### Rule 12 ✅ — Preserve user-intent two-line gaps

If the user wrote two blank lines between forms (signaling a
section break), preserve. Never more than two.

```scheme
(:wat::core::define (foo) ...)

(:wat::core::define (bar) ...)


(:wat::core::define (baz) ...)
```

(The blank-blank-line is preserved; collapsed to two if user
had three or more.)

---

## §5 — Special forms ❓ DRAFT

User: *"we'll work on special forms later... we'll iterate on
those... i am /very/ opinionated."* These are first-pass
proposals; expect heavy markup.

### Rule 13 ✅ — `:let*` is always vertical

User locked 2026-05-02. **Note:** the substrate currently has
only `:let*`; will be renamed to `:let` later. Same rule applies
post-rename.

**Substrate constraints (verified):**
- `:wat::core::let*` takes EXACTLY 2 args: bindings list + body
- One body expression only, not multiple
- Bindings list can be empty (substrate-legal but pointless;
  wat-lint will flag)

**Shape:**
- `:wat::core::let*` keyword alone on line 1
- Bindings vector `(` indented 2 from `let*` `(`
- First binding `((sym :type) value)` shares the line with the
  bindings vector `(`
- Subsequent bindings align at the same column as the first
  binding (one character past the bindings vector `(`)
- Bindings vector closes after the last binding
- Body indented 2 from `let*` `(` (same depth as bindings vector)
- **Always vertical** — no fits-on-one-line exception

**Two-binding canonical:**
```scheme
(:wat::core::let*
  (((some-symbol :some-type) :some-value)
   ((another-value :another-type) :another-value))
  (...body...))
```

**Single binding (sub-rule 13a):**
```scheme
(:wat::core::let*
  (((only-sym :T) value))
  body)
```

Same shape; bindings vector wraps the single binding.

**Wrap rule (Option B locked, sub-rule 13b — wrap value at +2
from binding's `(`):**

When a binding doesn't fit on one line, wrap with the value
indented +2 from the binding's `(`:

```scheme
;; binding ( at column N; typed-param ( at N+1; value at N+2
((some-symbol :some-type)
   some-very-long-value-expression)
```

**Important — this differs from existing wat-rs code.** Files
like `wat/stream.wat` currently wrap values at typed-param's
column (+1 from binding's `(`, not +2). The formatter will
REFLOW all existing wat code to Option B on first run. Expected
diff churn; not a blocker.

**Long FQDN handling (sub-rule 13c, from EQ3):**

Per Rule 23 (FQDN never wrap), if a binding's typed-param has a
very long FQDN, the binding line stays over-length; the
formatter doesn't break the FQDN. Over-length lines are a
LINT signal — they scream *"you need a type alias"* — but
wat-fmt's job is to format faithfully, not to refactor.

```scheme
;; long FQDN; line stays over 120 cols; wat-lint flags it
((handle :wat::some::very::deep::namespace::WithExtremelyLongTypeName) value)
```

**Nested `:let*` (sub-rule 13d, from EQ4):**

Same shape, just deeper indent inherited from the parent
context. A nested `let*` sees its parent's indent as its
"column 0" baseline; everything else applies relative to
that:

```scheme
(:wat::core::let*
  (((outer :T) value))
  (:wat::core::let*
    (((inner :U) (compute outer)))
    (use outer inner)))
```

The inner `let*` follows the same rule; bindings vector
indented 2 from the inner `let*` `(`; body indented 2 from
the inner `let*` `(`.

**Multi-line value bodies (sub-rule 13e, from EQ5):**

When a binding's value is itself a multi-line form (a
`lambda`, another `let*`, a `match`, a long `cond`), the
value follows ITS OWN rule at relative indent. The let* rule
only specifies WHERE the value's first line lands (per the
wrap rule); the value's internal structure follows the value's
form's rule.

```scheme
(:wat::core::let*
  (((handler :T)
     (:wat::core::lambda
       ((x :i64)
        -> :i64)
       (:wat::core::* x 2))))
  body)
```

Here the lambda's first character (its `(`) is at +2 from the
binding's `(` (per Option B), and the lambda itself follows
the lambda rule (Rule 14b — TBD).

**Empty bindings (substrate-legal but pointless):**

```scheme
(:wat::core::let*
  ()
  body)
```

Substrate accepts; equivalent to just `body`. Formatter handles
gracefully (empty bindings vector on its own line); wat-lint
should flag as code-smell when it ships. Same rule as
non-empty case.

**Why this shape:**
- Same "always vertical" discipline as Rules 14 + 16
- Bindings horizontally compact (typed-param + value on one
  line when possible) so the visual scan is "one binding =
  one row"
- Subsequent bindings align via the bindings vector's content
  column, matching standard Lisp idiom
- Body separated from bindings by indent equality (both at
  +2 from let*'s `(`) — visual symmetry between "what we
  bind" and "what we do with the bindings"

**Deferred (linked rules, not in scope for Rule 13 itself):**
- Rule 14b — `:lambda` (deferred from `:define` round)
- Rule 14c — `:defmacro` (deferred from `:define` round)
- Rule 23 onwards — FQDN handling (already mostly settled
  but may need refinement after we use it more)
- Rule 25 — type annotation tightness
- A future rule on multi-line value indent calibration if
  Option B causes issues in practice

### Rule 14 ✅ — `define` is always vertical

User locked 2026-05-02 with explicit canonical examples covering
0-arg / 1-arg / multi-arg cases.

**Shape:**
- `:wat::core::define` keyword alone on line 1
- signature subform `(:NAME ...)` indented 2; name on
  signature's first line
- args one per line, indented 4 (zero or more — including the
  zero case)
- `-> :RET` on own line, indented 4, always last in signature
- body indented 2, matching the signature's open paren depth
- **always vertical** — no fits-on-one-line exception,
  regardless of arg count

**0-arg / nullary — signature collapses to one line:**
```scheme
(:wat::core::define
  (:pi -> :f64)
  3.14159)
```

**Amended 2026-05-02 during the `:lambda` round.** The original
locked version had the arrow on its own line:
```scheme
(:wat::core::define
  (:pi
    -> :f64)
  3.14159)
```
That's now superseded. **The general principle:** when the args
region is empty, the signature collapses to one line (name +
arrow + return type all together). The args-on-their-own-lines
shape only applies when there ARE args; once there's at least
one arg, the form stays fully vertical.

**1-arg (Q1 → always vertical, even at the cost of 5 lines for a
1-line function):**
```scheme
(:wat::core::define
  (:double
    (x :i64)
    -> :i64)
  (:wat::core::* x 2))
```

**Multi-arg (the user's original example):**
```scheme
(:wat::core::define
  (:my-fn
    (some-arg :some-type)
    (next-arg :next-type)
    -> :ret-type)
  (...body...))
```

**Why this shape:**
- `define` alone separates the "what we're doing" keyword from
  the "what we're defining"
- Signature subform is a self-contained structural unit
- One arg per line is diff-friendly (adding/removing an arg is
  a one-line diff)
- Arrow + ret type on its own line is visually distinct from
  args
- Body indent matches signature indent → visual symmetry between
  the two halves of the form
- Always-vertical rule is simple to apply; no edge cases for
  short signatures

**Deferred (linked rules, not in scope for Rule 14 itself):**
- `lambda` — Q3, user said "pretty much exactly yes" but want
  to address it as its own rule. **Now locked as Rule 14b.**
- `defmacro` — Q4, same, "pretty much exactly yes"
- `let*` (and related) — Q5, deferred. **Now locked as Rule 13.**

These will get their own rules; Rule 14 covers `define` only.

---

### Rule 14b ✅ — `:lambda` is always vertical

User locked 2026-05-02. Same fundamental shape as Rule 14
(`:define`) — vertical, each element on its own line — with
ONE structural divergence: lambda has no name, so the **first
arg takes the head position of the signature subform**
(occupies the column where `:define`'s name would sit).

**Shape:**
- `:wat::core::lambda` keyword alone on line 1
- Signature subform `(...)` indented 2
- First arg at the head of signature (column +1 from
  signature's `(`)
- Subsequent args aligned at the same column as the first arg
- `-> :RET` on own line, aligned with args, last in signature
- Body indented 2, matching the signature's open paren depth
- **Always vertical** — no fits-on-one-line exception

**The single divergence from `:define`** (verified 2026-05-02):

If `:lambda`'s signature `(` is at column N:
- First arg at column N+1 (head position)
- Subsequent args at column N+1 (aligned with first)
- Arrow at column N+1 (aligned with args)

Compare to `:define` at the same position:
- Name at column N+1 (head position)
- Args at column N+2 (indent 2 from signature `(`, below name)

One column difference in arg indent. Standard Lisp "args align
to first arg" idiom applied when the head is the first arg
itself. Matches what's already in real wat code (e.g.,
`wat/stream.wat:95-99`).

**0-arg / nullary — signature collapses to one line:**
```scheme
(:wat::core::lambda
  (-> :ret-type)
  body)
```

Same general principle as `:define`'s 0-arg case (amended
2026-05-02): when the args region is empty, the signature
collapses to one line.

**1-arg:**
```scheme
(:wat::core::lambda
  ((only-arg :T)
   -> :ret-type)
  body)
```

**Multi-arg (the standard case):**
```scheme
(:wat::core::lambda
  ((some-arg :some-type)
   (next-arg :next-type)
   -> :ret-type)
  (...body...))
```

**Lambda as a value inside another form** (per Rule 13e —
multi-line value composes naturally):
```scheme
(:wat::core::let*
  (((handler :T)
     (:wat::core::lambda
       ((x :i64)
        -> :i64)
       (:wat::core::* x 2))))
  body)
```

The lambda's `(` lands at +2 from the binding's `(` (per
Rule 13b — Option B wrap), and the lambda then applies its
own rule with that `(` as its column-0 baseline. Rules
compose; nothing extra to specify.

**Why this shape:**
- Same "always vertical" discipline as Rules 13, 14, 16
- Diff-friendly; one arg = one line
- Composes cleanly inside let* bindings, function-call
  argument positions, and as a return value
- Matches existing wat-rs convention without modification

**Deferred (linked rules):**
- Rule 14c — `:defmacro` — **now locked.** See below.

---

### Rule 14c ✅ — `:defmacro` is structurally identical to `:define`

User locked 2026-05-02. Substrate verified: `:defmacro` uses
the same signature shape as `:define` (real example in
`wat/holon/Reject.wat`).

**Rule 14c IS Rule 14 with `:wat::core::defmacro` substituted
for `:wat::core::define` everywhere.** No divergences.

The three considered-and-dismissed "divergences":

1. **Args typed `:AST<T>` instead of `:T`.** Implementation
   detail of macros (they receive ASTs); doesn't change the
   signature shape. Rule 14 doesn't constrain what the type
   expression IS, just where it sits in the signature.
2. **Body is typically a quasi-quoted template** (backtick +
   `,x` unquotes). Body convention, not a signature one. The
   defmacro rule doesn't constrain the body's internal
   structure — that's whatever the body's form requires
   (template syntax has its own formatting story when we get
   to Rule 30 / quasiquote).
3. **Macros usually have at least one arg.** A 0-arg macro is
   theoretically possible (always expands to the same form
   regardless of context); shape would be the collapsed
   nullary form per Rule 14's amended 0-arg case.

**0-arg (collapsed signature, per Rule 14's amendment):**
```scheme
(:wat::core::defmacro
  (:user::always-bar -> :AST<:wat::holon::HolonAST>)
  `(:wat::holon::Atom "bar"))
```

**1-arg:**
```scheme
(:wat::core::defmacro
  (:user::wrap
    (x :AST<wat::holon::HolonAST>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Bundle (:wat::core::vec :HolonAST ,x)))
```

**Multi-arg (the canonical Reject.wat example, verbatim):**
```scheme
(:wat::core::defmacro
  (:wat::holon::Reject
    (x :AST<wat::holon::HolonAST>)
    (y :AST<wat::holon::HolonAST>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Blend
     ,x
     ,y
     1.0
     (:wat::core::- 0.0
       (:wat::core::/ (:wat::holon::dot ,x ,y)
                           (:wat::holon::dot ,y ,y)))))
```

The body's structure (quasiquote, unquote, nested forms, etc.)
follows whatever rule applies to its constituent forms. Rule
14c only specifies the signature + outer body indent.

### Rule 16 ✅ — Conditional family is always vertical

User locked 2026-05-02. Covers `:wat::core::if`, `:wat::core::cond`,
`:wat::core::match`. **Audit confirmed the family is closed at
these three** (no `:when`, `:unless`, `:case`, `:select`,
`:do`, `:begin`, `:loop`, `:while`, `:for`, `:switch` — the
substrate doesn't ship them).

**Shape (shared across the family):**
- Keyword alone on line 1
- Principal expression (if the form has one) on its own line
  indented 2
- `-> :ret-type` on its own line indented 2
- Branches / clauses / arms each on their own line indented 2
- Always vertical — no fits-on-one-line exception

**`:wat::core::if`** (the user's original example):
```scheme
(:wat::core::if
  cond-expr
  -> :ret-type
  true-branch
  false-branch)
```

**`:wat::core::cond`** (no principal expression; `-> :ret`
slides up to position 1; clauses stack):
```scheme
(:wat::core::cond
  -> :ret-type
  ((test-1) body-1)
  ((test-2) body-2)
  (:else default))
```

`:else` is required as the last clause per 058-036.

**`:wat::core::match`** (scrutinee in the principal-expression
position; arms stack):
```scheme
(:wat::core::match
  scrutinee-expr
  -> :ret-type
  ((pattern-1) body-1)
  ((pattern-2) body-2)
  (_ default))
```

**Sub-rule 16a — Clause body wrap (EQ1 → Option A).** When a
clause's body doesn't fit on one line, wrap inside the clause
indented 2 from the clause's open paren:
```scheme
((test-1)
  (some-long-body
    (with-subexpressions)))
```
Same "always indent 2 for nested structure" principle as
Rule 14's body indent.

**Sub-rule 16b — Single arm stays vertical (EQ2).** A
`:wat::core::match` with one arm (or a `:wat::core::cond` with
just `:else`) still uses the full vertical shape:
```scheme
(:wat::core::match
  scrutinee
  -> :ret-type
  ((only-pattern) body))
```

**Sub-rule 16c — Degenerate `:cond` with only `:else` (EQ3).**
Same shape as multi-clause `:cond`:
```scheme
(:wat::core::cond
  -> :ret-type
  (:else default))
```

**Why this shape:**
- Same "always vertical" discipline as Rule 14 (`:define`)
- One unified rule covers all three forms — no special cases
  per form
- Diff-friendly: adding/removing a branch/clause/arm is a
  one-line diff
- Top-to-bottom reading; no horizontal scanning across branches
- `-> :ret` is visually distinct, on its own line, in a
  consistent position relative to the principal expression

**NOT in this family** (different rules later):
- `:wat::core::try` (retired but still hooked) — one-arg
  propagation, no return type annotation, no branches
- `:wat::core::Result/try` / `:wat::core::Option/try` — same
  shape as the retired `:try` (one-arg propagation)
- `:wat::core::and` / `:wat::core::or` / `:wat::core::not` —
  boolean operators (separate "short binary form" rule when
  we get to it)
- `:wat::core::Result/expect` / `:wat::core::Option/expect` —
  extraction with panic-on-failure (Rule 19b — different shape)

---

## §5b — Try and Expect families ✅

### Rule 19 ✅ — `:try` family is always two lines

User locked 2026-05-02. Covers `:wat::core::Result/try`,
`:wat::core::Option/try`, and the retired `:wat::core::try`
(kept hooked only for migration diagnostics; same shape applies
for as long as it lingers).

**Substrate constraints (verified):**
- `Result/try` and `Option/try` take EXACTLY 1 arg
- No return type annotation (return type inferred from
  enclosing function's `:Result<T,E>` or `:Option<T>`
  signature)
- One body expression only

**Shape:**
- Keyword alone on line 1
- The single arg on line 2 indented 2
- **Always two lines** — even when the arg fits on the same
  line as the head. The "fits-on-one-line" exception is gone.

**Canonical:**
```scheme
(:wat::core::Result/try
  (:wat::kernel::spawn-program src :wat::core::None))

(:wat::core::Option/try
  (:wat::core::HashMap/get table key))

(:wat::core::try
  body)
```

**Multi-line arg (per Rule 13e — value follows its own rule at
relative indent):**
```scheme
(:wat::core::Result/try
  (:wat::core::match
    scrutinee
    -> :Result<:T,:E>
    ((Ok v) (Ok (process v)))
    ((Err e) (Err (wrap-err e)))))
```

The try's first line is always head-only; the arg's internal
structure is the arg's form's concern.

**Why two lines always:**
- Try is control-flow (early-return propagation); deserves
  visual weight
- Consistent with the user's "always vertical for structurally
  significant forms" preference (define, lambda, let*,
  conditional family)
- One canonical shape per AST — formatter normalizes
  one-liner usages to two-line shape

**Note on retired `:wat::core::try` (EQ3):** user is mid-
refactor of the retirement. The shape rule applies wherever
`:try` lands during the migration; wat-lint flags the
deprecated head with a migration suggestion (separate
linter concern).

### Rule 19b ✅ — `:expect` family is always vertical (4 lines)

User locked 2026-05-02. Covers `:wat::core::Result/expect` and
`:wat::core::Option/expect`. **Different shape from `:try`** —
expect carries an explicit return type annotation and a panic
message; it's structurally closer to the conditional family
(typed head, vertical args).

**Substrate constraints (verified):**
- Both forms take EXACTLY 4 args, surfaced to the user as:
  `(:Type/expect -> :T <res-or-opt-expr> "panic message")`
- The 4 args under the hood: `->` literal, the type, the
  value being unwrapped, the panic message string

**Shape:**
- Keyword alone on line 1
- `-> :ReturnType` on line 2 indented 2
- The value being unwrapped on line 3 indented 2
- The panic message string on line 4 indented 2
- **Always four lines** — no fits-on-one-line exception

**Canonical (the user's locked form):**
```scheme
(:wat::core::Result/expect
  -> :SomeType
  result-expr
  "panic message string")

(:wat::core::Option/expect
  -> :SomeType
  option-expr
  "panic message string")
```

**Multi-line value or message (compose via Rule 13e
principle):** if the value or message is itself a multi-line
form, it follows its own rule at relative indent; the expect's
shape is unchanged at the outer level.

```scheme
(:wat::core::Result/expect
  -> :SomeType
  (:wat::core::let*
    (((intermediate :T)
       (compute-thing x)))
    (refine intermediate))
  "panic if compute or refine fails")
```

**Why this shape:**
- Same "always vertical" discipline as Rules 13, 14, 14b, 14c,
  16, 19
- Type annotation visually distinct on its own line (matches
  conditional-family precedent in Rule 16)
- Panic message visually distinct from the value being
  expected — the message is the failure-mode declaration, the
  value is the success path
- Consistent line count makes diff churn predictable

**Relationship to Rule 19:**
- `:try` is propagation (early-return; type inferred); 2 lines
- `:expect` is panic-on-failure (terminates; type explicit); 4 lines
- Same Result/Option semantic family, different syntactic
  shape; different rules

---

## §6 — Collections (Bundle / vec / HashMap / HashSet) ✅

User locked 2026-05-02 with all four rules.

### Rule 20 ✅ — `:vec` is always vertical

User answered EQ1 → A (always vertical, no fits-on-one-line
exception). Matches the convention already in `seed-fixture.wat:60-66`.

**Shape:**
- Head + element-type on line 1
- Elements one per line indented 2
- **Always vertical** when there are elements (no one-line collapse)
- Empty collapses to head-only (EQ5 → A)

**Empty:**
```scheme
(:wat::core::vec :T)
```

**Single element (still vertical):**
```scheme
(:wat::core::vec :T
  only-elem)
```

**Multi-element (the typical case, matches existing real-world
seed-fixture.wat):**
```scheme
(:wat::core::vec :wat::telemetry::Event
  (:demo::seed::log-event 1000 "alpha")
  (:demo::seed::log-event 2000 "beta")
  (:demo::seed::log-event 3000 "gamma")
  (:demo::seed::log-event 4000 "delta")
  (:demo::seed::log-event 5000 "epsilon"))
```

**Multi-line element (composes via Rule 13e — element follows
its own rule at relative indent):**
```scheme
(:wat::core::vec :wat::holon::HolonAST
  (:wat::holon::Bind
    :role-subject
    (:wat::holon::Atom "dog"))
  (:wat::holon::Bind
    :role-verb
    (:wat::holon::Atom "chases")))
```

### Rule 21 ✅ — `:Bundle` is structurally identical to `:try`

User answered EQ4 → same as `:try`. `:wat::holon::Bundle` takes
exactly one arg (a vec of HolonAST); shape rule is identical to
Rule 19's `:try` family.

**Shape:**
- Keyword alone on line 1
- Single arg on line 2 indented 2
- **Always two lines** — even when the arg is short
- Inner vec follows its own rule (Rule 20) at relative indent

**Canonical (matches real-world `Ngram.wat`):**
```scheme
(:wat::holon::Bundle
  (:wat::core::vec :wat::holon::HolonAST
    (:wat::holon::Bind :role-subject (:wat::holon::Atom "dog"))
    (:wat::holon::Bind :role-verb    (:wat::holon::Atom "chases"))
    (:wat::holon::Bind :role-object  (:wat::holon::Atom "toy"))))
```

**Single-element Bundle (still 2 lines outer + Rule 20 vec
inner):**
```scheme
(:wat::holon::Bundle
  (:wat::core::vec :wat::holon::HolonAST
    (:wat::holon::Bind :role-subject (:wat::holon::Atom "dog"))))
```

**Bundle wrapping non-vec arg (e.g., a `:wat::core::map` call):**
```scheme
(:wat::holon::Bundle
  (:wat::core::map
    (:wat::std::list::window xs n)
    (:wat::core::lambda
      ((window :wat::holon::Holons)
       -> :wat::holon::HolonAST)
      (:wat::holon::Sequential window))))
```

The inner form follows whatever rule applies to it; Bundle's
rule only constrains the outer two-line shape.

### Rule 22 ✅ — `:HashMap` always vertical; k-v pairs on same line; no alignment

User answers locked:
- EQ1 → A (always vertical, even when fits on one line)
- EQ2 → A (k-v pairs on same line, not each on own line)
- EQ3 → no alignment, single space between key and value
- EQ5 → A (empty collapses to head-only)

**Shape:**
- Head + value-type on line 1
- Each key-value pair on its own line, indented 2
- Single space between key and value (no column alignment)
- **Always vertical** when there are pairs
- Empty collapses to head-only

**Empty:**
```scheme
(:wat::core::HashMap :V)
```

**Single pair (still vertical):**
```scheme
(:wat::core::HashMap :V
  key value)
```

**Multi-pair (matches real-world `WorkUnit.wat:236-238`):**
```scheme
(:wat::core::HashMap :wat::telemetry::Tag
  asset-key asset-val
  stage-key stage-val)
```

**No column alignment — single space, even with varying-length
keys** (per EQ3):
```scheme
(:wat::core::HashMap :V
  :foo 1
  :bar-name 42
  :baz 100)
```

NOT:
```scheme
;; rejected — column alignment causes diff churn when a key
;; length changes
(:wat::core::HashMap :V
  :foo       1
  :bar-name  42
  :baz       100)
```

**Multi-line value (composes via Rule 13e — value wraps below
the key, indented 2 from the pair's start column):**
```scheme
(:wat::core::HashMap :V
  asset-key
    (:wat::holon::Atom :BTC)
  stage-key
    (:wat::holon::Atom :market-eval))
```

When a value is too long to fit on the same line as its key,
the key sits alone on its line and the value wraps to the next
line indented 2 from the pair's column. (This matches Rule 13b
Option B from `:let*`'s wrap.)

### Rule 22b ✅ — `:HashSet` mirrors `:vec`

`:wat::core::HashSet` is structurally a Bundle alias keyed on
membership; for formatting it follows the same shape as `:vec`
(head + element-type + elements one per line). Same edge cases
apply.

**Empty:**
```scheme
(:wat::core::HashSet :T)
```

**Single element:**
```scheme
(:wat::core::HashSet :T
  only-elem)
```

**Multi-element:**
```scheme
(:wat::core::HashSet :T
  elem-1
  elem-2
  elem-3)
```

### Why this shape (collections as a family)

- **Always vertical** is consistent with every other locked rule
  (13, 14, 14b, 14c, 16, 19, 19b)
- One canonical shape per AST — formatter normalizes one-liner
  usages to the vertical form
- Diff-friendly: adding/removing an element or pair is a
  one-line diff
- Bundle/vec/HashMap/HashSet share one principle, with HashMap's
  k-v-on-same-line as the one structural variant
- Empty collections collapse to head-only — matches the
  general "args region empty → no inflation" principle from
  Rule 14's 0-arg amendment

### Composition notes

- Collections nested inside `:let*` bindings, function-call
  arguments, or `:define`/`:lambda` bodies follow their own
  rule at relative indent (per Rule 13e principle)
- The bindings vector in `:let*` is NOT a `:vec`; it's a
  bindings-list literal, which has its own Rule 13 shape
  (different from Rule 20)
- Bundle/vec composition is handled by each rule independently:
  Bundle puts its arg on the next line; the inner vec follows
  Rule 20 from there

---

## §7 — Symbols and FQDN handling ✅

User locked 2026-05-02 with the principle:

> "symbols in wat cannot contain whitespace - they are as long as
> they are - long names are a visual indicator for a type alias"

This is the unifying principle behind Rules 23, 24, AND 13c
(long FQDN in let* binding). Stated clearly:

### The atomicity-and-signal principle

**Symbols are atomic.** The formatter cannot break them. No
wrapping inside an FQDN; no inserting whitespace in a type
sigil; no truncation; no abbreviation. What the symbol IS is
what gets emitted, in full, on whatever line it lands.

**Long symbols are a SIGNAL, not a problem to fix.** When a
name is long enough to push a line past 120 cols, the formatter
leaves the over-length line alone. That visible over-length is
**the formatter telling you, visually, that you should make a
type alias.** Hiding the length via wrapping would mask the
signal; preserving the length surfaces it.

**The formatter conserves information by refusing to fix.** It
doesn't lint, doesn't refactor, doesn't auto-alias. It lays the
length bare so the lint signal lands on the user.

This is a **feedback-loop principle**, not just a layout rule.
The formatter is part of how wat tells you "your naming has
drifted; make an alias."

### Rule 23 ✅ — Symbols (including FQDNs) never wrap

`:wat::holon::HolonAST` always stays on one line. If a call's
head FQDN doesn't fit at the parent's indent, the CALL breaks
before the head (extending the parent's wrap), not inside the
symbol. If the symbol itself is so long that no breaking helps,
the line stays over-length and the visible smell points at
"alias this."

```scheme
;; YES — wrap the call's args, not the FQDN
(:user::wrapper-fn arg-1
                   :wat::holon::HolonAST/some-very-long-method-name)

;; NO — never break inside the FQDN (this isn't even legal wat)
(:user::wrapper-fn arg-1
                   :wat::holon::HolonAST/some-very-long-
                                         method-name)

;; YES — over-length line stays over-length; the LENGTH is the signal
(:user::wrapper-fn :wat::holon::HolonAST/an-egregiously-named-method-call-that-blows-past-120)
;; ↑ formatter doesn't try to fix; user reads "I should alias this"
```

### Rule 24 ✅ — Reject illegal whitespace in type sigils

`<>`, `:(...)`, `:fn(...)`, `:[...]` cannot contain whitespace
at the wat parser level (per CHEATSHEET, enforced by `/ignorant`).
Listed here as a Rule 24 entry because users may try to "format"
code that has illegal whitespace and expect wat-fmt to fix it.
**wat-fmt does NOT fix — it rejects with a parse diagnostic
pointing at the offending character.**

```scheme
;; YES — atomic type sigils
:Atom<HolonAST>
:fn(:i64,:i64) -> :i64

;; PARSE ERROR — wat-fmt errors at the space
:Atom< HolonAST >
:fn(:i64, :i64) -> :i64
```

This is a parse-time error, not a format-time choice. Same
principle as Rule 23: the formatter doesn't fix what's
substrate-illegal; it surfaces it as an error.

### How this composes with other rules

- **Rule 13c (long FQDN in let* binding)** — same principle
  applied to a let* binding: don't wrap; let the line be
  over-length; the over-length screams "make a type alias."
- **Rule 22 (HashMap k-v alignment rejected)** — same family
  of "formatter doesn't mask smells" thinking; column alignment
  would mask "your keys are inconsistently named."
- **Future wat-lint rule** — over-length lines from oversized
  symbols become lint warnings: *"this symbol is N chars; consider
  a type alias."* The formatter sets up the lint signal by
  preserving the length; the linter delivers the message.

---

## §8 — Type annotations ✅

User locked 2026-05-02 with "we're good — we'll make this later
— our notes are good." Rules 25 + 26 follow the
atomicity-and-signal principle from §7.

### Rule 25 ✅ — Type tight to parameter

```scheme
;; YES
((x :T) (y :U))

;; NO — never extra space before type
((x  :T) (y  :U))

;; NO — never break before type
((x
  :T)
 (y
  :U))
```

`(x :T)` is one structural unit. The space between name and
type is exactly one character; the type sigil is atomic per
Rule 24.

### Rule 26 ✅ — Arrow stays on the signature line (mostly moot post-Rule 14)

The principle: don't break just the arrow.

```scheme
;; YES — arrow rides with the signature
(define (:user::foo (x :T) -> :U) body)

;; NO — arrow alone on a wrap line
(define (:user::foo (x :T)
                    -> :U) body)
```

**Note:** Rule 26 was sketched before Rule 14 went
always-vertical. With Rules 14 / 14b / 14c locked, define /
lambda / defmacro signatures always wrap to vertical regardless,
and the arrow gets its own line as part of the vertical
signature. Rule 26 is largely subsumed for those forms.

Rule 26 still applies to OTHER places where signatures appear
(e.g., function-call type annotations, lambda-as-value in
non-binding contexts) — same principle: if the signature wraps,
the WHOLE signature wraps; never just the arrow on its own.

---

## §9 — Atoms and literals ✅

User locked 2026-05-02. All three rules follow the
atomicity-and-signal principle from §7: the formatter doesn't
rewrite primitive literals; the user's choices are preserved
verbatim.

### Rule 27 ✅ — String literals preserved character-for-character

Strings pass through unchanged. No rewrapping; no escape
canonicalization (`ÿ` stays as `ÿ`, never normalized
to `ÿ` or vice versa); no quote-style normalization.

The parser enforces what's valid (escape sequences, encoding);
the formatter doesn't second-guess.

### Rule 28 ✅ — Integer / float strict preservation

Numeric literals preserved as written. `1_000_000` stays as
`1_000_000`; `1000000` stays as `1000000`; `0x_DEAD_BEEF` stays
as written; `1.5e10` stays in scientific form.

The user's choice of separators / radix / notation IS information
about intent (separators for readability of magic constants;
hex for bit patterns; scientific for very-large/small). The
formatter preserves intent.

If the file's numeric style ends up inconsistent (e.g.,
`1_000` next to `2000`), that's a wat-lint signal, not a
wat-fmt fix.

### Rule 29 ✅ — Keywords / symbols strict preservation

Case preserved as written. The parser already enforces what's
valid (per CONVENTIONS: kebab-case for verbs, CamelCase for
types, snake_case for nothing); the formatter doesn't
double-enforce.

`nil` stays `nil`; `Nil` stays `Nil`; `:Foo` stays `:Foo`.
Whatever the parser accepts, the formatter emits.

---

## §10 — Quasiquote / unquote / unquote-splicing ✅

User locked 2026-05-02 with "i trust your gut... we'll tweak it
later if necessary." All three sub-rules and six edge-case
defaults locked per the proposal in the chat.

**Important correction to earlier draft:** the prior version of
this rule claimed wat doesn't have reader macros. WRONG. wat-rs
ships reader macros for all three:

| Reader macro | Desugars to |
|---|---|
| `` `form `` | `(:wat::core::quasiquote form)` |
| `,sym` | `(:wat::core::unquote sym)` |
| `,@sym` | `(:wat::core::unquote-splicing sym)` |

Confirmed in `wat-rs/src/parser.rs:151-153`. All real-world
wat code (Reject.wat, Ngram.wat, test.wat) uses the
reader-macro shorthand exclusively; nobody writes the explicit
FQDN form.

### Rule 30 ✅ — Reader macros formatted as shorthand, transparent to wrapped form

User locked. Three sub-rules, all transparency-based.

**Sub-rule 30a — Reader-macro shorthand is the canonical form.**
When the formatter encounters `:wat::core::quasiquote` /
`:unquote` / `:unquote-splicing` in the AST, it emits the
shorthand (`` ` ``, `,`, `,@`), never the explicit FQDN form.
Even if the user wrote the FQDN, the formatter normalizes to
shorthand.

```scheme
;; AST: (:wat::core::quasiquote (some form))
;; emitted as:
`(some form)
;; NEVER as:
(:wat::core::quasiquote (some form))
```

**Sub-rule 30b — Reader macro directly attached to following
form, no space.**

```scheme
;; YES
`(form)
,symbol
,@symbol

;; NO — space between reader macro and form
` (form)
, symbol
,@ symbol
```

**Sub-rule 30c — Quasiquote is transparent to its wrapped
form's layout.** The wrapped form follows whatever rule applies
to it (define / vec / cond / etc.). The backtick rides on the
same line as the form's open paren; the form's internal
structure isn't affected by being quasiquoted.

```scheme
;; backtick + multi-line define inside — define follows Rule 14
`(:wat::core::define
   (,name -> :wat::test::TestResult)
   ,body)
```

### Locked defaults for the six edge cases

- **EQ1 (always normalize to shorthand):** YES. AST FQDN forms
  emit as shorthand on output. Per 30a.
- **EQ2 (no space between reader macro and form):** YES.
  Per 30b.
- **EQ3 (wrapped form's layout follows its own rule):** YES.
  Per 30c. Quasiquote is purely a syntactic prefix.
- **EQ4 (nested quasiquotes):** Each level handled independently.
  No special escape syntax beyond what wat already provides.
  Formatter emits the AST faithfully without trying to be
  clever about depth-aware unquoting.
- **EQ5 (`,@` in non-list position):** Substrate type-checker's
  problem; the formatter emits as written. wat-lint flags
  suspect uses.
- **EQ6 (multi-line argument to unquote):** Comma sits on the
  line where the call starts; the call's internal structure
  follows the call's rule. Same transparency principle as 30c.

```scheme
;; long unquoted call wraps per the call's rule
,(some-long-call
   arg-1
   arg-2)
```

### Canonical examples (matching real-world wat)

**Simple splice into a call (from `test.wat:228-231`):**
```scheme
(:wat::core::defmacro
  (:wat::test::program & (forms :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::Vector<wat::WatAST>>)
  `(:wat::core::forms ,@forms))
```

**Multi-arg expansion with multiple unquotes (from
`wat/holon/Reject.wat`):**
```scheme
(:wat::core::defmacro
  (:wat::holon::Reject
    (x :AST<wat::holon::HolonAST>)
    (y :AST<wat::holon::HolonAST>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Blend
     ,x
     ,y
     1.0
     (:wat::core::- 0.0
       (:wat::core::/ (:wat::holon::dot ,x ,y)
                           (:wat::holon::dot ,y ,y)))))
```

**Nested forms with splice + unquote (from `test.wat:310-322`):**
```scheme
`(:wat::core::define (,name -> :wat::test::TestResult)
   (:wat::kernel::run-sandboxed-ast
     (:wat::core::forms
       ,@prelude
       (:wat::core::define
         (:user::main
           (stdin  :wat::io::IOReader)
           (stdout :wat::io::IOWriter)
           (stderr :wat::io::IOWriter)
           -> :wat::core::unit)
         ,body))
     (:wat::core::Vector :wat::core::String)
     :wat::core::None))
```

### Why this shape

- **Transparency is the principle.** Reader macros are syntactic
  sugar that doesn't affect the wrapped form's structure. The
  formatter respects that — formats wrapped forms by their own
  rules; the reader macro is just a prefix character.
- **Shorthand is universal in real wat code.** No existing code
  uses the FQDN form; canonicalization to shorthand matches
  what users actually write.
- **No space anywhere.** Lisp tradition. `` `(form) ``,
  `,sym`, `,@sym` are all atomic visually.
- **Compositionality.** Quasiquote inside lambda inside let*
  inside define all compose by each form following its own
  rule; the reader macros add no constraint.

### Tweak point flagged

User direction: *"we'll tweak it later if necessary."* Likely
candidates for future revision:
- If real-world defmacro bodies grow patterns that the
  current rule emits awkwardly
- If nested quasiquote becomes common enough to warrant a
  more sophisticated layout
- If wat adds new reader macros (e.g., a `'sym` for plain
  quote — currently `(quote ...)` is explicit)

---

## §11 — Multi-line strings ✅

User locked 2026-05-02 after correcting my deferral:

> "what's the deferral?.... multiline should just be
>
>   (..sometheing...
>     "sdfsad
>      sdfsdfsa
>      asdfas")
>
> right?... we have these already existing?..
>
> clojure docstrings are how i'm thinking"

The user was right. Multi-line strings already work in wat —
the lexer at `wat-rs/src/lexer.rs:294-323` accepts newlines
inside string literals (the loop only stops on `"`, `\`, or
EOF; no special-case rejection of `\n`). The deferral was
wrong; the rule is fully derivable from the atomicity principle
in §7 plus Rule 27.

### Rule 31 ✅ — Multi-line strings are atomic; continuation lines preserved verbatim

**Multi-line strings are atomic, just like single-line strings.**
The formatter cannot change their content — including the
continuation-line whitespace embedded inside the string.

**The rule:**
- Opening `"` lands wherever the string sits in its parent form
  (per the parent's layout rule)
- Continuation lines stay EXACTLY as the user typed them
- The formatter NEVER touches whitespace inside a string
  literal, even if the result looks misaligned with the
  surrounding indentation

**Canonical (Clojure-docstring style):**
```scheme
(:wat::core::define
  (:my-fn (x :T) -> :U)
  "Compute the thing.

   Takes an X and returns a Y. The Y has these properties:
     - thing 1
     - thing 2"
  body)
```

The continuation lines align under the opening `"` because the
user typed them that way; the formatter preserves it.

**If the parent form's indentation changes, the string moves
position but content stays:**

```scheme
;; before: parent had less indent
(:some-form
  "line one
   line two")

;; after: surrounded by more nesting; opening `"` moves right
(:wat::core::let*
  (((x :T) some-value))
  (:some-form
    "line one
   line two"))
;;   ↑ continuation line's indent unchanged — it's INSIDE the string
;;   ↑ now visually misaligned with the new opening `"` position
;;   ↑ formatter cannot fix this; the indent IS the string's content
```

The visual misalignment after a reformat IS information — it
tells the user "your string content has indentation baked in
that no longer matches the surrounding form." That's a
**rewrite-the-string** signal, not a **wat-fmt fixes it**
situation. Same atomicity-and-signal principle as long FQDNs:
the formatter conserves information; user revises if needed.

### Why this just works

- Substrate already accepts multi-line strings (lexer doesn't
  reject `\n`)
- Rule 27 (string preservation) already covers single-line
  strings; multi-line is the same rule
- The atomicity principle from §7 means the formatter doesn't
  modify primitives
- Clojure-docstring-style is just "string literal in a
  particular position"; the formatter doesn't need a
  special docstring concept

### What this does NOT specify

- **Where docstrings live in a function** — not the formatter's
  call. wat doesn't (yet) have a substrate-level docstring
  position; if one ships later, the formatter just emits
  whatever's in that AST slot per Rule 31. Today, docstrings
  live as `;;` comments above the form (Rule 7) OR as string
  literals inside the body if the calling convention permits.
- **Heredoc / triple-quote syntax** — wat doesn't have these
  separate forms today; standard `"..."` literals already
  span lines. If wat adds explicit heredoc syntax later, this
  rule extends to it with the same atomicity discipline.

---

## What's NOT covered (intentionally)

- **Naming conventions** (kebab-case vs snake_case vs CamelCase)
  — wat-fmt only handles whitespace/layout; naming rules belong
  in wat-lint.
- **Code organization** (function ordering, exports first vs
  defines first) — also wat-lint.
- **Dead code removal** — refactoring, not formatting.
- **Auto-fix for any wat-lint warning** — see DESIGN.md
  "What's NOT in scope."

## How user marks this up

Suggested workflow:

1. Read each rule top-down.
2. For ✅ CONFIRMED rules: skip unless something feels wrong;
   reopen explicitly if so.
3. For ❓ DRAFT rules: react. Replace example, change wording,
   reject entirely, or accept (mark ✅).
4. For 🔧 OPEN rules: pick a recommendation or define your
   own; mark ❓ DRAFT or ✅ CONFIRMED.
5. Add new rules as needed.
6. Once the file feels close, the rules become the test
   corpus's golden-file expectations.
