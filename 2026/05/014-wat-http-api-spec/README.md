# wat-http-api-spec — single source of truth for HTTP API contracts

User direction (2026-05-03):

> *"i think we should model out another pair from this...*
>
> *wat-http-api-spec wat-http-api-server wat-http-api-client*
>
> *wat-http-api-spec => like swagger.. consume this to produce an
> api spec that both server and client can use"*

> *"think on how aws uses their json defs for api modeling... i want
> something like that... swagger... openapi... these are the things
> to model here"*

---

## What wat-http-api-spec is

The **single source of truth** for HTTP API contracts. The wat
equivalent of OpenAPI / Swagger (REST-shaped specifications) +
Smithy (AWS's IDL).

A user crate (`my-http-api`) consumes wat-http-api-spec and
declares:
- **Operations** (named API operations: `:create-user`, `:get-user`)
- **HTTP method + path** (`:post "/users"`, `:get "/users/:id"`)
- **Input shapes** (request body, query params, headers, path
  params; declared via wat-schema)
- **Output shapes** (response body, status codes; via wat-schema)
- **Errors** (typed error variants per operation)
- **Decorations** (auth requirements, idempotency, rate limits,
  caching policy, etc.)
- **Metadata** (tags, descriptions, examples; for docs generation)

The compiled spec is a **typed wat value** that both the server
crate (arc 015) and the client crate (arc 016) reference. **Both
sides see the same source of truth at compile time.**

## Why this matters — the structural drift property

When server and client BOTH derive from the same compiled spec,
**shape drift becomes structurally impossible**. The server can't
return a shape the spec doesn't allow (validation at the server
boundary catches it). The client can't construct a request the
spec doesn't allow (the typed call function structurally can't
produce one). Every value crossing the wire conforms to one
source of truth.

This is the load-bearing property the three-crate design earns:
**✅✅✅ Honest** at the architectural level. Server/client drift
is the most common production bug class in API-driven systems
(client expects field X; server renamed it; runtime explosion).
We make that bug structurally unrepresentable.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-http-api-spec/`
per the arc-013 pattern. Foundation crate for the
api-server / api-client pair.

```
wat-rs/crates/wat-http-api-spec/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-schema (../wat-schema)
  src/                 # Rust shim (spec compilation; type
                       #   bridging; reachability checks)
  wat/http/api/spec/   # The DSL: define, operation, shape,
                       #   error, auth, rate-limit forms
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
```

## Layering

```
LAYER 5 — wat-http-api-server   (arc 015 — uses spec)
          wat-http-api-client   (arc 016 — uses spec)
  ↓ both depend on
LAYER 4 — wat-http-api-spec     (THIS ARC)
  ↓ uses
LAYER 3 — wat-schema            (arc 013 — refined types,
                                  shapes, rules, policies)
  ↓ uses
LAYER 2 — wat type system
  ↓
LAYER 1 — wat-vm
```

## Sketch — what a spec looks like

```scheme
(:wat::http::api::spec::define :MyApi
  ;; Metadata
  :version "1.0.0"
  :title "My API"
  :base-path "/api/v1"
  :description "Example API for the wat-http-api-spec README"

  ;; Reusable refined types (delegated to wat-schema)
  (:type :Email
    (:string :pattern #"^[^@]+@[^@]+\.[^@]+$" :max-length 254))
  (:type :Password (:string :min-length 8 :max-length 128))
  (:type :UserId (:uuid))

  ;; Reusable shapes
  (:shape :User
    (:id :UserId)
    (:email :Email)
    (:name :string :min-length 1 :max-length 100)
    (:created-at :iso-8601))

  (:shape :CreateUserRequest
    (:email :Email)
    (:name :string :min-length 1 :max-length 100)
    (:password :Password))

  ;; Error variants — typed across all operations
  (:errors
    (:Conflict   :status 409 :body (:shape (:message :string)))
    (:NotFound   :status 404 :body (:shape (:message :string)))
    (:BadRequest :status 400 :body (:shape (:violations (:vec :Violation)))))

  ;; Operations — the API surface
  (:operation :create-user
    :method   :post
    :path     "/users"
    :input    :CreateUserRequest
    :output   (:status 201 :body :User)
    :errors   [:Conflict :BadRequest]
    :auth     (:bearer-token :scopes ["users:write"])
    :idempotent? false
    :description "Create a new user account.")

  (:operation :get-user
    :method        :get
    :path          "/users/:id"
    :path-params   (:shape (:id :UserId))
    :output        (:status 200 :body :User)
    :errors        [:NotFound]
    :auth          (:bearer-token :scopes ["users:read"])
    :idempotent?   true
    :cacheable?    (:max-age 60)
    :description   "Get a user by ID.")

  (:operation :list-users
    :method       :get
    :path         "/users"
    :query-params (:shape
                    (:limit  (:default 50 (:i64 :range 1 200)))
                    (:offset (:default 0  (:i64 :min 0))))
    :output       (:status 200 :body (:shape
                                       (:users (:vec :User))
                                       (:total :i64)))
    :auth         (:bearer-token :scopes ["users:read"])
    :idempotent?  true))
```

The spec is **wat code**. Define-time. The compiled spec is a
typed value that arcs 015/016 consume directly.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: spec compilation, type bridging, decoration taxonomy, error model, integration with wat-schema, the structural-drift-impossibility argument. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arc 013 firms up.) |

## Conventions inherited

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: spec compilation errors are typed; no
  exceptions
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: depends on wat-schema (arc 013) and wat
  substrate; no new external Rust deps

## Cross-references

- **arc 013 (wat-schema)** — the foundation. Spec shapes ARE
  wat-schema shapes. Refined types, rules, policies all
  available in spec declarations.
- **arc 015 (wat-http-api-server)** — primary consumer. Auto-
  generates routes, input/output validation, error responses,
  auth middleware from the spec.
- **arc 016 (wat-http-api-client)** — primary consumer. Auto-
  generates typed call functions, auth header injection, error
  decoding from the spec.
- **arc 010 (wat-http-router)** — transitive (via 015). The
  api-server crate generates routes that flow through
  wat-http-router.
- **arc 011 (wat-http-client)** — transitive (via 016). Client
  call functions ultimately use wat-http-client for transport.
- **WAT-NETWORK.md** — spec-declared auth requirements compose
  with the wat-network's mTLS + signed-payload identity layer.

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-http-api-spec` (gaze-approved; the `http-`
  prefix keeps it in the HTTP family; the `api-` prefix
  differentiates spec-driven layer from bare wat-http-server;
  `spec` is the universal noun for what this produces)
- **Architecture:** sketched; design firms up via chat iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. arc 013 (wat-schema) has shipped slice 1 (so refined types,
     shapes, and validation are available)
  2. arc 010 (wat-http-router) has shipped slice 1 (so the server
     side has routing to compose with)
  3. User signals "let's start"

## Why this is a separate arc from server / client

Could the spec live inside wat-http-api-server? Yes. So why
separate?

Three reasons aligned with the four questions:

1. **Honest layering.** The spec is the contract; server and
   client are different consumers of that contract. Mock
   servers, doc generators, contract test runners are also
   consumers. Splitting respects that the spec stands alone.

2. **Simple shape.** Each crate does one thing. Same elegance
   as wat-schema standing alone (substrate-tier) so every
   boundary uses it; the spec is a similar shared contract.

3. **Obvious dependency direction.** Server and client both
   depend on the spec; the reverse is never true. Splitting
   makes this visible at the package graph.

This mirrors how Smithy organizes things (the spec is its own
artifact; Smithy generates servers, clients, docs, tests from
the same spec).
