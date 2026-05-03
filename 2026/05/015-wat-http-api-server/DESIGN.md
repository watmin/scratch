# wat-http-api-server — DESIGN

The opinionated server skeleton that consumes a wat-http-api-spec
and auto-generates the entire boilerplate of a typed HTTP API.

---

## The four questions are the design compass

- **Obvious?** A user reading their generated app sees one form
  per operation; the spec drives everything else.
- **Simple?** One generation step; one consistent error
  envelope; predictable middleware stack.
- **Honest?** Handler signatures derived from the spec; the
  user can't accidentally implement a different contract; failure
  modes are typed per the spec.
- **Good UX?** Spec + handler bodies = a complete production
  HTTP API. Auth, validation, errors, rate limits all wired
  for free.

## Architecture

```
                    SPEC (arc 014)
                         │
                         │ compile-time read
                         ▼
┌──────────────────────────────────────────────────────┐
│ wat-http-api-server (THIS ARC)                       │
│                                                      │
│   For each :operation in spec:                       │
│     - Register a route on wat-http-router            │
│     - Wire input validator (wat-schema, input shape) │
│     - Wire output validator (wat-schema, output shape)│
│     - Wire auth middleware (per :auth declaration)   │
│     - Wire rate-limit middleware (per :rate-limit)   │
│     - Wire cache header emitter (per :cacheable?)    │
│     - Wire idempotency-key handling (per :idempotent?)│
│     - Generate the handler signature stub            │
│     - Wire the user's handler body to the signature  │
│                                                      │
│   Consistent error envelope:                         │
│     - Typed :ApiError union (per spec :errors)        │
│     - Status codes per error variant                 │
│     - Body shape per error variant                   │
└────────────────────────┬─────────────────────────────┘
                         │ produces
                         ▼
                  Handler (arc 009)
                  ready to serve
```

The output of generation is a `:wat::http::server::Handler` —
the bare handler interface from arc 009. This handler can be
wrapped in additional middleware, composed with other handlers,
or served directly via `(:wat::http::server::serve ...)`.

## Generation — what the macro does

```scheme
(:wat::http::api::server::generate :my-server
  :spec :MyApi
  :options (:strict-output? true
            :error-format    :rfc7807))
```

At compile time:

1. **Read the spec.** Iterate operations.
2. **For each operation:**
   - Look up input shape from spec; compile a validator
   - Look up output shape; compile an output-validator (optional;
     can be disabled in production for perf)
   - Look up auth declaration; instantiate the auth middleware
   - Look up rate-limit; instantiate the rate-limit middleware
   - Look up cache headers; instantiate the cache emitter
   - Compose middleware in canonical order (see below)
   - Register a route on wat-http-router with method + path
   - Generate a handler stub the user implements
3. **Compose error envelope.** Build the union of all error
   variants across operations; generate a serializer that maps
   `:ApiError` variants to HTTP responses.
4. **Return** a single `:Handler` representing the entire API.

## Middleware composition order

```
Incoming request
    ↓
[1] Rate-limit middleware (cheap reject; protect downstream)
    ↓
[2] Auth middleware (verify identity; populate :Auth context)
    ↓
[3] Input validation (parse body; validate against spec input)
    ↓
[4] Path-param validation (typed extraction)
    ↓
[5] Idempotency-key handling (if applicable; check replay cache)
    ↓
[6] User handler body — the only thing the user wrote
    ↓
[7] Output validation (validate handler output against spec output)
    ↓
[8] Response serialization (typed value → wire bytes per content-type)
    ↓
[9] Cache headers (Cache-Control / ETag per spec :cacheable?)
    ↓
[10] Error envelope (if any prior step Err'd, format consistently)
    ↓
Outgoing response
```

This order is fixed; users don't get to reshape it (per the four-
questions discipline — opinionated middleware = fewer footguns).
Custom middleware can layer ABOVE step [1] or BELOW step [10] via
arc 010's `:middleware` kwarg.

## Handler signature derivation

For an operation:

```scheme
(:operation :create-user
  :method   :post
  :path     "/users"
  :input    :CreateUserRequest
  :output   (:status 201 :body :User)
  :errors   [:Conflict :BadRequest])
```

The generated handler signature is:

```scheme
(:Handler
  (req :CreateUserRequest)
  -> :Result<:User, (:or :Conflict :BadRequest)>)
```

The user implements:

```scheme
(:wat::http::api::server::define-handler :my-server :create-user
  (:lambda ((req :CreateUserRequest)
            -> :Result<:User, (:or :Conflict :BadRequest)>)
    ...business logic...))
```

The compiler enforces:
- Handler signature matches spec (input type, output type, errors)
- Handler can ONLY return declared error variants
- All operations from the spec have a handler defined (compilation
  error if any are missing)

## Path params and query params

For an operation with path params:

```scheme
(:operation :get-user
  :method      :get
  :path        "/users/:id"
  :path-params (:shape (:id :UserId))
  :output      (:status 200 :body :User))
```

Generated signature:

```scheme
(:Handler
  (id :UserId)
  -> :Result<:User, ...>)
```

Path params become explicit handler arguments. Query params
similarly:

```scheme
(:operation :list-users
  :method       :get
  :path         "/users"
  :query-params (:shape (:limit (:i64 :range 1 200))
                        (:offset (:i64 :min 0))))
```

Generated:

```scheme
(:Handler
  (limit :i64) (offset :i64)
  -> :Result<...>)
```

Where input body + path params + query params all coexist:

```scheme
(:Handler
  (req :Body) (id :PathId) (limit :Query) ...
  -> :Result<...>)
```

The signature surfaces every parameter the spec declared. No
`req.params["id"]` style untyped access; everything is named
and typed.

## Auth middleware — taxonomy

```scheme
:auth :none                ; explicitly public; no middleware
:auth (:bearer-token :scopes ["users:read"])
:auth (:basic-auth)
:auth (:api-key :header "X-API-Key")
:auth (:mtls :cert-validator :MyCertValidator)
:auth (:custom :validator :MyCustomAuthFn)
```

Each variant has a corresponding Rust shim that:
1. Extracts the credential from the request
2. Validates it (against the configured validator)
3. Populates an `:Auth` context value
4. Either continues to the next middleware OR rejects with 401/403

The `:Auth` context is available in handler bodies via the
ambient binding pattern. Handlers don't need to re-validate
auth; the middleware already did.

For `:mtls`, integrates with the wat-network's SPIFFE identity
(see WAT-NETWORK.md). The sidecar terminates TLS; the cert
chain is forwarded as headers; the wat handler validates.

## Rate-limit middleware

```scheme
:rate-limit (:by :remote-addr :max 10/minute)
:rate-limit (:by :auth-subject :max 100/minute)
:rate-limit (:tiered
              (:by :remote-addr :max 1000/hour)
              (:by :auth-subject :max 10000/hour))
```

Each variant uses a sliding-window counter (in-process by
default; pluggable for distributed via Redis-backed counter
later). Exceeded → 429 with `Retry-After` header.

## Cache headers

```scheme
:cacheable? (:max-age 60)
:cacheable? (:max-age 300 :vary [:authorization])
:cacheable? nil   ; explicitly not cacheable
```

Generates `Cache-Control` and `Vary` headers; emits `ETag` if
the response body contains an ID-shaped field (heuristic;
overridable).

## Error envelope

Every error response uses the same envelope:

```edn
{:error/code    :Conflict
 :error/message "email already in use"
 :error/details {...spec-defined body for this variant...}
 :request/id    "uuid-of-this-request"
 :timestamp     "2026-05-03T..."}
```

Status code from the spec's error variant declaration
(`:Conflict :status 409`). Body shape from the spec's variant.
The wrapper fields (`:request/id`, `:timestamp`) are universal.

The user can override the envelope template at generation time:

```scheme
(:wat::http::api::server::generate :my-server
  :spec :MyApi
  :options (:error-format :rfc7807))   ; or :default; or :custom
```

## Per the four questions on the architecture

- **Obvious?** ✅✅ — spec drives everything visible; user code
  is just handler bodies; the rest is generated and consistent
- **Simple?** ✅ — one generate-step; one composition order;
  one error envelope; nothing exotic
- **Honest?** ✅✅✅ — handler signature derived from spec
  (compile-time enforced); user can't drift from the contract;
  every operation must have a handler (compile error if not);
  outgoing responses validated against output shape
- **Good UX?** ✅✅ — declarative; opinionated paved road;
  custom middleware composes above/below the generated stack;
  user only writes business logic

**Strong shape. ✅✅✅ Honest is real** — the contract is the
spec; the handler can't lie about what it implements; the
substrate enforces this at compile time. Same triple-checkmark
class as wat-pause (freeze invariant) and wat-http-api-spec
(structural drift impossibility).

## Cross-references

- **arc 014 (wat-http-api-spec)** — the contract; hard dep
- **arc 010 (wat-http-router)** — generated routes flow through;
  hard dep
- **arc 013 (wat-schema)** — validation; transitive
- **arc 009 (wat-http-server)** — bare handler interface;
  transitive
- **arc 016 (wat-http-api-client)** — natural complement; same
  spec
- **arc 011 (wat-http-client)** — transitive (via 016)
- **WAT-NETWORK.md** — auth declarations compose with sidecar
  mTLS + SPIFFE
- **DEPENDENCY-DOCTRINE.md** — no new external deps; all
  through existing wat-rs crates

## Open architectural questions

A. **Output validation in production?** Validating every outgoing
   response against the spec is expensive. Lean: ON in
   development; toggleable OFF in production via
   `:options (:strict-output? false)`. Spec-violating responses
   ARE bugs but at production cost may be too high.

B. **Custom middleware injection points.** Users may want to
   inject middleware between e.g. auth and validation (for
   request enrichment). Lean: support `:before-validation`,
   `:after-validation`, `:before-handler`, `:after-handler`
   hooks; not free-form "anywhere in the stack."

C. **Handler-not-implemented behavior.** What if the user
   calls `generate` but doesn't define handlers for every
   spec operation? Lean: compile error (consistent with the
   four-questions Honest discipline; no silent gaps).

D. **Operation overrides.** Can a user override the generated
   middleware for a specific operation (e.g., disable
   auth for one endpoint)? Lean: NO — the spec is the
   contract; if you want different auth, change the spec.
   The exception goes IN the spec, not around it.

E. **Multiple specs in one server.** Can a single server serve
   v1 + v2 specs simultaneously (gradual migration)? Lean: yes,
   via composition — generate v1 + v2 separately; mount under
   different base paths; combine with arc 010's `:mount` form.

## What's NOT in scope

- **The spec itself** — that's arc 014
- **The client side** — that's arc 016
- **Custom routing logic** — use wat-http-router directly if
  you need it
- **Plain HTTP handlers without spec-driven generation** — use
  wat-http-server / wat-http-router directly
- **WebSocket / SSE / streaming responses** — different shape;
  sibling arc if needed
- **Per-handler raw control over the middleware stack** —
  opinionated by design; if you need that level of control,
  you're in arc 010 territory
