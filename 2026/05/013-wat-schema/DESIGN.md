# wat-schema — DESIGN

Substrate-tier crate for declarative shape enforcement at
boundaries. Builds on the wat type system; adds refined types,
shape composition, clara-esque rules, and policy actions on
validation failure.

---

## The four questions are the design compass

- **Obvious?** A user reading a schema declaration knows what
  shape is required and what happens on violation.
- **Simple?** One concept (schema); refined types + shapes + rules
  + policies compose; nothing exotic.
- **Honest?** Validation failures are typed structured values; no
  exceptions; the contract IS the implementation.
- **Good UX?** Declaring a schema is the most readable thing on
  the boundary; reusable; positive security model wins for free.

## The four layers

```
┌───────────────────────────────────────────────────────┐
│ LAYER 4 — Policies                                    │
│           Actions on validation failure               │
│           (reject / log / quarantine / rate-limit /   │
│            mirror / transform / alert)                │
└───────────────────────┬───────────────────────────────┘
                        │ uses
                        ▼
┌───────────────────────────────────────────────────────┐
│ LAYER 3 — Rules (clara-esque)                         │
│           Cross-field invariants; conditional shape   │
│           requirements; multi-shape OR; compound      │
│           validations                                 │
└───────────────────────┬───────────────────────────────┘
                        │ uses
                        ▼
┌───────────────────────────────────────────────────────┐
│ LAYER 2 — Shapes                                      │
│           Composite type declarations using refined   │
│           types. Single declaration; reusable; named  │
└───────────────────────┬───────────────────────────────┘
                        │ uses
                        ▼
┌───────────────────────────────────────────────────────┐
│ LAYER 1 — Refined types                               │
│           Types with constraints. (:string :min-length│
│            8 :max-length 128); (:i64 :range 1 100);   │
│            (:email); (:uuid); (:url-https); (:pattern │
│            #"...")                                    │
└───────────────────────┬───────────────────────────────┘
                        │ uses
                        ▼
┌───────────────────────────────────────────────────────┐
│ wat type system (already in wat-rs proper)            │
└───────────────────────────────────────────────────────┘
```

Each layer composes cleanly. Layer 1 is the smallest unit
(constrained primitive type). Layer 2 composes Layer 1 into
structures. Layer 3 adds cross-field intelligence. Layer 4 is
the action half (orthogonal to validation).

## Layer 1 — Refined types

Constraints attached to base types. Each refined type is a
type-checkable predicate plus an error message generator.

### String refinements

```scheme
(:string)                          ; any string
(:string :min-length 1)            ; non-empty
(:string :min-length 8 :max-length 128)
(:string :pattern #"^\d{6}$")     ; regex match
(:string :one-of ["yes" "no"])    ; enum
(:string :max-length 254 :pattern #"^[^@]+@[^@]+\.[^@]+$")  ; email-ish
```

### Number refinements

```scheme
(:i64)                             ; any i64
(:i64 :range 1 100)                ; 1..=100
(:i64 :min 0)                      ; non-negative
(:f64 :exclusive-range 0.0 1.0)    ; (0,1) — exclusive bounds
(:f64 :precision 2)                ; max 2 decimal places
```

### Collection refinements

```scheme
(:vec :T :max-length 1000)         ; bounded vec
(:vec :T :min-length 1)            ; non-empty vec
(:vec :T :unique? true)            ; no duplicates
(:hashmap :K :V :max-keys 100)
```

### Domain refinements (built on the above)

```scheme
(:email)                           ; sugar for the email regex
(:url-https)                       ; valid URL with https scheme
(:uuid)                            ; UUID format
(:hex-string :length 64)           ; SHA-256-shaped hex
(:bearer-token)                    ; "Bearer xxxxx"
(:iso-8601)                        ; ISO 8601 timestamp
(:rfc3339)                         ; RFC 3339 timestamp
```

These domain refinements are reusable named patterns. A wat
codebase can `(:wat::schema::define-type :Email (:email))` and
reuse `:Email` everywhere.

### How refinement runs at runtime

The Rust shim takes a typed value and a refinement; runs the
refinement; returns either:
- `Ok(typed-value)` — refinement passed; original value
  returned
- `Err(violation)` — typed structured `Violation` carrying
  what failed and where

No exceptions. No panics. Failure is data.

## Layer 2 — Shapes

Composite type declarations describing object structure. A shape
is a typed combinator.

### Shape declaration

```scheme
(:wat::schema::define :LoginRequest
  (:body (:shape
    (:email    :Email)            ; required field
    (:password :Password)
    (:totp     (:optional :TotpCode))  ; optional field
    (:remember (:default false :bool))))  ; optional with default
  (:headers (:shape
    (:authorization :BearerToken)
    (:user-agent    (:optional :string)))))
```

### Field modifiers

| Modifier | Meaning |
|---|---|
| `(:optional :T)` | field may be absent; if present, must be `:T` |
| `(:default v :T)` | field may be absent; defaults to `v` if so |
| `(:nullable :T)` | field may be `nil` (different from absent) |
| `(:one-of [:T1 :T2 ...])` | tagged union; field must match exactly one |
| `(:flexible :T)` | accept additional fields not in shape |
| `(:strict :T)` | reject if any unexpected field present (default) |

### Shape composition

```scheme
;; Inline shape
(:shape (:name :string) (:age :i64))

;; Named (reusable) shape
(:wat::schema::define :Person
  (:shape (:name :string) (:age :i64)))

;; Shape extension (adds fields)
(:wat::schema::define :Employee
  (:extend :Person
    (:shape (:salary :i64) (:title :string))))

;; Shape intersection (must match all)
(:wat::schema::define :PersonAndEmployee
  (:and :Person :Employee))

;; Shape union (must match any)
(:wat::schema::define :PersonOrCompany
  (:or :Person :Company))
```

### Validating against a shape

```scheme
(:wat::schema::validate :LoginRequest input)
;; => :Result<:LoginRequest, :ValidationError>
```

A successful validation returns the input as a typed
`:LoginRequest`. Every consumer downstream gets type-bridged
guarantees from there.

A failed validation returns a structured `:ValidationError`
listing every violation found (not just the first — collect all
for one-pass reporting).

## Layer 3 — Rules (clara-esque)

Beyond per-field refinement, cross-field rules express
invariants the schema declaration alone can't:

### Conditional invariants

```scheme
(:wat::schema::define :LoginRequest
  ...field declarations...

  ;; Cross-field rule: TOTP required for high-tier accounts
  (:rule "totp required for gold+ accounts"
    :when (:and (:absent :totp)
                (:>= (:account-tier :user) :gold))
    :then :reject))
```

### Cross-field invariants

```scheme
(:rule "amount cannot exceed account balance"
  :when (:> :amount (:balance :account))
  :then :reject)

(:rule "end-date must be after start-date"
  :when (:< :end-date :start-date)
  :then :reject)
```

### Multi-shape disambiguation

```scheme
(:wat::schema::define :Payment
  (:or :CardPayment :BankPayment :CryptoPayment)

  ;; Rule fires AFTER union matches; refines based on which arm
  (:rule "card payments require CVV in production"
    :when (:and (:matches :CardPayment)
                (:env :production))
    :then (:require :cvv)))
```

### Rule shape

A rule has three parts:
- `:when` — pattern over the validated input + ambient context
- `:then` — action: `:reject` (with optional message) /
  `:require <field>` (must be present) / `:warn` (log; don't
  reject) / `:transform <fn>` (modify input; common for
  defaults)
- `:tag` (optional) — name for logging/metrics

The rule engine evaluates ALL rules (collects all violations);
short-circuits only when an unrecoverable rule fires.

### Why clara-esque

Clara is a Clojure rules engine — patterns + actions. wat-schema's
rules are a small DSL inspired by Clara's pattern-then-action
shape, NOT a full RETE network. Just enough to express
cross-field invariants declaratively without descending to
imperative validation code.

The user's reference to "telemetry's clara-esque shape
validation" suggests this pattern has been used in the wat
ecosystem before for similar purposes. **Reuse the discipline.**

## Layer 4 — Policies

Validation produces a result; **policy decides what to do with
the result.** Cleanly separable from validation.

### Policy declaration

```scheme
(:wat::schema::policy :reject-and-log
  :on-violation
    (:reject-with :status 400
                  :body-template :default-error-body)
    (:log-to :security-pipeline
             :level :warn
             :include [:violations :remote-addr :timestamp]))
```

### Built-in policy actions

| Action | What it does |
|---|---|
| `:reject-with :status N` | Return HTTP status `N` with structured error body |
| `:log-to <pipeline>` | Emit event to a logging pipeline |
| `:rate-limit (:by <key> :max <rate>)` | Increment a per-key rate counter; reject if exceeded |
| `:quarantine` | Stash the request for later analysis (forensics) |
| `:mirror-to <endpoint>` | Send a copy to honeypot/analysis endpoint |
| `:transform <fn>` | Apply a transformation; re-validate; if pass, continue |
| `:alert <channel>` | Trigger an alert (PagerDuty / Slack / etc.) |
| `:custom <wat-fn>` | Arbitrary wat function; takes `:Violation`; returns `:Action` |

### Policy composition

```scheme
;; Policies stack
(:wat::schema::policy :production-strict
  :on-violation
    (:rate-limit (:by :remote-addr :max 10/minute))
    (:log-to :security-pipeline :level :warn)
    (:reject-with :status 400))

;; Different policies for different violation types
(:wat::schema::policy :tiered
  :on-refinement-violation (:reject-with :status 400)
  :on-rule-violation       (:alert :security-team)
  :on-shape-violation      (:reject-with :status 400 :log :info))
```

### Per the failure-engineering position

Validation NEVER throws or panics. Failures are typed values; the
policy layer decides what to DO with the failure. This separation
matches every other failure-engineering pattern in the substrate
(arc 011's no-retries; arc 009's HandlerError taxonomy; etc.).

The reject decision is OPINIONATED by the policy; the validation
layer is unopinionated.

## Integration with the wat type system

The wat type system already proves "this value has type :T at
runtime." wat-schema extends this with:
- **Refinement** — narrowing :T to satisfy additional predicates
- **Composition** — declaring how :T's compose into shapes
- **Cross-field** — invariants that span fields
- **Boundary semantics** — what to do at I/O edges where
  untrusted bytes become typed values

The wat-vm enforces the type proofs internally; wat-schema runs
at boundaries where bytes/JSON/EDN cross into the wat-vm.

A shape declaration is itself a TYPE. A handler's signature
`:Handler<:Request<:LoginRequest>>` declares the handler accepts
a request whose body conforms to `:LoginRequest`. The compiler
ensures the handler doesn't receive malformed data; the wat-vm
ensures the schema validation runs at the boundary; the policy
layer decides what happens to bad data.

## Cross-boundary usage patterns

### HTTP request validation (arcs 009/010)

```scheme
(:wat::http::router::define-app :my-app
  (:post "/api/users"
    :schema       :CreateUserRequest
    :on-violation :reject-and-log-policy
    :handler      create-user))
```

### HTTP response validation (arc 009 — optional)

```scheme
(:wat::http::router::define-app :my-app
  (:get "/api/users/:id"
    :schema-out   :UserResponse           ; assert response shape
    :handler      get-user))
```

### RPC payload validation (arc 007)

```scheme
(:wat::remote-program::define :compute-risk
  :input-schema  :RiskComputeInput
  :output-schema :RiskComputeOutput
  :handler       compute-risk-impl)
```

### Config file validation

```scheme
(:wat::schema::validate-and-load :MyAppConfig
  :path "/etc/my-app/config.edn"
  :on-violation :exit-with-clear-error)
```

### IPC message validation (arc 109)

```scheme
(:wat::program::define :worker
  :message-schema :WorkerMessage         ; declared shape
  :on-violation   :log-and-drop          ; bad message; drop
  :handler        worker-impl)
```

## Per the four questions on the architecture

- **Obvious?** ✅✅ — schema is a universal concept; declarations
  read as documentation of the contract; policy is the one
  thing about wat-schema that's non-obvious (acceptable — it's
  separable and reusable)
- **Simple?** ✅ — four layers; each composes; nothing exotic;
  the rules engine is small (NOT a full RETE network)
- **Honest?** ✅✅ — validation failures are typed values; the
  policy layer is the only opinionated part and it's named
  explicitly; nothing pretends to be more than it is
- **Good UX?** ✅✅ — declarative; reusable; one schema crate
  serves every boundary; positive security wins for free

Strong shape. Honest is ✅✅ because the typed-failure model is
real. **Could be ✅✅✅** if we can structurally guarantee that
no validation can ever throw — a Rust-level assertion that every
refinement is total. Achievable with care; worth pursuing in
slice planning.

## Open architectural questions

A. **Refined-type extensibility.** How do users add new domain
   refinements (e.g., `:credit-card` with Luhn check)? Two
   options: (1) wat-level via combinators; (2) Rust shims
   registered as extensions. Lean: (1) for most cases; (2) only
   when wat can't express the constraint efficiently.

B. **Rule engine evaluation order.** Strict declaration order, or
   data-flow order (rule on field A runs before rule on field
   B)? Lean: declaration order; document the constraint; rules
   shouldn't depend on each other's order in practice.

C. **Recursive schema references.** Can `:Tree` reference itself?
   Yes for type purposes; need to guard against infinite-loop
   validation. Lean: structural recursion bounded by input
   structure; finite for finite inputs.

D. **Async validation.** Some validations need I/O (e.g., "this
   email isn't on the blocklist," "this UUID exists in the
   database"). Should refinements be async? Lean: NO — keep
   wat-schema purely synchronous; async checks belong in
   handlers, not schemas. Schemas validate STRUCTURE; not
   semantic state-of-the-world.

E. **Schema migration / versioning.** When endpoints change
   shape, how do schemas evolve without breaking clients?
   Out of scope for v1; document the question; address with
   a `wat-schema-migrate` sibling crate later if patterns
   surface.

F. **Schema → OpenAPI / JSON Schema export.** Schemas are
   structured; could be exported to standard formats for client
   SDK generation. Out of scope for v1; clean future arc.

## What's NOT in scope

- **Async validation** — schemas are synchronous structural
  checks; semantic checks belong in handlers
- **Schema migration / versioning** — sibling crate later
- **Standard format export (OpenAPI, JSON Schema)** — sibling
  crate later
- **Code generation from schemas** — sibling crate later if
  someone needs SDK generation
- **Database schema integration** — wat-schema validates row
  shapes IF the database client returns them; the database
  client itself is application-tier
- **Custom evaluation strategies (RETE, etc.)** — the small rule
  engine is sufficient; no need for a real production rules
  engine
- **Schema editor / GUI** — schemas are wat code; edit in your
  editor of choice

## The honest connection to security

Schema enforcement is a substrate property; security is its
dominant beneficiary. **Three quotes worth holding in mind for
the implementation discipline:**

1. *"The most reliable way to defeat injection is to never write
   code that can be injected against."* Schemas reject
   non-conforming input before it reaches code that could be
   injected.

2. *"Type the boundary."* Wat already does this for native types;
   wat-schema completes it for refined / composite / rule-validated
   shapes.

3. *"Failure is data."* Validation failures are structured typed
   values; the policy layer decides their fate. The boundary
   stays clean.

A wat application with schemas at every boundary has stronger
positive-security guarantees than a similar application behind
any WAF. **The substrate makes this cheap; we should make this
the default.**
