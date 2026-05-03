# wat-http-api-server — spec-driven server skeleton

User direction (2026-05-03):

> *"wat-http-api-server => consumes a user's api-spec definitions
> and uses them for input+output scheme validation -- deps on
> http-router, user-api-spec (transitive wat-schema, wat-http-router)"*

> *"the http server layer... it leverages the nature of the api spec
> to auto generate routes and errors.. they are qualified in the
> user's spec crate..."*

> *"but its built on our sinatra-esque server, users can shim in
> whatever middle way they want... the http-api stuff is just a
> middleware for routes and validations"*

---

## What wat-http-api-server is

The **opinionated server skeleton** that consumes a wat-http-api-spec
and auto-generates the entire boilerplate of a typed HTTP API.

Given a spec, this crate produces:
- **Route registrations** (one per spec operation; via wat-http-router)
- **Input validation** (request body, query params, headers, path
  params; via wat-schema using the spec's input shape)
- **Output validation** (response body conforms to the spec's
  output shape; same wat-schema)
- **Error responses** (typed error envelope; status codes per
  spec error variants; consistent across all endpoints)
- **Auth middleware** (per operation's auth declaration)
- **Rate-limit middleware** (per operation's rate-limit decoration)
- **Cache headers** (per operation's cacheability declaration)
- **Idempotency keys** (per operation's idempotency hint;
  optional)

The user implements **only the handler bodies**. The signatures
are derived from the spec.

## Why this is a separate crate from wat-http-router

It's an **opinionated middleware layer** above the bare router.
Users who want raw control of routes use wat-http-router directly.
Users who want the spec-driven paved road use this crate.

This matches the substrate's pattern everywhere:
- wat-http-server is the minimal foundation
- wat-http-router is the opinionated DSL on top
- wat-http-api-server is a MORE opinionated layer above that
- Each layer is opt-in; users land where their need matches

User direction:

> *"the http-api stuff is just a middleware for routes and
> validations"*

Exactly. Composes cleanly; doesn't replace.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-http-api-server/`
per the arc-013 pattern.

```
wat-rs/crates/wat-http-api-server/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-http-api-spec (../wat-http-api-spec),
                       #   wat-http-router (../wat-http-router),
                       #   wat-schema (../wat-schema; transitive
                       #   via spec)
  src/                 # Rust shim (spec consumption; route
                       #   generation; middleware composition)
  wat/http/api/server/ # The DSL: generate, handlers, middleware
                       #   wiring, error envelope
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
```

## Layering

```
LAYER 5 — user app code             handler bodies (the actual
                                     business logic)
  ↓ consumed by
LAYER 4 — wat-http-api-server (THIS ARC)
            generates routes + validators + middleware
            from the spec
  ↓ depends on
LAYER 3 — wat-http-router (arc 010)  routing primitive
            wat-schema (arc 013)     validation
            wat-http-api-spec (arc 014)  contract
  ↓
LAYER 2 — wat-http-server (arc 009)  bare HTTP handler interface
LAYER 1 — Rust ecosystem             tokio + hyper
```

## How a user uses it

The user has THREE crates per API:

```
my-http-api               ← spec (uses wat-http-api-spec; arc 014)
my-http-api-server        ← uses wat-http-api-server + my-http-api
                            implements the handlers
my-http-api-client        ← uses wat-http-api-client + my-http-api
                            (arc 016)
```

### my-http-api (the spec)

```scheme
(:wat::http::api::spec::define :MyApi
  ...operations, shapes, errors as in arc 014 README...)
```

### my-http-api-server (the server impl)

```scheme
;; Generate the server skeleton
(:wat::http::api::server::generate :my-server
  :spec :MyApi)

;; my-server is now a Handler (per arc 009) — fully composed.
;; Implement the per-operation handlers:

(:wat::http::api::server::define-handler :my-server :create-user
  ;; Signature derived from spec:
  ;;   (req :CreateUserRequest)
  ;;     -> :Result<:User, (:or :Conflict :BadRequest)>
  (:lambda ((req :CreateUserRequest)
            -> :Result<:User, (:or :Conflict :BadRequest)>)
    ;; Business logic only. Validation already happened.
    (:wat::core::let*
      ((existing (:db/find-user-by-email (:Request/email req))))
      (:wat::core::if (:Optional/some? existing)
        (:Result/err (:Conflict :message "email already in use"))
        (:wat::core::let*
          ((user (:db/create-user
                   :email    (:Request/email req)
                   :name     (:Request/name req)
                   :password (:Request/password req))))
          (:Result/ok user))))))

(:wat::http::api::server::define-handler :my-server :get-user
  (:lambda ((id :UserId)
            -> :Result<:User, :NotFound>)
    (:wat::core::let*
      ((user (:db/find-user-by-id id)))
      (:wat::core::if (:Optional/some? user)
        (:Result/ok (:Optional/unwrap user))
        (:Result/err (:NotFound :message "user not found"))))))

;; Run the server (per arc 009)
(:wat::http::server::serve
  :handler   :my-server
  :listeners (:wat::core::vec :Listener
    (:Listener/uds :path "/var/run/my-server.sock")
    (:Listener/tcp :addr "127.0.0.1:8080")))
```

The user writes ONLY:
- The spec (in my-http-api)
- The handler bodies (in my-http-api-server)
- The deployment config (listeners, env vars, etc.)

Everything else is derived from the spec.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: spec consumption, route generation, middleware composition, error envelope, handler signature derivation. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arcs 010, 013, 014 firm up.) |

## Conventions inherited

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: typed errors per the spec; no exceptions
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: depends on wat-http-router (canonical
  routing layer) + wat-schema (canonical validation) + spec

## Cross-references

- **arc 014 (wat-http-api-spec)** — the contract this consumes;
  hard dependency
- **arc 010 (wat-http-router)** — routes generated from spec
  flow through; hard dependency
- **arc 013 (wat-schema)** — validation runs via wat-schema;
  transitive via spec
- **arc 009 (wat-http-server)** — the server handler interface;
  what the generated handler ultimately serves; transitive
- **arc 016 (wat-http-api-client)** — the natural complement;
  same spec; different role
- **WAT-NETWORK.md** — spec-declared auth composes with the
  wat-network's mTLS + signed-payload identity; the
  api-server defaults compose cleanly with sidecar deployment

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-http-api-server` (gaze-approved; matches
  established api-* family; differentiates from bare
  wat-http-server via the `api-` prefix)
- **Architecture:** sketched; design firms up via chat iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. arcs 010 + 013 + 014 have shipped slice 1 (the foundations
     this crate composes)
  2. User signals "let's start"

## The clean separation

The substrate provides:
- Route registration (auto)
- Input/output validation (auto)
- Error responses (auto)
- Auth middleware (auto)
- Rate limiting (auto)
- Cache headers (auto)
- Consistent error envelope (auto)

The user provides:
- WHAT each operation does (handler bodies)
- The data flow / business logic

The user direction:

> *"the user must still define /what/ happens in those operations
> with those shapes... as they must for when they are consuming
> or requesting them... but we can provide a paved road with
> good practices here?..."*

Exactly the right framing. Substrate = paved road; user = business
logic. Clean separation; no leak in either direction.
