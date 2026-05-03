# wat-http-api-spec — DESIGN

The single source of truth for HTTP API contracts. Foundation
crate for the wat-http-api-{server,client} pair.

---

## The four questions are the design compass

- **Obvious?** A user reading a spec declaration knows what the
  API looks like — operations, shapes, errors, decorations all
  in one place.
- **Simple?** One DSL; one compiled artifact; both consumers
  read the same thing.
- **Honest?** The spec IS the contract; not "should be." Validation
  on both sides runs against the same compiled spec; drift is
  structurally impossible.
- **Good UX?** Spec declarations read as documentation; auto-
  generated routes/clients eliminate boilerplate; positive
  security falls out via wat-schema.

## Architecture

```
┌────────────────────────────────────────────────────────┐
│ User crate (my-http-api)                               │
│   (:wat::http::api::spec::define :MyApi ...)           │
│   ↓ compiles to                                        │
│   typed Spec value (exported)                          │
└────────────────────────────┬───────────────────────────┘
                             │ both reference
            ┌────────────────┴────────────────┐
            ▼                                 ▼
┌──────────────────────────┐    ┌────────────────────────┐
│ my-server (arc 015)       │    │ my-client (arc 016)    │
│   reads spec at compile  │    │   reads spec at        │
│   time; auto-generates   │    │   compile time; auto-  │
│   routes, validators,    │    │   generates typed call │
│   handlers signatures    │    │   functions            │
└──────────────────────────┘    └────────────────────────┘
```

Both consumers reference the SAME compiled Spec value. There is
no copy. There is no derivation gap. **The spec IS the contract;
the consumers are projections of the contract for their roles.**

## The spec — what it contains

### Top-level metadata

```scheme
(:wat::http::api::spec::define :MyApi
  :version     "1.0.0"
  :title       "My API"
  :description "Example API"
  :base-path   "/api/v1"
  :servers     ["https://api.example.com"
                "https://staging.api.example.com"]
  ...)
```

Optional. Used by doc generators; informational for clients.

### Type aliases (delegated to wat-schema)

```scheme
(:type :Email
  (:string :pattern #"^[^@]+@[^@]+\.[^@]+$" :max-length 254))
(:type :Password (:string :min-length 8 :max-length 128))
(:type :UserId (:uuid))
```

These are wat-schema refined types. Reusable across operations.

### Shapes (delegated to wat-schema)

```scheme
(:shape :User
  (:id :UserId)
  (:email :Email)
  (:name :string :min-length 1 :max-length 100))
```

Standard wat-schema shapes; the spec aggregates them so both
server and client see the same structure.

### Errors

```scheme
(:errors
  (:Conflict   :status 409 :body (:shape (:message :string)))
  (:NotFound   :status 404 :body (:shape (:message :string)))
  (:BadRequest :status 400 :body (:shape (:violations (:vec :Violation)))))
```

Errors are typed variants. Operations declare which errors they
can return; client decoders handle each variant; server enforces
that handlers don't return undeclared errors.

### Operations

The heart of the spec. Each operation is a named API endpoint:

```scheme
(:operation :create-user
  :method        :post              ; required
  :path          "/users"           ; required
  :input         :CreateUserRequest ; optional; default no body
  :output        (:status 201 :body :User)  ; required
  :errors        [:Conflict :BadRequest]    ; optional; default []
  :auth          (:bearer-token :scopes ["users:write"])  ; optional
  :idempotent?   false              ; optional; default true for GET/HEAD/OPTIONS
  :cacheable?    nil                ; optional; default nil
  :rate-limit    (:by :remote-addr :max 10/minute)  ; optional
  :description   "Create a new user account."  ; optional; for docs
  :tags          ["users" "write"]  ; optional; for grouping
  :examples      [...])             ; optional; for docs
```

Path can include params: `/users/:id` declares `:id` as a path
param (typed via `:path-params`).

Query params: `:query-params (:shape (:limit (:i64 :range 1 200)))`.

Headers: `:headers (:shape (:x-request-id :uuid))`.

Each operation is a complete declaration. Server and client both
compile against this exact shape.

### Auth declarations

```scheme
:auth (:bearer-token :scopes ["users:read"])
:auth (:basic-auth)
:auth (:api-key :header "X-API-Key")
:auth (:mtls :cert-validator :MyCertValidator)  ; wat-network mTLS
:auth :none                                      ; explicitly public
```

Auth declarations are typed enums. Server middleware enforces
them; client SDKs inject the appropriate headers.

### Rate-limit declarations

```scheme
:rate-limit (:by :remote-addr :max 10/minute)
:rate-limit (:by :auth-subject :max 100/minute)
:rate-limit (:tiered
              (:by :remote-addr :max 1000/hour)
              (:by :auth-subject :max 10000/hour))
```

Server middleware enforces; clients can read the spec to know
expected limits (for circuit-breaker patterns).

### Idempotency hint

```scheme
:idempotent? true   ; client can retry safely
:idempotent? false  ; client should NOT retry without idempotency key
```

Default: `true` for GET/HEAD/OPTIONS; `false` for POST/PUT/PATCH/DELETE.
Per arc 011's failure-engineering position (no auto-retries in client),
this is INFORMATIONAL — the client decides retry policy with this hint
in hand.

### Cacheability declaration

```scheme
:cacheable? (:max-age 60)
:cacheable? (:max-age 300 :vary [:authorization])
:cacheable? nil   ; do not cache
```

Server emits Cache-Control headers; client SDKs may use this for
local-cache decisions; CDNs read the headers.

## The compilation pipeline

```
spec source (wat code)
    ↓
[macro expansion]
    ↓
spec AST (wat-schema-typed)
    ↓
[reachability check]      → all referenced types exist?
[uniqueness check]        → no duplicate operation names?
[path validity check]     → valid HTTP path patterns?
[method validity check]   → valid HTTP methods?
[auth taxonomy check]     → all auth variants registered?
    ↓
typed Spec value
    ↓
exported to consumer crates (server + client)
```

Compilation errors are TYPED. Spec compilation failures fail
fast, at the spec-crate's compile time, before any consumer
crate even tries to depend on it.

The exported `Spec` value is opaque to consumers — they read
operations via accessor functions, never reach into the
internal representation. This means future spec format
changes don't break consumer crates.

## Type bridging — how server and client read the spec

### Server side (arc 015)

```scheme
(:wat::http::api::server::generate :my-server
  :spec :MyApi)
```

The server crate iterates `(:Spec/operations spec)`; for each
operation, it:
1. Registers a route on wat-http-router
2. Configures input validator (from operation's input shape)
3. Configures output validator (from operation's output shape)
4. Wires auth middleware (from operation's auth declaration)
5. Wires rate-limit middleware (from operation's rate-limit)
6. Generates a handler-signature stub the user implements

The handler signature is derived from the spec; the user can't
declare a handler that doesn't match the spec.

### Client side (arc 016)

```scheme
(:wat::http::api::client::generate :my-client
  :spec :MyApi
  :base-url "https://api.example.com")
```

The client crate iterates `(:Spec/operations spec)`; for each
operation, it:
1. Generates a typed call function (e.g., `:my-client/create-user`)
2. The function signature accepts the input shape; returns
   `:Result<output-type, errors-union>`
3. Auth header injection per operation's auth declaration
4. Path interpolation for path params

The user calls these functions; the substrate handles serialization,
validation, transport.

## The error contract

Every operation declares which error variants it can return. The
server's handler signature includes those variants in its
`:Result`; the compiler enforces that handlers can't return
undeclared variants.

```scheme
;; Spec says: :create-user errors are [:Conflict :BadRequest]
;; Generated handler signature:
;;   :Handler<:CreateUserRequest>
;;   -> :Result<:User, (:or :Conflict :BadRequest)>

;; Trying to return :NotFound from this handler is a TYPE ERROR.
```

Same on client side: the client's call function returns the
exact union of declared errors. Decoding handles all variants.

## EDN as wire; JSON via wat-edn

Per the established wat pattern (arc 009 — "two protocols on
same handler"):

- **EDN** is the native wire format. wat-native; richer types;
  comments; tagged literals
- **JSON** via wat-edn translation, content-type negotiated

The spec is wire-format-agnostic. Validation happens against the
typed wat value; serialization adapts per content-type.

## Per the four questions on the architecture

- **Obvious?** ✅✅ — universal pattern (Smithy/OpenAPI/gRPC); a
  user landing in a spec file knows what they're looking at
- **Simple?** ✅ — one DSL; one compiled artifact; consumers read
  the same value
- **Honest?** ✅✅✅ — server/client drift is structurally
  impossible when both compile against the same Spec value;
  this is the load-bearing earned property
- **Good UX?** ✅✅ — spec reads as documentation; auto-generation
  eliminates boilerplate; consistent error envelope; positive
  security via wat-schema everywhere

**Strong shape. ✅✅✅ Honest is the load-bearing earned property.**
The compile-time-shared-Spec design makes drift unrepresentable
in a way that no API design pattern (OpenAPI tooling that
generates separate server/client artifacts; manually maintained
typed clients; etc.) achieves. We win this for free because
both consumers reference the same wat value at compile time.

## Cross-references

- **arc 013 (wat-schema)** — types, shapes, refined types,
  rules, policies all delegated; spec aggregates schemas
- **arc 015 (wat-http-api-server)** — primary consumer
- **arc 016 (wat-http-api-client)** — primary consumer
- **arc 010 (wat-http-router)** — transitive (via 015)
- **arc 011 (wat-http-client)** — transitive (via 016)
- **DEPENDENCY-DOCTRINE.md** — no new external deps; uses
  wat-schema and wat substrate

## Open architectural questions

A. **Spec versioning.** Multiple versions of the same API
   coexist (e.g., v1 + v2 endpoints during migration). How does
   the spec express this? Lean: separate Spec values per version
   (`:MyApiV1`, `:MyApiV2`); composer crate that mounts both.

B. **Spec composition.** Big APIs have hundreds of operations.
   How do we organize? Lean: spec modules
   (`(:include :other-module-spec)`); reuse type aliases across
   modules.

C. **Streaming responses (SSE; WebSocket; HTTP/2 push).** How
   do operations declare streaming output? Out of scope for v1;
   sibling arc later if real consumer asks.

D. **Spec-derived doc generation.** Should this crate ship a
   doc generator, or sibling crate? Lean: sibling
   (`wat-http-api-doc`?); keep this crate focused on the spec
   primitive.

E. **Spec-derived mock server.** Same pattern; sibling crate
   (`wat-http-api-mock`?) generates a server that returns
   spec-conforming canned responses.

F. **Spec-derived contract tests.** Sibling crate
   (`wat-http-api-contract-test`?) verifies a real server
   implementation matches the spec.

G. **Cross-language SDK generation.** Could the wat spec produce
   TypeScript / Go / Python clients (like Smithy does)? Out of
   scope for v1; the spec is wat-shaped value; future arc could
   add a translator if needed.

## What's NOT in scope

- **Server implementation** — that's arc 015
- **Client SDK** — that's arc 016
- **Doc / mock server / contract test generators** — sibling
  arcs later
- **Cross-language SDK generators** — sibling arcs later
- **Runtime spec mutation** — spec is compile-time; no
  runtime add/remove operations (use a different design pattern
  for dynamic APIs)
- **WebSocket / SSE / streaming output operations** — different
  shape; sibling arc if needed
