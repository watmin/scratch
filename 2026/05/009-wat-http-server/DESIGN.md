# wat-http-server — DESIGN

The Ruby Rack equivalent for wat. Minimal HTTP handler
interface specification + Rust shim that uses tokio + hyper
for the network layer.

---

## The four questions are the design compass

Per the established discipline (carried from arcs 003-008):

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape
  per concept.
- **Honest?** What's named matches what's there; the boundary
  between Rust network and wat application is honest.
- **Good UX?** A user can do the right thing without ceremony.

## The Rack pattern, applied to wat

Ruby's Rack defines a minimal interface: `call(env)` returns
`[status, headers, body]`. Everything else (Sinatra; Rails;
Roda; Puma; Unicorn) sits on top of this signature. The
elegance is one-signature-everything-composes-from-it.

For wat:

```scheme
;; The minimal handler signature
(:wat::http::server::Handler
  (request :wat::http::server::Request)
  -> :wat::core::Result<:wat::http::server::Response,
                        :wat::http::server::HandlerError>)
```

A handler is a function. Period. Middleware are higher-order
functions that wrap handlers (handler → handler). Composition
is function composition. **Same elegance; wat-shaped.**

## Architecture — the four layers

```
┌───────────────────────────────────────────────────────────────┐
│ LAYER 4 — wat-http-router                                      │
│           (arc 010; the Sinatra DSL on top)                   │
│           routing; path params; HTTP method dispatch          │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           │ depends on Layer 3
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ LAYER 3 — wat-http-server (THIS ARC)                           │
│           Handler / Middleware / Request / Response in wat    │
│           Composition; error model; type contracts            │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           │ Rust shim invokes wat handler
                           │ via wat-vm; serializes Request/Response
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ LAYER 2 — Rust shim (THIS ARC)                                │
│           tokio runtime; hyper service; HTTP parse/write      │
│           dispatches each request to the wat handler          │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           │ standard Rust async
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ LAYER 1 — Rust HTTP ecosystem                                 │
│           tokio runtime; hyper HTTP/1+2; tower middleware     │
│           (we don't reinvent any of this)                     │
└───────────────────────────────────────────────────────────────┘
```

**We don't compete with hyper.** We sit on top of it. Hyper does
HTTP at line rate; tokio handles async; tower provides
middleware infrastructure at the network layer if needed; the
wat-vm handles application logic.

## The wat-side interface

### Request type

```scheme
(:wat::core::struct :wat::http::server::Request
  ((method   :wat::http::server::Method)        ; :get :post :put :delete :patch ...
   (path     :wat::core::String)
   (query    :wat::core::HashMap<:String, :String>)
   (headers  :wat::core::HashMap<:String, :String>)
   (body     :wat::http::server::Body))         ; :bytes / :stream / :empty
  "An HTTP request as seen by a wat-http-server handler.

   Receives raw HTTP from the Rust shim; type-bridged into wat
   for handler consumption. All headers normalized to lowercase.
   Body type depends on Content-Length / Transfer-Encoding;
   handlers can pattern-match on the body shape.")
```

### Response type

```scheme
(:wat::core::struct :wat::http::server::Response
  ((status   :wat::core::i64)                  ; 200, 404, 500, etc.
   (headers  :wat::core::HashMap<:String, :String>)
   (body     :wat::http::server::Body))
  "An HTTP response emitted by a wat-http-server handler.

   The Rust shim serializes back to HTTP; status is the
   integer code; headers are name-value pairs (lowercase
   normalized); body is bytes or stream.")
```

Constructors via wat-kwargs (arc 008) for ergonomics:

```scheme
(:wat::http::server::Response/ok :body "hello world")
(:wat::http::server::Response/not-found :body "user not found")
(:wat::http::server::Response/json :body data)
(:wat::http::server::Response/builder
  :status 418
  :headers (:wat::core::HashMap :String "x-extra" "metadata")
  :body "I'm a teapot")
```

### Handler signature

```scheme
(:wat::http::server::Handler
  (request :Request)
  -> :wat::core::Result<:Response, :HandlerError>)
```

A handler is just a function with this signature. No special
class; no inheritance; no decorator. Just a function.

### Middleware — handler wrappers

```scheme
;; Middleware: handler -> handler
(:wat::http::server::Middleware
  (next :Handler)
  -> :Handler)

;; Concrete example: logging middleware
(:wat::core::define
  (:wat::http::server::middleware::log
    (next :Handler)
    -> :Handler)
  "Wrap a handler with request/response logging."
  (:wat::core::lambda
    ((req :Request) -> :Result<:Response, :HandlerError>)
    (:wat::core::let*
      ((_ (:wat::io/log :info (:wat::core::concat
                                "request: "
                                (:Request/method req)
                                " "
                                (:Request/path req))))
       (resp (next req)))
      resp)))

;; Compose: middleware . middleware . middleware . handler
(:wat::core::define
  (:my-app -> :Handler)
  (:wat::http::server::compose
    (:wat::core::vec :Middleware
      :wat::http::server::middleware::log
      :wat::http::server::middleware::compress
      :wat::http::server::middleware::cors)
    :my-leaf-handler))
```

`compose` is just function composition — folds the middleware
list around the leaf handler.

### Error model

```scheme
(:wat::core::enum :wat::http::server::HandlerError
  ((BadRequest      (message :String)))     ; 400 — client's fault
  ((Unauthorized    (message :String)))     ; 401
  ((Forbidden       (message :String)))     ; 403
  ((NotFound        (message :String)))     ; 404
  ((Conflict        (message :String)))     ; 409
  ((InternalError   (message :String)
                    (cause :HolonAST)))     ; 500 — server's fault
  ((Custom          (status :i64)
                    (body :String))))       ; arbitrary status + body
```

The Rust shim converts `Err(handler-error)` to the appropriate
HTTP status response. **The handler signature includes its
error type explicitly.** No exception escapes the handler;
no panic produces a 200 OK silently. Failure engineering at
the handler boundary.

## Rust shim — what Layer 2 does

```rust
// pseudo-Rust illustrating the shim's responsibility
async fn dispatch(
    req: hyper::Request<Body>,
    wat_vm: Arc<WatVm>,
    handler_name: &str,
) -> hyper::Result<hyper::Response<Body>> {
    // 1. Parse the request
    let wat_request = parse_to_wat_request(req).await?;

    // 2. Invoke the wat handler via wat-vm
    let result = wat_vm.invoke(handler_name, wat_request)?;

    // 3. Serialize the wat response back to HTTP
    let hyper_response = match result {
        Value::Result::Ok(wat_response) => serialize(wat_response),
        Value::Result::Err(handler_error) => error_to_response(handler_error),
    };

    Ok(hyper_response)
}
```

The shim:
- Owns the tokio runtime + hyper service
- Parses incoming HTTP → wat Request
- Invokes the wat handler via the wat-vm
- Serializes wat Response → outgoing HTTP
- Maps HandlerError variants to HTTP status codes

The wat handler is the application; the shim is the network
boundary. Clean separation.

## Per the four questions

- **Obvious?** ✅ — handler is a function with one signature;
  middleware are wrappers; composition is function composition
- **Simple?** ✅✅ — one signature; everything else composes
  from it; same elegance as Rack
- **Honest?** ✅✅ — handler signature includes its error type
  explicitly; no exception escapes; tokio/hyper handle network
  honestly; wat handles application honestly; clean boundary
- **Good UX?** ✅✅ — wat code expresses HTTP servers in the
  same idiom as everything else; Rust ecosystem handles the
  hard parts; deployers use familiar k8s + istio patterns

Strong shape. ✅✅ Honest is real — the boundary is honest;
errors don't disappear; the handler signature can't lie about
what it returns.

## Two protocols on the same handler

A wat-http-server app can serve TWO PROTOCOLS over the same HTTP
endpoint:

1. **Plain REST** for non-wat clients — JSON request/response;
   standard HTTP semantics; works with any HTTP client
2. **wat-wire format** for wat-network peers — Q-channel
   Ok/Err discriminated EDN frames over HTTP body; signed
   payload validation; typed contract enforcement

The handler logic is the same; the request/response formats
differ; content-type negotiation determines which format is
used per request. This means a single wat-http-server
deployment can serve BOTH non-wat clients (any language; any
framework; standard HTTP) AND wat-network peers with full
typed-cryptographic semantics.

This is the realization of WAT-NETWORK.md's "in-network mode
vs out-of-network mode" — same handler; different protocol
layer; client-driven.

## Transport — listener as configuration

The handler signature is invariant under transport choice. The
listener is runtime configuration. This is the property that
makes wat-http-server genuinely deployment-agnostic.

The Rust shim (Layer 2) accepts any tokio listener that
produces streams implementing `AsyncRead + AsyncWrite + Unpin`.
Both transport options are first-class:

- **`tokio::net::TcpListener`** — INET sockets (cross-host;
  requires port + interface)
- **`tokio::net::UnixListener`** — UDS sockets (host-local;
  requires filesystem path + permissions)

Same hyper machinery; same wat handler invocation; same
Request/Response types. The transport vanishes at the handler
boundary.

### Configuration shape

```scheme
(:wat::http::server::serve
  :handler :my-app
  :listeners (:wat::core::vec :Listener
    (:Listener/uds :path "/var/run/wat-http-server.sock"
                   :perms 0o600)
    (:Listener/tcp :addr "127.0.0.1:8080")))
```

A wat-http-server deployment binds whatever listeners its
environment requires. The handler runs once, ignorant of which
listener delivered each request.

### Why dual-bind is the common production pattern

In a service-mesh pod, the typical bindings are:

- **UDS** for sidecar→app traffic: zero TCP/IP stack overhead;
  filesystem permissions as access control; no port to scan
- **localhost TCP (127.0.0.1)** for compatibility: kubelet
  liveness/readiness probes; Prometheus scrapers; quick `curl`
  from inside the pod for debugging; anything that doesn't speak
  UDS natively

Notably absent: `0.0.0.0` TCP. Cross-pod traffic terminates at
the istio sidecar, NOT at the wat-http-server container. The
app should not be reachable from outside the pod directly —
absence of an external listener is itself a defense.

### What this property buys

- **Performance:** UDS for sidecar→app skips L3 (IP) and L4
  (TCP) entirely. The kernel just copies bytes between two
  user processes via socket buffers. No TCP handshake; no IP
  routing; no checksum.
- **Security:** UDS-only listener means an attacker who lands
  inside the pod cannot reach the app via TCP — there's no
  TCP listener at all. Combined with `chmod 600` on the
  socket file and matching uid:gid, the socket is
  kernel-access-controlled.
- **Compatibility:** localhost TCP keeps every TCP-only client
  working without code changes. Dual-bind lets trusted callers
  use the fast path while leaving the universal path open for
  tools that don't speak UDS.

### Per the four questions on transport flexibility

- **Obvious?** ✅ — `:listeners` is a list; bind what you need
- **Simple?** ✅ — one handler; many transports; nothing
  special-cased per transport
- **Honest?** ✅✅ — handler signature structurally cannot
  include transport; the abstraction is at the right layer;
  the transport choice is real and visible at the deployment
  surface
- **Good UX?** ✅ — pod operators choose listeners per
  environment without touching application code

Not triple-honest because handlers can introspect
`req.remote_addr` or sidecar-added headers (`x-forwarded-*`,
SPIFFE id) for application-layer trust decisions. The
*signature* can't tell; the request *value* can carry
transport-derived metadata. That's the right shape — pure
invariance would prevent useful decisions like "trust this
request because the SPIFFE-verified sidecar attested it."

### A layer was eliminated, not erased

For the sidecar→app traffic path, L3 (IP) and L4 (TCP)
genuinely vanish from the data path. The kernel does in-process
buffer copy; no network stack. **The layer is eliminated for
that traffic.**

But it isn't *erased from the system* — TCP loopback remains
available for compatibility, and cross-pod traffic still rides
TCP+mTLS to the sidecar. The honest framing is: the layer
became *optional per listener*. The substrate didn't lose a
capability; the deployment gained a choice.

## Compression — sidecar by default; app opt-in for cacheable hot paths

Compression in this architecture sits at the **sidecar**, not
the app. This is both the standard service-mesh deployment
pattern AND the perf-correct default. The wat handler speaks
plaintext to the sidecar over UDS; the sidecar handles
Content-Encoding negotiation and codec for the wire.

### Data flow

**Outbound (response → client):**
```
wat handler → wat-http-server (plaintext bytes; no Content-Encoding)
  → UDS → sidecar (compresses per client Accept-Encoding;
                   gzip / brotli / zstd)
  → TLS → wire
```

**Inbound (request → handler):**
```
wire → TLS → sidecar (decompresses if Content-Encoding present)
  → UDS → wat-http-server (plaintext bytes)
  → wat handler (sees decoded body always)
```

The wat handler **never sees compressed bytes in either
direction.** It speaks plaintext to the sidecar over UDS; the
sidecar handles negotiation and codec for the wire.

### Why the sidecar is the perf-correct default

1. **Envoy / istio-proxy is C++ with hand-tuned codecs**
   (gzip, brotli, zstd). The sidecar is purpose-built for
   this. Application logic stays in the app; wire concerns
   stay in the sidecar.
2. **One compression boundary, not two.** App emits plaintext;
   sidecar compresses once. No double-encode, no transcoding
   overhead.
3. **UDS bandwidth is plentiful.** Sending uncompressed bytes
   over UDS is fine — the kernel does in-process buffer copy.
   Compressing into UDS just to have the sidecar decompress
   and re-compress for the wire would be wasted CPU.
4. **Operations decoupling.** SRE can adjust compression
   algorithms (gzip → zstd → brotli) at the mesh level
   without app changes or redeploys.

### What clients see

Compression is a **free win** because the universe of HTTP
clients already speaks it. Browsers and standard HTTP
libraries (curl, reqwest, requests, fetch) decompress
Content-Encoding transparently. From the client's perspective
there's nothing to do — `Accept-Encoding: br, gzip, zstd`
goes out, body comes back decoded.

`wat-http-client` (arc 011) follows the same pattern: outbound
requests advertise `Accept-Encoding`; inbound responses are
transparently decompressed before the wat handler sees them.

### App-level compression — opt-in for hot cacheable endpoints

The default sidecar-based pattern wins for almost everything.
The legitimate exception is **cacheable dynamic responses at
very high RPS** where compressing once and serving the
compressed form repeatedly beats re-compressing on every hit.

Opt-in middleware accommodates this without forcing it:

```scheme
;; Default: handler emits plaintext; sidecar compresses
:my-handler

;; Opt-in: app-level compression with response caching
(:wat::http::server::compose
  (:wat::core::vec :Middleware
    (:middleware/compress-cached
      :algorithm :brotli
      :cache-key (:lambda ((req :Request))
                   (:Request/path req))))
  :my-handler)
```

The middleware sets `Content-Encoding: br` on the response;
the sidecar sees it already compressed and forwards as-is.
For top-1% RPS endpoints with stable payloads, this saves
real CPU. For everything else, sidecar handles it.

### Static assets — pre-compress at build time

Out of scope for wat-http-server, but worth flagging the
pattern: a sibling `wat-http-static` crate would pre-compute
`.gz`, `.br`, `.zst` variants at build time and serve cached
compressed bytes per `Accept-Encoding`. This is the
highest-perf path for any static content and is the standard
shape for asset servers.

### Security caveat — BREACH / CRIME

- **BREACH (2013)** affects HTTP body compression of responses
  containing secrets that depend on attacker-controlled input.
  Mitigations are application-layer: don't mix secrets +
  user input in compressed responses; use length padding;
  rate-limit. This applies to ANY HTTP framework with
  compression and isn't specific to wat-http-server. Worth
  documenting because deployment at scale will surface it.
- **CRIME (2012)** was TLS-level compression of secret-bearing
  traffic. **Irrelevant here** — we never compress at the TLS
  layer; sidecar compresses HTTP body only, after TLS
  termination.

### Per the four questions on compression placement

- **Obvious?** ✅ — compression is at the sidecar by default;
  app-level opt-in is explicit middleware
- **Simple?** ✅ — handler doesn't think about compression;
  sidecar does its job; one decision point per response
- **Honest?** ✅ — the layer where compression happens is
  visible in deployment config (sidecar) or middleware
  composition (app); no hidden codecs, no surprise headers
- **Good UX?** ✅ — handler authors write plaintext; SRE
  configures wire compression at the mesh; advanced perf
  patterns are opt-in middleware

Standard quad. Compression placement is a deployment-flexibility
decision, not a substrate-novelty one. The architecture
supports the right pattern at every level without forcing any.

## Connection to wat-network deployment

The wat-network deployment story (per WAT-NETWORK.md), with
transport choices made explicit:

```
        cross-pod traffic
            (TCP+mTLS)
                │
                ▼
┌───────────────────────────────────────────────────────┐
│ k8s pod                                               │
│                                                       │
│   ┌─────────────────────┐    ┌───────────────────┐    │
│   │ istio sidecar        │    │ wat-http-server   │    │
│   │ - mTLS termination   │UDS │   container       │   │
│   │ - SPIFFE identity    ├────┤ - UDS listener    │   │
│   │ - L4/L7 authz        │    │   (from sidecar)  │   │
│   └─────────────────────┘    │ - TCP localhost   │    │
│                              │   listener        │    │
│                              │   (probes/tools)  │    │
│           ┌──── kubelet ────►│                   │    │
│           │   probes (TCP)   │ - Signed payload  │    │
│           │                  │   verification    │    │
│           │                  │ - Handlers        │    │
│                              └───────────────────┘    │
└───────────────────────────────────────────────────────┘
```

**Layer composition:**
- Network layer: istio sidecar handles mTLS, SPIFFE identity,
  L4/L7 authz for cross-pod traffic
- Transport layer: UDS for trusted sidecar→app; TCP loopback
  for tooling/probes
- Application layer: wat handler does its OWN signed payload
  verification on the request body

Two layers of cryptographic verification compose; spoofing
requires breaking both. The layered approach is honest:
network-layer concerns stay in the sidecar; transport choice
stays in the listener config; application-layer security stays
in the handler.

This is **standard k8s + istio service deployment with wat as
the application** — no exotic infrastructure required.

## Open architectural questions

Three flagged for slice-time decisions; all guided by the
four questions:

A. **Body streaming vs full-buffer.** For small bodies,
   buffering is fine; for large bodies (file uploads;
   streaming JSON; SSE responses), streaming is needed. Should
   v1 support streaming bodies, or buffer-only for slice 1?
   Lean: buffer-only for slice 1; streaming as slice 2 or
   later when a real consumer needs it.

B. **WebSocket / SSE / HTTP/2 push.** wat-http-server is
   request/response. Streaming protocols (WebSocket; SSE;
   HTTP/2 push) are different shapes. Out of scope for v1?
   Lean: yes, out of scope; could be a sibling arc later.

C. **Health check / metrics endpoints.** Standard practice
   for k8s deployments. Does wat-http-server include a
   built-in `/healthz` or `/metrics` endpoint, or leave it
   to the user? Lean: leave to user; provide convenience
   middleware (log; cors; compress) but not opinionated
   endpoints.

## What's NOT in scope

- **Routing** — that's arc 010 (wat-http-router). This arc is
  just the handler interface + middleware + Rust shim.
- **HTTP client** — that's RemoteProgram (arc 007). This arc
  is server-side only.
- **TLS / mTLS termination** — istio sidecar (or a similar
  service mesh) handles it. The wat-http-server container
  receives plain HTTP locally (over UDS or TCP loopback) with
  verified identity headers.
- **WebSocket / SSE / streaming protocols** — request/response
  only for v1.
- **Authentication / authorization** — application-layer
  concerns; user implements via middleware. wat-http-server
  provides the middleware mechanism; doesn't ship auth.
