# wat-http-api-client — DESIGN

The opinionated typed client SDK generator. Consumes a
wat-http-api-spec; produces typed call functions; symmetric
to arc 015 across the wire.

---

## The four questions are the design compass

- **Obvious?** A user calling `(:my-client/create-user :input ...)`
  knows exactly what's happening; the function name maps 1:1
  to the spec operation.
- **Simple?** One generation step; one typed function per
  operation; consistent error handling.
- **Honest?** Call signatures derived from the spec; the user
  can't construct an invalid request; outputs validated against
  spec; errors typed exhaustively.
- **Good UX?** Spec + a few configuration kwargs at client
  creation = a complete typed SDK; user code looks like calling
  local functions.

## Architecture

```
                    SPEC (arc 014)
                         │
                         │ compile-time read
                         ▼
┌────────────────────────────────────────────────────────┐
│ wat-http-api-client (THIS ARC)                         │
│                                                        │
│   For each :operation in spec:                         │
│     - Generate a typed call function                   │
│     - Function signature: input shape →                │
│         :Result<output, errors>                        │
│     - Body: serialize input → wat-http-client call →   │
│         deserialize output → validate → return         │
│     - Path interpolation (path params → URL)           │
│     - Query param encoding                             │
│     - Header injection (Content-Type; Accept; Auth)    │
│     - Error variant decoding (status code → variant)   │
│                                                        │
│   Client configuration (at generation time):           │
│     - :base-url                                        │
│     - :auth (credential source)                        │
│     - :timeout (default per call)                      │
│     - :retry (caller's responsibility — default :never)│
└────────────────────────┬───────────────────────────────┘
                         │ produces
                         ▼
                   Typed call functions
                  (wat-vm callable forms)
```

The output is a set of wat functions, one per spec operation.
User code calls them like any other function. No client-object;
no method-chaining. Just typed function calls.

## Generation — what the macro does

```scheme
(:wat::http::api::client::generate :my-client
  :spec     :MyApi
  :base-url "https://api.example.com"
  :auth     (:bearer-token-from-env "API_TOKEN")
  :timeout  (:Duration/seconds 30))
```

At compile time:

1. **Read the spec.** Iterate operations.
2. **For each operation:**
   - Generate a typed call function named `:my-client/op-name`
   - Build the function signature from spec input + output + errors
   - Generate function body:
     - Construct path (interpolate path params)
     - Construct query string (encode query params)
     - Construct headers (Content-Type, Accept, Auth-from-config)
     - Serialize input body (via wat-edn → EDN bytes; or JSON if
       Content-Type chosen)
     - Call wat-http-client (the bare HTTP layer)
     - Receive response
     - Match status code against spec output + error declarations
     - Deserialize body (per Content-Type)
     - Validate against spec output shape OR error variant body shape
     - Return `:Ok output` OR `:Err :Variant` with typed value
3. **Generate the input constructors** (typed; per spec input
   shape).

Compile-time: spec read, validators built, function signatures
typed.

Run-time: call functions invoked, requests sent, responses
validated.

## Typed call function shape

For an operation:

```scheme
(:operation :create-user
  :method   :post
  :path     "/users"
  :input    :CreateUserRequest
  :output   (:status 201 :body :User)
  :errors   [:Conflict :BadRequest])
```

Generated function:

```scheme
(:wat::core::define
  (:my-client/create-user
    :input :CreateUserRequest
    -> :Result<:User, (:or :Conflict :BadRequest)>)
  ...generated body...)
```

User calls:

```scheme
(:wat::core::let
  ((req (:CreateUserRequest
          :email    "user@example.com"
          :name     "Alice"
          :password "secret-secret"))
   (result (:my-client/create-user :input req)))
  (:wat::core::match result
    ((:Ok user) ...success...)
    ((:Err :Conflict :message m)   ...handle conflict...)
    ((:Err :BadRequest :violations vs) ...handle validation...)))
```

Path-param operations:

```scheme
;; Spec:
(:operation :get-user
  :method      :get
  :path        "/users/:id"
  :path-params (:shape (:id :UserId))
  :output      (:status 200 :body :User)
  :errors      [:NotFound])

;; Generated function:
(:my-client/get-user :id user-id)
;; -> :Result<:User, :NotFound>
```

Path params are explicit kwargs. Same naming as the spec.

## Auth header injection

Configured at client creation:

```scheme
;; Static token
(:wat::http::api::client::generate :my-client
  :spec :MyApi
  :auth (:bearer-token "static-secret-token"))

;; Token from environment variable
(:wat::http::api::client::generate :my-client
  :spec :MyApi
  :auth (:bearer-token-from-env "API_TOKEN"))

;; Token from a callable (refreshable)
(:wat::http::api::client::generate :my-client
  :spec :MyApi
  :auth (:bearer-token-from-fn :token-refresher-fn))

;; mTLS (client cert + key from files; for wat-network peers)
(:wat::http::api::client::generate :my-client
  :spec :MyApi
  :auth (:mtls
          :cert-pem (:read-file "/etc/ssl/client.pem")
          :key-pem  (:read-file "/etc/ssl/client.key")))

;; API key in custom header
(:wat::http::api::client::generate :my-client
  :spec :MyApi
  :auth (:api-key :header "X-API-Key" :value "secret-key"))
```

Each call function automatically injects the appropriate auth
headers. The user doesn't manually attach credentials; the
config does it once.

## Error variant decoding

The spec declares which errors each operation can return. The
generated call function decodes responses:

1. If status code matches the spec's `:output :status`, deserialize
   as the output shape. Validate. Return `:Ok output`.
2. If status code matches a spec error variant, deserialize body
   as the variant's body shape. Return `:Err variant`.
3. If status code doesn't match anything declared in the spec,
   return `:Err (:Custom :status N :body bytes)` — escape hatch
   for unexpected responses.

The caller pattern-matches the result. Exhaustive matching is
enforced by the type system.

## Retry semantics — deferred to caller

Per arc 011's failure-engineering position:

> *"wat-http-client does not retry. A failed call returns
> Err(ClientError::...). The caller decides retry semantics."*

This holds for wat-http-api-client too. **No automatic retries.**

The spec's `:idempotent? true` declaration is INFORMATIONAL —
the call function exposes the hint via a `(:Operation/idempotent?
op)` accessor, but the generated call doesn't act on it.

Caller-side patterns (out of scope for this crate; sibling
crates if patterns settle):

```scheme
;; Caller writes their own retry loop
(:wat::core::let-loop
  ((attempt 0))
  (:wat::core::match (:my-client/get-user :id user-id)
    ((:Ok u) (:Result/ok u))
    ((:Err (:NetworkError ...)))
      (:if (:< attempt 3)
        (:recur (:+ attempt 1))
        (:Result/err (:RetriesExhausted)))
    ((:Err other) (:Result/err other))))

;; Or use a sibling combinator (future)
(:wat::http::retry::with-backoff
  :max-attempts 3
  :backoff      (:exponential :base-ms 100 :max-ms 5000)
  :retry-on     [:NetworkError :Timeout]
  :call         (:my-client/get-user :id user-id))
```

## Per the four questions on the architecture

- **Obvious?** ✅✅ — call functions named after spec operations;
  signatures derived from spec; user code reads as local
  function calls
- **Simple?** ✅ — one function per operation; one config-time
  setup; no client-object juggling; no method chains
- **Honest?** ✅✅✅ — input shape constructor enforces validity;
  output validated against spec; error variants exhaustively
  typed; caller can't pattern-match on a non-existent variant;
  drift impossible because the spec IS the source of truth
- **Good UX?** ✅✅ — spec + minimal config = full SDK; familiar
  to anyone who's used a generated SDK from Smithy/OpenAPI/gRPC

**Strong shape. ✅✅✅ Honest is real** — the symmetry across
the wire (server compiles against same Spec; client compiles
against same Spec) makes drift unrepresentable. Same triple-
checkmark class as arc 015.

## Cross-references

- **arc 014 (wat-http-api-spec)** — the contract; hard dep
- **arc 011 (wat-http-client)** — outbound transport; hard dep
- **arc 013 (wat-schema)** — validation; transitive
- **arc 015 (wat-http-api-server)** — symmetric across the wire;
  same spec; drift impossible
- **WAT-NETWORK.md** — `:auth (:mtls ...)` integrates with
  wat-network identity layer
- **DEPENDENCY-DOCTRINE.md** — no new external deps

## Open architectural questions

A. **Async vs blocking call style.** Should generated functions
   block, or return async tasks? Lean: blocking (CSP-style;
   matches wat's everywhere-blocking model); async happens at
   the wat-vm/runtime layer per ASYNC-COEXISTENCE pattern;
   user sees blocking calls.

B. **Streaming responses.** Spec says `:output (:stream :T)` for
   streaming endpoints? Out of scope for v1; sibling arc
   (`wat-http-api-client-stream`?) if real consumers emerge.

C. **Pagination conventions.** OpenAPI / Smithy / gRPC have
   first-class pagination support. Should the spec encode
   pagination patterns? Out of scope for v1; sibling arc
   later or extension.

D. **Mock-server-from-spec.** The spec is enough information
   to generate a deterministic mock server (returns
   spec-conforming canned responses). Useful for client testing
   without a real server. Out of scope for this arc; sibling
   crate (`wat-http-api-mock`?).

E. **Contract testing.** Verify a real server response matches
   the spec from the client side. Out of scope for v1; sibling
   crate (`wat-http-api-contract-test`?).

F. **Per-call configuration override.** Can a user override
   timeout / auth / etc. per call? Lean: yes via call kwargs
   (`:timeout`, `:auth-override`); rare but useful for
   special cases.

## What's NOT in scope

- **The spec itself** — that's arc 014
- **The server side** — that's arc 015
- **General-purpose HTTP client** — that's arc 011
- **Retry combinators** — caller's responsibility (per failure
  engineering); sibling crate later if patterns settle
- **Mock server / contract test generators** — sibling crates
- **Streaming responses** — different shape; sibling arc
- **OAuth flows / cookie-store / session management** —
  application-layer; user implements via call functions + their
  own state machine
