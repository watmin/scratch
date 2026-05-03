# wat-http-client — HTTP client (the other end of arc 009)

User direction (2026-05-03):

> *"i think we need a wat-http-client or something to handle
> the other end of this?..."*

Naming locked: `wat-http-client` (gaze: `client` is the
universal noun for this role; pairs cleanly with
`wat-http-server`; same `http-` prefix discipline as 009/010).
The server/client pair is a universal pattern; using the
universal nouns keeps the substrate readable across language
boundaries.

---

## What wat-http-client is

The other end of `wat-http-server` (arc 009). A wat-side
HTTP client that lets a wat program make outbound HTTP
requests — to other wat-http-server apps, to third-party REST
APIs, to any HTTP endpoint on the public web.

A client invocation is a function call:

```scheme
(:wat::http::client::call
  :method :get
  :url "https://api.example.com/users/42"
  :headers (:wat::core::HashMap :String "accept" "application/json"))
;; => :Result<:Response, :ClientError>
```

The signature mirrors arc 009's handler — `Request` in,
`Result<Response, Error>` out. Same Request/Response types
where it makes sense; same error model shape; same
plaintext-handler discipline (compression handled
transparently by the underlying client).

## Why a separate arc from arc 009

The same `Request` / `Response` types serve both directions of
HTTP, but the **machinery** is different:

- **Server-side (arc 009):** accept connections; route to
  handlers; serialize responses
- **Client-side (this arc):** establish connections; send
  requests; await responses

These have different concerns:
- Connection pooling (per-host limits; idle timeouts)
- DNS resolution
- TLS client config (cert validation; SNI; ALPN)
- Redirect handling
- Retry semantics (or deliberately *not* — see DESIGN.md)
- Proxy support (HTTP_PROXY, HTTPS_PROXY env vars)
- Timeouts (connect; read; write; total)

The Rust ecosystem provides this via **reqwest** (built on
hyper). wat-http-client is the wat-side wrapper.

## Layering

```
LAYER 4 — application code     wat handlers calling out
LAYER 3 — wat-http-client      Wat-side interface (THIS ARC)
LAYER 2 — Rust shim            reqwest client; connection pool;
                                 dispatch from wat-vm
LAYER 1 — Rust ecosystem       reqwest; hyper; rustls; tokio
```

Same shape as arc 009 inverted. Layer 1 is the same Rust
ecosystem; Layer 2 is a different shim (client semantics
instead of server semantics); Layer 3 is the wat-side client
interface; Layer 4 is application code that uses it.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-http-client/`
per the arc-013 pattern. Same shape as wat-fmt / wat-lint /
wat-cov / wat-doc / wat-http-server.

```
wat-rs/crates/wat-http-client/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-edn (response body parsing for EDN),
                       #   reqwest, tokio, rustls
  src/                 # Rust shim (reqwest + dispatch to wat-vm)
  wat/http/client/     # client function, Request/Response/Error
                       #   types in wat
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
                       #   (mock servers; real-network smoke)
```

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: reqwest-based shim, connection pooling, TLS config, transport options (TCP / UDS), compression handling, retry policy, error model, Request/Response sharing with arc 009. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arc 009 firms up.) |

## Conventions inherited

From the foundation-tier arcs (003-008) and arcs 009/010:

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: errors are typed; no exceptions; caller
  decides retry/recovery
- Type contract enforces what convention would otherwise hope for

## Cross-references

- **arc 009 (wat-http-server)** — the natural pair. Same Request
  and Response types where possible; same plaintext-handler
  discipline; same UDS-or-TCP transport flexibility.
- **arc 007 (RemoteProgram)** — sits ON TOP of wat-http-client
  for the wat-network-typed-RPC case. RemoteProgram is "call
  this typed function on a wat-network peer"; wat-http-client
  is "make this HTTP request to this URL." Different
  abstraction levels; complementary.
- **arc 010 (wat-http-router)** — the routing DSL on top of
  arc 009. wat-http-client doesn't have a comparable "routing"
  layer — clients don't route — but config-pattern crates
  (an `http-client-builder` or similar) could ship later if
  needed.
- **WAT-NETWORK.md** — the wider wat-network architecture.
  RemoteProgram is the typed-RPC layer; wat-http-client is the
  general-purpose HTTP layer beneath it.

## Status

- **Captured:** 2026-05-03
- **Naming:** locked via gaze; `wat-http-client` (universal
  noun; pairs with `wat-http-server`)
- **Architecture:** sketched; design firms up via chat
  iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. arc 009 (wat-http-server) has shipped slice 1 (so the
     Request/Response types are firm and shareable)
  2. arc 007 (RemoteProgram) wire protocol decisions firm enough
     to know what wat-http-client needs to expose for it
  3. User signals "let's start"