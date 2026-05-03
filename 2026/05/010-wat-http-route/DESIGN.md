# wat-http-route — DESIGN

The Sinatra equivalent for wat. A routing DSL that compiles
declarative route definitions into a single
`wat-http-serve::Handler` (arc 009).

---

## The four questions are the design compass

- **Obvious?** A user reading a route definition knows what it
  matches without consulting docs.
- **Simple?** The DSL adds nothing the user couldn't write by
  hand; it's pure ergonomic sugar.
- **Honest?** A route compiles to a handler; the DSL doesn't
  hide where dispatch happens; failures land at the route's
  doorstep, not in some opaque router internals.
- **Good UX?** Route declarations should be the most readable
  part of an application; the discipline lives in the
  declarations, not the router internals.

## The Sinatra pattern, applied to wat

Ruby's Sinatra:

```ruby
get '/users/:id' do
  user = User.find(params[:id])
  user.to_json
end
```

The DSL is method × path-pattern → handler. Routes are matched
in declaration order; first match wins; path captures bind to
named parameters.

For wat:

```scheme
(:wat::http::route::define-app :my-app
  (:get "/users/:id"
    :handler (:wat::core::lambda
      ((req :Request) -> :Result<:Response, :HandlerError>)
      (:wat::core::let*
        ((id (:Request/param req "id"))
         (user (:User-service/find :id id)))
        (:Result/ok (:Response/json :body user)))))

  (:post "/users"
    :handler create-user-handler)

  (:get "/health"
    :handler (:wat::core::lambda ((req :Request)) ...)))
```

Same elegance; wat-shaped. The DSL is a wat macro that expands
to a `wat-http-serve::Handler` whose body is a pattern match on
(method, path-pattern).

## Compilation strategy

A route declaration compiles to two pieces of data:

1. **A pattern matcher** — given a request, decide which route
   matches (or none).
2. **A handler dispatch table** — once a route matches, call
   its handler with path captures bound.

Compiled output:

```scheme
;; What (define-app ...) produces under the hood
(:wat::core::define
  (:my-app -> :wat::http::serve::Handler)
  (:wat::core::lambda
    ((req :Request) -> :Result<:Response, :HandlerError>)
    (:wat::core::match (:Request/method req) (:Request/path req)
      ((:get  (:path-pattern "/users/:id"))   (:apply get-user-handler req))
      ((:post (:path-pattern "/users"))       (:apply create-user-handler req))
      ((:get  (:path-pattern "/health"))      (:apply health-handler req))
      (else (:Result/err
              (:HandlerError/NotFound :message "no route matches"))))))
```

The DSL is just sugar over `match` + path-pattern primitives.
A user could write the match expression by hand; the DSL
makes the common case ergonomic.

## Path pattern syntax

Sinatra-style path patterns; familiar to most developers.

| Syntax | Matches | Captures |
|---|---|---|
| `/users` | exact `/users` only | none |
| `/users/:id` | `/users/{anything-no-slash}` | `id` |
| `/users/:id/posts/:post-id` | nested capture | `id`, `post-id` |
| `/files/*path` | wildcard greedy | `path` |
| `/static/?` | optional trailing | none |

Path captures bind to named parameters accessible via
`(:Request/param req "id")`. The compiler turns the pattern
into a deterministic matcher (a regex equivalent or a
hand-rolled tokenized matcher).

## Method dispatch

The first DSL form is the method:

```scheme
(:get    "/path" :handler ...)
(:post   "/path" :handler ...)
(:put    "/path" :handler ...)
(:patch  "/path" :handler ...)
(:delete "/path" :handler ...)
(:head   "/path" :handler ...)
(:options "/path" :handler ...)
(:any    "/path" :handler ...)  ;; matches any method
```

These compile to the appropriate Method enum variants in the
match expression.

## Subroute composition (mounting)

Sinatra supports nesting via inheritance + `use`. wat-http-route
provides explicit composition:

```scheme
(:wat::http::route::define-app :api-v1
  (:get "/health" :handler health)
  (:get "/users/:id" :handler get-user))

(:wat::http::route::define-app :api-v2
  (:get "/health" :handler health-v2)
  (:post "/orders" :handler create-order))

(:wat::http::route::define-app :main-app
  (:mount "/api/v1" :app :api-v1)
  (:mount "/api/v2" :app :api-v2)
  (:get "/" :handler index-handler))
```

`(:mount "/prefix" :app sub-app)` strips the prefix from the
incoming request path and dispatches to the sub-app. This is
straightforward function composition: the parent app is a
handler; mounted apps are handlers; mount strips a path
prefix and forwards.

## Per-route middleware

Sometimes a route needs middleware that other routes don't.
The DSL accommodates per-route middleware via a kwarg:

```scheme
(:get "/admin/dashboard"
  :middleware (:wat::core::vec :Middleware
                :wat::http::serve::middleware::require-auth
                :wat::http::serve::middleware::require-admin)
  :handler admin-dashboard)
```

Per-route middleware compose with app-level middleware (passed
to `define-app` or applied via wat-http-serve's `compose`
combinator).

## Error handling

When no route matches:

- Default: `HandlerError::NotFound` with a generic message.
- Customizable via `(:not-found :handler ...)` declaration in
  the app.

When a route's handler returns `Err`:

- The error propagates up to wat-http-serve, which converts it
  to an HTTP response per the standard error model.
- Routes can override error rendering via
  `(:error-handler :for :NotFound :handler ...)` for custom
  not-found bodies, etc.

## Per the four questions

- **Obvious?** ✅ — route declarations read like Sinatra; pattern
  is universally familiar to web developers
- **Simple?** ✅ — DSL is sugar over match + path patterns;
  nothing the user couldn't write by hand
- **Honest?** ✅ — what a route declaration says is what gets
  matched; no hidden routing magic; failures land at the
  declared doorstep; type contracts enforced at the handler
  signature boundary
- **Good UX?** ✅ — the most readable part of an application
  is its routes; familiar pattern; kwarg-based syntax means
  optional features (middleware, content-type filters) don't
  bloat the common case

Strong shape but no triple-checkmarks here. Honest is good
but ordinary; this arc is ergonomics on a primitive that
already shipped its honest contract (arc 009).

## Connection to arc 008 (wat-kwargs)

The DSL leans heavily on kwargs. Each route declaration is a
kwarg-style call:

```scheme
(:get "/users/:id"
  :handler get-user
  :middleware (...)
  :content-type "application/json")
```

This is exactly arc 008's territory — auto-generated kwarg
variants from function signatures. The route DSL compiles
each `(:get path :handler h ...)` form into a route record
with optional fields populated from the kwargs.

## Connection to arc 009 (wat-http-serve)

A complete app:

```scheme
;; Define routes
(:wat::http::route::define-app :my-app
  (:get  "/health" :handler health)
  (:post "/users"  :handler create-user)
  (:get  "/users/:id" :handler get-user))

;; Apply app-level middleware
(:wat::core::define
  (:my-app-with-middleware -> :Handler)
  (:wat::http::serve::compose
    (:wat::core::vec :Middleware
      :wat::http::serve::middleware::log
      :wat::http::serve::middleware::compress)
    :my-app))

;; Run via wat-http-serve
(:wat::http::serve::serve
  :handler :my-app-with-middleware
  :port 8080)
```

The route DSL produces a wat-http-serve::Handler; that handler
gets composed with middleware via wat-http-serve combinators;
the result is served via wat-http-serve's listener. **Each
arc owns one concern; together they compose.**

## Open architectural questions

A. **Path pattern matching backend.** Hand-rolled tokenized
   matcher vs compile-to-regex. Lean: hand-rolled for
   transparency and absence of regex dependency; can pivot if
   benchmarks show otherwise.

B. **Trailing slash semantics.** Strict (`/users` ≠ `/users/`)
   vs lenient (both match). Lean: strict by default; lenient
   via opt-in flag on `define-app`.

C. **Path normalization.** Should `/users/../admin` reach the
   `/admin` route? Lean: no; reject path traversal at the
   normalization step before matching.

D. **Static file serving.** Sinatra has `set :public, ...`.
   Out of scope for v1? Lean: yes, out of scope; ship a
   sibling `wat-http-static` crate later if the need is real.

## What's NOT in scope

- **HTTP listener** — that's wat-http-serve. This arc only
  produces a handler.
- **Static file serving** — separate crate if needed.
- **Template rendering** — separate crate if needed.
- **Sessions / cookies / auth** — application-layer middleware,
  not routing concerns.
- **WebSocket / SSE** — different shape; sibling arc to
  wat-http-serve.

## Pure-wat philosophy

Unlike wat-http-serve (which has a real Rust shim layer),
wat-http-route is pure wat. The crate may have NO Rust source
code at all. The pattern matcher, the dispatch table, the
mount mechanism — all expressible in wat-vm.

This is intentional. Routing logic is computation, not IO. It
should live in the layer that handles computation. The
substrate (wat-rs + wat-http-serve + arc 008's kwargs) is
expressive enough to handle it.

If we find ourselves reaching for Rust code in this crate, that's
a signal we got the abstraction wrong.
