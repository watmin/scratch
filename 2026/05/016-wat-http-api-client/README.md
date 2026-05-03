# wat-http-api-client — spec-driven typed client SDK

User direction (2026-05-03):

> *"wat-http-api-client => consumes a user's api-spec definitions
> and uses the for input+output scheme validation -- deps on
> http-client, user-api-spec (transitive wat-scheme)"*

> *"the spec enables both server and client to bootstrap"*

---

## What wat-http-api-client is

The **opinionated typed client SDK generator** that consumes a
wat-http-api-spec and auto-generates typed call functions for
every operation in the spec.

Given a spec, this crate produces:
- **One typed call function per operation** (e.g.,
  `:my-client/create-user`, `:my-client/get-user`)
- **Typed input shape constructors** (build inputs the spec
  guarantees are valid)
- **Output deserialization + validation** (responses validated
  against the spec's output shape)
- **Typed error variant decoding** (errors land as the typed
  `:Result<:Output, (:or :Variant1 :Variant2 ...)>` from
  the spec)
- **Auth header injection** (per operation's auth declaration;
  credentials configured at client creation time)
- **Path interpolation** (path params plug into the URL)
- **Optional retry / circuit-breaker hooks** (per operation's
  idempotency hint; opt-in)

The user calls these functions; the substrate handles
serialization, transport, validation. **Same spec; same
guarantees as the server side.**

## The structural drift property — earned again

The client compiles against the SAME `Spec` value the server
does (arc 014's design). The client can't construct a request
the spec doesn't allow, because the typed constructor literally
doesn't permit it. The client can't fail to handle an error
variant the spec declares, because the `:Result` union forces
exhaustive handling.

**Server and client cannot drift.** Same spec → same shapes →
same types on both ends. ✅✅✅ Honest at the architectural
level.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-http-api-client/`
per the arc-013 pattern.

```
wat-rs/crates/wat-http-api-client/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-http-api-spec (../wat-http-api-spec),
                       #   wat-http-client (../wat-http-client),
                       #   wat-schema (../wat-schema; transitive
                       #   via spec)
  src/                 # Rust shim (spec consumption; call
                       #   function generation; auth injection;
                       #   error decoding)
  wat/http/api/client/ # The DSL: generate, configure,
                       #   call-function templates
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
                       # (mock servers from spec; round-trip
                       # tests with arc 015)
```

## Layering

```
LAYER 5 — user app code            calls :my-client/operation-name
  ↓ uses
LAYER 4 — wat-http-api-client (THIS ARC)
            generates typed call functions from spec;
            handles serialization, transport, validation
  ↓ depends on
LAYER 3 — wat-http-client (arc 011)  outbound HTTP
            wat-http-api-spec (arc 014)  contract
  ↓
LAYER 2 — wat-schema (arc 013)       validation (transitive)
LAYER 1 — Rust ecosystem             reqwest + tokio + rustls
```

## How a user uses it

```scheme
;; In my-http-api-client crate:
(:wat::http::api::client::generate :my-client
  :spec     :MyApi
  :base-url "https://api.example.com"
  :auth     (:bearer-token-from-env "API_TOKEN")
  :timeout  (:Duration/seconds 30)
  :retry    (:never))   ; per failure-engineering — caller decides

;; Auto-generated functions; one per spec operation:
;;
;;   (:my-client/create-user
;;     :input :CreateUserRequest)
;;     -> :Result<:User, (:or :Conflict :BadRequest)>
;;
;;   (:my-client/get-user
;;     :id :UserId)
;;     -> :Result<:User, :NotFound>
;;
;;   (:my-client/list-users
;;     :limit  (:i64 :range 1 200)
;;     :offset (:i64 :min 0))
;;     -> :Result<:ListUsersResponse, ...>

;; User code calls the typed functions:
(:wat::core::let
  ((result (:my-client/create-user
             :input (:CreateUserRequest
                      :email    "user@example.com"
                      :name     "Alice"
                      :password "..."))))
  (:wat::core::match result
    ((:Ok user) ...handle the user...)
    ((:Err :Conflict :message m) ...handle conflict...)
    ((:Err :BadRequest :violations vs) ...handle bad input...)))
```

The user writes:
- The spec (in my-http-api; arc 014)
- Calls to the typed functions (wherever they consume the API)
- The configuration (base URL, auth, timeout, retry policy)

Everything else — request building, serialization, transport,
deserialization, validation, error decoding — is generated.

## Why this is a separate crate from wat-http-client

Same reason wat-http-api-server is separate from wat-http-router:
**opinionated middleware on top of a bare primitive.**

- wat-http-client is the bare HTTP client (any URL, any method,
  any body)
- wat-http-api-client is the spec-driven typed SDK generator

Users who want raw HTTP calls (third-party APIs without a wat
spec; one-off scripts) use wat-http-client directly. Users who
want the spec-driven typed SDK use this crate. Both layers are
opt-in.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: spec consumption, call function generation, auth injection, error decoding, retry semantics, integration with wat-http-client. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arcs 011, 013, 014 firm up.) |

## Conventions inherited

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: NO retries by default (caller decides per
  arc 011 position); errors are typed per spec; no exceptions
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: depends on wat-http-client (canonical) +
  wat-http-api-spec (the contract); no new external deps

## Cross-references

- **arc 014 (wat-http-api-spec)** — the contract; hard dep
- **arc 011 (wat-http-client)** — outbound HTTP; hard dep
- **arc 013 (wat-schema)** — validation; transitive via spec
- **arc 015 (wat-http-api-server)** — natural complement; same
  spec; different role; **drift impossible** because both
  compile against the same Spec value
- **WAT-NETWORK.md** — auth declarations compose with the
  wat-network's mTLS layer; client cert configuration via
  spec's `:auth (:mtls ...)` variant
- **DEPENDENCY-DOCTRINE.md** — no new external deps

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-http-api-client` (gaze-approved; matches
  api-* family; differentiates from bare wat-http-client via
  the `api-` prefix)
- **Architecture:** sketched; design firms up via chat iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. arcs 011 + 013 + 014 have shipped slice 1
  2. User signals "let's start"

## The structural symmetry

The user's three crates per API:

```
my-http-api               — declares what the API IS
       ↑
       │ both compile against the same Spec value
       │
   ┌───┴───────┐
   ▼           ▼
my-http-api-server    my-http-api-client
   ↓                          ↓
implements operations   calls operations
   ↓                          ↓
serves responses        receives responses
       ↑                      ↑
       └─── wire format ──────┘
            (EDN / JSON
             via content-type
             negotiation)
```

The wire format is symmetric. The validation runs symmetrically
(server validates inbound + outbound; client validates inbound +
outbound). The error variants are symmetric. The auth shape is
symmetric. **Symmetry across the wire is enforced by the spec.**

This is what big-tech learned the hard way: APIs are contracts;
contracts must be a single source of truth; both ends derive
from it; drift is the most expensive bug class. The substrate
makes drift structurally impossible by making the contract a
typed value both ends compile against.

Worth being explicit about: **this is the strongest API
correctness story available in any language, anywhere, today.**
Smithy / OpenAPI / gRPC all do it via codegen with separate
artifacts; the artifacts can drift if generators diverge. The
wat substrate compiles both consumers against the SAME value.
There is no separate artifact to drift; the spec IS the
artifact.
