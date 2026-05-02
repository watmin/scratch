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
- Rule 14c — `:defmacro` (still ❓ DRAFT; user said "pretty
  much exactly yes" relative to `:define`)

### Rule 15 ❓ — `lambda` keeps params on head line; body indented 2

```scheme
(:wat::core::lambda ((x :T) (y :U) -> :V)
  (:wat::core::+ x y))
```

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
  extraction with panic-on-failure

---

## §6 — Collections (Bundle / Vec / HashMap / HashSet) ❓ DRAFT

### Rule 20 ❓ — Hung indent for long collection literals

If the collection fits under the line limit, keep on one line:

```scheme
(:wat::core::vec :i64 1 2 3 4 5)
```

Otherwise, head + element-type on first line; elements indented 2:

```scheme
(:wat::core::vec :wat::holon::HolonAST
  (:wat::holon::Bind :role-subject (:wat::holon::Atom "dog"))
  (:wat::holon::Bind :role-verb    (:wat::holon::Atom "chases"))
  (:wat::holon::Bind :role-object  (:wat::holon::Atom "toy")))
```

### Rule 21 ❓ — Bundle is a vec of HolonASTs

```scheme
(:wat::holon::Bundle
  (:wat::core::vec :wat::holon::HolonAST
    (:wat::holon::Bind ...)
    (:wat::holon::Bind ...)))
```

The `Bundle` head and the inner `vec` head each on their own
line; the vec elements indented 2 from `vec`'s open paren.

### Rule 22 ❓ — HashMap key-value alignment

When a HashMap fits on one line, keep it. Otherwise:

```scheme
(:wat::core::HashMap :wat::core::Symbol :i64
  :foo  1
  :bar  42
  :baz  100)
```

Keys aligned; values aligned in a column. Two-space minimum gap
between key and value. (If alignment looks ugly because of one
extreme-length key, fall back to one-pair-per-line without
column alignment.)

---

## §7 — FQDN handling ❓ DRAFT

### Rule 23 ❓ — Never wrap an FQDN

`:wat::holon::HolonAST` always stays on one line, regardless of
context. If a function call's head FQDN doesn't fit at the
parent's indent, the call breaks BEFORE the head (extending the
parent's wrap), not inside the FQDN.

```scheme
;; YES — wrap the call, not the FQDN
(:user::wrapper-fn arg-1
                   :wat::holon::HolonAST/some-very-long-method-name)

;; NO — never break inside the FQDN
(:user::wrapper-fn arg-1
                   :wat::holon::HolonAST/some-very-long-
                                         method-name)
```

### Rule 24 ❓ — Reject illegal whitespace in `<>`, `:(...)`, `:fn(...)`, `:[...]`

Per the existing CHEATSHEET rule (and what `/ignorant` enforces).
wat-fmt rejects input that has it with a clear "this isn't valid
wat syntax" diagnostic, pointing at the exact offending character.

```scheme
;; YES
:Atom<HolonAST>
:fn(:i64,:i64) -> :i64

;; NO — wat-fmt errors at the space
:Atom< HolonAST >
:fn(:i64, :i64) -> :i64
```

(This is a parse-time error, not a format-time choice. Listed
here because users may try to "format" code that has this and
expect wat-fmt to fix it. wat-fmt does NOT fix; it rejects with
diagnostic.)

---

## §8 — Type annotations ❓ DRAFT

### Rule 25 ❓ — Tight binding to parameter

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

### Rule 26 ❓ — Return arrow on the signature line

```scheme
;; YES
(define (:user::foo (x :T) -> :U) body)

;; NO
(define (:user::foo (x :T)
                    -> :U) body)
```

If the signature including `-> :T` doesn't fit, wrap the WHOLE
signature to its own line per Rule 14.

---

## §9 — Atoms and literals 🔧 OPEN

### Rule 27 🔧 — String literal preservation

Strings preserved character-for-character. No rewrapping. No
canonicalization of escapes. (User: confirm? or should `ÿ`
get normalized to its Unicode form, etc.?)

### Rule 28 🔧 — Integer / float canonicalization

Decision needed:
- Strict preservation (e.g., `1_000_000` stays as written)
- Canonicalize to no-separators (`1000000`)
- Canonicalize to with-separators every-3-digits (`1_000_000`)

Most formatters either strict-preserve or canonicalize-with-
separators. Recommendation: **strict preservation** for v1; user
can override later if the file looks inconsistent.

### Rule 29 🔧 — Keyword / symbol canonicalization

- Always lowercase keywords? Or preserve case?
- `nil` vs `NIL` vs `Nil`?

Recommendation: **strict preservation**; the parser already
enforces what's valid.

---

## §10 — Quote and quasiquote ❓ DRAFT

### Rule 30 ❓ — Quote stays in `(quote ...)` form

wat doesn't have reader macros (`'foo` for `(quote foo)`). All
quotes are explicit. wat-fmt formats `(quote foo)` per Rule 13/14
and friends; nothing special.

If quasiquote / unquote primitives ship later (`,` and `,@`-like
forms), this rule needs revision.

---

## §11 — Multi-line strings 🔧 OPEN

### Rule 31 🔧 — Multi-line string handling

If wat supports multi-line string literals (heredoc or triple-
quote), the formatter should preserve them verbatim INCLUDING
internal whitespace. But the surrounding form's indentation
shouldn't break the string's content.

Decision needed once multi-line string syntax is settled in
wat-rs.

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
