# wat-schema — declarative shape enforcement at boundaries

User direction (2026-05-03):

> *"alright.. i was just thinking 'i need a waf'... and i don't think
> i do... what i need is... a scheme enforcement solution.... 'my
> shapes are this' and we can do something similar to what we did
> with telemetry's clara-esque shape validation?...*
>
> *idk...."*

> *"wat-schema - i like it - let's get a new scratch for this"*

[The user said "scheme" — clear context-from-typo for "schema."
Confirmed naming: `wat-schema`.]

---

## What wat-schema is

A substrate-tier crate for **declarative shape enforcement at
boundaries**. Built on top of wat's type system; adds three
things on top:

1. **Refined types** — types with constraints
   (`(:string :min-length 8 :max-length 128)`,
   `(:i64 :range 1 100)`, `(:email)`, `(:uuid)`).
2. **Shapes** — composite type declarations
   (`(:shape (:email :Email) (:password :Password))`).
3. **Rules** — clara-esque conditional/cross-field validation
   (`(:rule "totp required for high-value" :when ... :then :reject)`).
4. **Policies** — actions on validation failure
   (`:reject-400-and-log`, `:quarantine`, `:rate-limit-source`).

The wat type system already enforces basic structural correctness
at runtime. wat-schema is the layer where you say:
- *"This endpoint accepts ONLY this shape"*
- *"These cross-field invariants must hold"*
- *"When validation fails, do this"*

## The recognition behind this arc

The user almost reached for a **WAF** (Web Application Firewall;
mod_security; naxsi; signature-based). The WAF model is
**negative security**: "block these patterns." Reactive — CVE
appears, rule update follows. Misses anything novel.

The recognition: what they actually need is **positive security**.
"Only allow this shape." Anything else gets rejected at the
boundary, regardless of whether it matches a known attack pattern.

**Positive security is strictly stronger than negative.** Every
serious security architect knows this. The industry shipped WAFs
because positive security was historically too expensive to
implement (every endpoint needs a schema; every schema needs
maintenance). **Wat's type system already does most of the
work.** The schema-enforcement layer adds the thin wrapping that
makes positive security cheap.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-schema/` per
the arc-013 pattern. **Substrate-tier**, like wat-kwargs (arc 008)
— it builds on the wat type system and is used by application-tier
crates at THEIR boundaries.

```
wat-rs/crates/wat-schema/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   regex (for :pattern refined type)
  src/                 # Rust shim (refined-type runtime checks;
                       #   regex compilation; policy dispatcher)
  wat/schema/          # The DSL: define, shape, rule, policy
                       #   forms; refined-type combinators;
                       #   structured error model
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
```

## Layering

```
LAYER 4 — application code at boundaries
            (HTTP routes; RPC handlers; config loaders;
             IPC consumers; database row mappers)
  ↓ uses
LAYER 3 — wat-schema (THIS ARC)
            refined types + shapes + rules + policies
  ↓ uses
LAYER 2 — wat type system
            (already in wat-rs proper)
  ↓
LAYER 1 — wat-vm
```

## Where it gets used

Schema validation is useful at **every boundary** where wat code
receives data from outside:

| Boundary | Schema purpose |
|---|---|
| HTTP request body (arcs 009/010) | Validate inbound request matches endpoint contract |
| HTTP response body (arc 009) | Optional: assert outbound response matches contract |
| HTTP outbound request (arc 011) | Validate body before sending |
| RPC payload (arc 007 RemoteProgram) | Typed-cryptographic validation extends to shape |
| Config file load | Reject malformed config at startup |
| IPC message (arc 109 program-with-mailbox) | Inter-program message contracts |
| Database row | When wat talks to typed-row sources |
| File parsing | Beyond what wat-edn covers structurally |

**One schema crate; many consumers.** This is the substrate-tier
shape — like serde for Rust, or pydantic for Python.

## Sketch

```scheme
;; Refined-type aliases (reusable atoms)
(:wat::schema::define-type :Email
  (:string :pattern #"^[^@]+@[^@]+\.[^@]+$"
           :max-length 254))

(:wat::schema::define-type :Password
  (:string :min-length 8 :max-length 128))

(:wat::schema::define-type :TotpCode
  (:string :pattern #"^\d{6}$"))

;; Shape declaration (composite)
(:wat::schema::define :LoginRequest
  (:body (:shape
    (:email    :Email)
    (:password :Password)
    (:totp     (:optional :TotpCode))))
  (:headers (:shape
    (:authorization (:bearer-token))
    (:user-agent    (:optional :string))))
  ;; Clara-esque rule: cross-field invariant
  (:rule "totp required for gold+ accounts"
    :when (:and (:absent :totp)
                (:>= (:account-tier :user) :gold))
    :then :reject))

;; Endpoint declares the schema; on-violation policy
(:wat::http::router::define-app :my-app
  (:post "/login"
    :schema       :LoginRequest
    :on-violation (:wat::schema::policy
                    :reject-with :status 400
                    :log-to      :security-pipeline
                    :rate-limit  (:by :remote-addr :max 10/min))
    :handler      login-handler))

;; The handler signature is now GUARANTEED:
;;   login-handler :: :Request<:LoginRequest> -> :Result<...>
;; If the request doesn't match :LoginRequest, the handler is
;; never called. The rejection happened at the schema boundary
;; with a structured error.
```

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: refined types, shapes, rule engine, policy actions, integration with type system, cross-boundary usage. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once existing arcs firm up enough that we know what they need.) |

## Conventions inherited

From the foundation-tier arcs and recent application-tier arcs:

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: validation failures are TYPED structured
  values; not exceptions
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: regex (canonical Rust regex crate) for
  pattern-matching refined types

## Cross-references

- **arc 005 (wat-pause)** — wat-pause's break primitive captures a
  typed Environment; wat-schema can validate the captured shape at
  attach time
- **arc 007 (RemoteProgram)** — typed RPC payloads ARE schemas;
  wat-schema is the formal layer
- **arc 008 (wat-kwargs)** — schemas use kwargs heavily; arc 008
  is a hard dependency for the DSL ergonomics
- **arc 009 (wat-http-server)** — request/response validation at
  HTTP edge; wat-schema is the validator
- **arc 010 (wat-http-router)** — `:schema` kwarg on route
  declarations references schemas defined here
- **arc 011 (wat-http-client)** — outbound request validation
  before send
- **arc 109 (program-with-mailbox)** — IPC message validation
- **DEPENDENCY-DOCTRINE.md** — regex as a chosen dep (canonical;
  battle-tested; same family as crossbeam, serde, tokio)
- **WAT-NETWORK.md** — schema enforcement at the network boundary
  IS the positive-security half of the wat-network's defense
  (mTLS handles identity; schemas handle content)

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-schema` (locked via gaze; universal noun for
  declarative shape validation; reads in any language; the
  database-schema overload doesn't conflict because the namespace
  clarifies)
- **Architecture:** sketched; design firms up via chat iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. arc 009 (wat-http-server) has shipped slice 1 (so we know what
     a schema-validated handler signature should look like)
  2. arc 008 (wat-kwargs) has shipped its slice 1 (the DSL leans
     heavily on kwargs)
  3. User signals "let's start"

## The honest security argument

Two reasons the user reached for a WAF first:

1. **Industry default.** Every shop has a WAF; it's the
   default-shipped network-edge security control.
2. **Reactive instinct.** When a CVE hits, you want a knob to
   turn — WAFs give you that knob.

Neither argues that WAFs are *better* than schema enforcement.
They argue WAFs are *available* and *familiar*.

Schema enforcement is better:
- **Catches novel attacks** (anything not matching the shape;
  no signature needed)
- **Self-documents the contract** (the schema IS the API spec)
- **Prevents legitimate mistakes too** (developer typos; client
  bugs; breaking changes that drift from contract)
- **Eliminates a class of CVEs structurally** (input validation
  bugs are 70%+ of OWASP Top 10; positive-shape rejection
  removes the class, not the symptom)

The substrate makes this cheap. Wat already type-bridges at the
wat-vm boundary; wat-schema adds the refinement, composition,
and policy layers on top. **A wat application with schemas at
its boundaries has stronger positive-security guarantees than
a similar application behind any WAF.**

This is the kind of "we already built the hard part for other
reasons; the security win falls out" property that nobody else
has. Worth being explicit about.
