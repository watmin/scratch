# wat-http-router — HTTP routing DSL (Sinatra analog)

User direction (2026-05-03):

> *"i think we can also build something like a sinatra on top
> of it?.. those two pair so well..."*

Direction on naming (after applying `/gaze`):

> *"i think we must name them http.... serve could collide
> with... anything.. and route... also collidable.. wat-http-server
> and wat-http-router feel good?.."*

Naming locked: `wat-http-router` (this arc) on top of
`wat-http-server` (arc 009). Per gaze: `http-` prefix prevents
collision; `route` describes the action; speaks
unambiguously.

---

## What wat-http-router is

The Ruby Sinatra equivalent for wat. **A routing DSL** that
sits on top of wat-http-server's minimal handler interface and
makes declarative HTTP applications ergonomic to write.

Where wat-http-server gives you ONE handler signature, this arc
gives you the syntactic sugar for splitting application logic
across many route handlers indexed by (method, path-pattern):

```scheme
;; The Sinatra-style DSL in wat
(:wat::http::router::define-app :my-app
  (:get "/users/:id"
    :handler get-user)

  (:post "/users"
    :handler create-user)

  (:get "/health"
    :handler (:wat::core::lambda
      ((req :Request) -> :Result<:Response, :HandlerError>)
      (:Result/ok (:Response/ok :body "OK")))))
```

The DSL compiles to a single `wat-http-server::Handler` —
nothing more, nothing less. The router IS a handler.

## How it pairs with wat-http-server

```
LAYER 4 — wat-http-router   THIS ARC: routing DSL → :Handler
LAYER 3 — wat-http-server   arc 009: minimal :Handler interface
LAYER 2 — Rust shim        tokio + hyper (in arc 009's crate)
LAYER 1 — Rust ecosystem   tokio runtime; hyper HTTP/1+2
```

**wat-http-router compiles route definitions to a
wat-http-server::Handler.** It owns no network code; no
tokio; no hyper. It's pure wat: pattern matching on
`Request.method` and `Request.path` to dispatch to the
appropriate handler.

This is exactly the Sinatra/Rack relationship in Ruby.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-http-router/`
per the arc-013 pattern.

```
wat-rs/crates/wat-http-router/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-http-server (../wat-http-server)
  src/                 # (probably empty / minimal — pure wat)
  wat/http/router/      # The DSL: define-app, route matching,
                       #   path pattern parser, method dispatch
  wat-tests/           # wat-level tests
  tests/               # Rust harness wrapping wat-tests/
```

**Likely no Rust code at all** for this crate. Pure wat,
because the routing logic is pure wat-vm computation. The DSL
is implemented via wat-kwargs (arc 008) for ergonomic route
declarations.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: route table compilation, path pattern matching, method dispatch, route → handler composition, kwarg-based syntax. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arc 009 firms up.) |

## Conventions inherited

From the foundation-tier arcs (003-008) and arc 009:

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Developer-first output
- Failure engineering at the architectural layer
- Type contract enforces correctness

## Cross-references

- **arc 009 (wat-http-server)** — the FOUNDATION this depends on.
  Routes compile to a wat-http-server::Handler; the DSL is
  syntactic sugar around the simpler primitive.
- **arc 008 (wat-kwargs)** — the route declaration DSL uses
  kwargs heavily (`(:get "/path" :handler my-fn)`). Direct
  dependency.
- **arc 007 (RemoteProgram)** — RemoteProgram is wat calling
  OUT typed; wat-http-server receives IN typed; wat-http-router
  organizes the IN side declaratively. Same closing-the-loop
  story as arc 009's cross-references.
- **WAT-NETWORK.md** — same deployment story as wat-http-server;
  this arc just makes the application code prettier.

## Status

- **Captured:** 2026-05-03
- **Naming:** locked via gaze; `wat-http-router` (verb-shaped;
  scoped under `http-`; pairs cleanly with `wat-http-server`)
- **Architecture:** sketched; design firms up via chat
  iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. arc 009 (wat-http-server) has shipped slice 1 (or close)
  2. arc 008 (wat-kwargs) has shipped its slice 1
  3. User signals "let's start"

## Why this is a separate arc

Could wat-http-router live inside wat-http-server? Yes. So why
separate?

Three reasons aligned with the four questions:

1. **Honest layering.** wat-http-server is the minimum
   interface; wat-http-router is opinionated DSL. Honest to
   keep them at separate crates so users who want bare-bones
   handler composition don't pull in the routing DSL.

2. **Simple shape.** Each crate does one thing. Same elegance
   as the Rack/Sinatra split in Ruby — Rack is the interface;
   Sinatra is one of many DSLs that target Rack. wat-http-router
   is one of potentially many routing DSLs that could target
   wat-http-server. (Roda-equivalent? Hanami-equivalent? Any
   future routing DSL ships independently.)

3. **Obvious dependency direction.** A wat application using
   wat-http-router obviously depends on wat-http-server; the
   reverse is never true. Making this explicit at the crate
   boundary documents the relationship in the package graph.
