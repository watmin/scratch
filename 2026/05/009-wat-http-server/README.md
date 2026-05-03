# wat-http-server — minimal HTTP handler interface (Rack analog)

User direction (2026-05-03):

> *"i think we need to model something like a ruby rack for an
> http server.... we should probaly build on top of tokio?... a
> tokio router or whatever can be expressed in wat and then wat
> can take over to do serving.. we just build on top of the
> rock solid http layer the rust ecosystem brings?...*
>
> *i /really/ like the ruby rack http interface solution....
> elegance..."*

Direction on naming (after applying `/gaze`):

> *"i think we must name them http.... serve could collide
> with... anything.. and route... also collidable.. wat-http-server
> and wat-http-router feel good?.."*

Naming locked: `wat-http-server` (this arc) + `wat-http-router`
(arc 010, depends on this one). Per gaze: `http-` prefix
prevents collision; the verbs (`serve`, `route`) describe the
actions; both names speak unambiguously.

---

## What wat-http-server is

The Ruby Rack equivalent for wat. **A minimal HTTP handler
interface specification** — one function signature; everything
composes from it.

A handler is a function:

```scheme
(:wat::http::server::Handler
  (request :wat::http::server::Request)
  -> :wat::core::Result<:wat::http::server::Response,
                        :wat::http::server::HandlerError>)
```

Middleware are higher-order functions that wrap handlers
(handler → handler). Composition is function composition. Same
elegance as Rack: ONE signature; everything else is composition.

## Architecture in one paragraph

The wat-vm doesn't reinvent HTTP. We build on top of the Rust
ecosystem's battle-tested HTTP machinery (tokio + hyper) via a
minimal Rust shim. The shim handles the network layer; the
wat-vm handles the application layer. The boundary is the
handler signature: Rust gives the wat handler a typed Request;
the wat handler returns a typed Response (or HandlerError); the
shim serializes it back out as HTTP.

**Transport is configuration, not contract.** The same handler
serves over TCP or Unix domain sockets — listener choice is a
deployment-time decision, invisible at the handler signature.
The common production pattern is dual-bind: UDS for trusted
sidecar→app traffic (zero TCP/IP stack overhead; filesystem
permissions as access control); TCP loopback for compatibility
(kubelet probes; Prometheus; tooling). See `DESIGN.md` →
*Transport — listener as configuration*.

## Layering

```
LAYER 4 — wat-http-router   DSL on top (arc 010)
LAYER 3 — wat-http-server   Minimal handler interface (THIS ARC)
LAYER 2 — Rust shim        tokio + hyper; HTTP IO; dispatch to wat-vm
LAYER 1 — Rust ecosystem   tokio runtime; hyper HTTP/1+2; tower
```

Layer 2 is where this arc's Rust code lives. Layer 3 is where
the wat-side handler interface and middleware machinery live.
Layer 4 (the routing DSL) is arc 010's territory.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-http-server/`
per the arc-013 pattern. Same shape as wat-fmt / wat-lint /
wat-cov / wat-doc.

```
wat-rs/crates/wat-http-server/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-edn (request body parsing for EDN),
                       #   tokio, hyper, tower
  src/                 # Rust shim (tokio + hyper + dispatch)
  wat/http/server/      # Handler / Middleware / Request / Response
                       # types and combinators in wat
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
```

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: Rust shim layering, handler interface, middleware composition, Request/Response types, error model, connection to wat-network deployment. |
| `SLICE-PLAN.md` | Slices for shipping. (Will be sized once design firms up.) |

## Conventions inherited

From the foundation-tier arcs (003-008):

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Developer-first output (EDN canonical; JSON via wat-edn)
- LLM out until delegate
- Failure engineering at the architectural layer
- Type contract enforces what convention would otherwise hope for

## Cross-references

- **arc 010 (wat-http-router)** — depends on this arc; the
  Sinatra-equivalent routing DSL on top
- **arc 007 (RemoteProgram)** — the complementary outbound
  side. RemoteProgram is wat calling OUT; wat-http-server is
  wat receiving IN. Together they close the loop on typed
  HTTP at both directions.
- **arc 008 (wat-kwargs)** — wat-http-router depends on it;
  this arc benefits indirectly through clean route-handler
  declarations
- **WAT-NETWORK.md** — the deployment story (k8s + istio
  sidecar in front of the wat-http-server app) is exactly this
  arc's deployment pattern
- **Q-channel (arc 007)** — wat-http-server can expose TWO
  protocols: plain REST-shaped HTTP for non-wat clients AND
  the wat-wire Ok/Err channel-discriminated EDN format for
  wat-network peers
- **Foundation toolkit (003-006)** — wat-http-server apps are
  wat code; they get fmt / lint / cov / doc for free

## Status

- **Captured:** 2026-05-03
- **Naming:** locked via gaze; `wat-http-server` (verb-shaped;
  scoped under `http-`; no collision)
- **Architecture:** sketched; design firms up via chat
  iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:** RemoteProgram (007)
  has answered enough of its open questions that the wire
  protocol is firm; user signals "let's start"
