# wat-http-client — SLICE-PLAN

Sketch only. Not sized for shipping. The bar to graduate this
arc into a real `wat-rs/docs/arc/...` arc is:

1. arc 009 (wat-http-serve) has shipped slice 1 (so the shared
   Request/Response types are firm)
2. arc 007 (RemoteProgram) wire protocol decisions firm enough
   to know what wat-http-client must expose for it
3. User signals "let's start"

When that happens, this slice plan gets re-sized.

---

## Slice 1 — Basic GET / POST round-trip

**Goal:** wat program makes an HTTP call to a real server and
gets a typed response back.

**Done when:**
- `wat-rs/crates/wat-http-client/` exists with arc-013 layout
- `wat-http-client::call`, `wat-http-client::get`,
  `wat-http-client::post` defined in wat
- Rust shim wraps reqwest::Client; dispatched via wat-vm
- TLS via rustls; system root certs; cert verification
  enabled by default
- Transparent gzip/br/zstd decompression for inbound bodies
- HTTP/1.1 + HTTP/2 (ALPN negotiated)
- Default 30s timeout; `:timeout` kwarg overrides
- Round-trip integration test: wat-http-client calls a real
  wat-http-serve from arc 009; both parsing and serializing
  the shared Request/Response types
- ClientError taxonomy populated; tests cover ConnectError,
  Timeout, InvalidUrl

**Out of scope for this slice:**
- UDS transport (slice 2)
- Connection pool tuning (slice 3)
- Outbound body compression (slice 4)
- Streaming response bodies (slice 5+)

---

## Slice 2 — UDS transport + connection pooling

**Goal:** client can dial UDS upstream; pool tuning works.

**Done when:**
- `uds:///path/to.sock/...` URL scheme dispatches over
  UnixStream via custom reqwest connector
- `wat-http-client::configure :pool-max-idle-per-host ...`
  affects pool behavior
- Keepalive idle timeout configurable
- Integration test: arc 009 over UDS + arc 011 over UDS;
  full round-trip on a single host using only socket files
- Documentation: when to use UDS (sidecar topology;
  filesystem-permissioned local IPC)

---

## Slice 3 — TLS client config + mTLS

**Goal:** client can present a client certificate; verify
server certs against custom roots.

**Done when:**
- `:client-cert` kwarg accepts PEM cert + key
- `:root-certs :custom + :custom-roots` works
- `:sni-hostname-override` works for cases where the URL
  hostname differs from the SNI required (e.g., calling
  through a load balancer)
- mTLS round-trip integration test against arc 009 deployed
  with a configured CA
- Documentation: when to use app-level mTLS vs sidecar mTLS
  (default: sidecar; rare cases: app-level)

---

## Slice 4 — Outbound body compression + redirects

**Goal:** client can compress request bodies; redirect
following is configurable.

**Done when:**
- `:body-encoding :gzip` (and `:br`, `:zstd`) compresses
  outbound body and sets Content-Encoding header
- `:follow-redirects? false` disables redirect following
- `:max-redirects N` caps the redirect chain
- `RedirectLimit` / `RedirectInvalid` errors surface
- Tests cover each combination

---

## Slice 5 — Streaming bodies + production hardening

**Goal:** large request/response bodies don't blow memory;
production deployment is documented.

**Done when:**
- `:body :stream` accepts an async stream of bytes for
  upload
- `Response.body :stream` exposes a stream for download
- Memory usage stays bounded for 1GB+ bodies
- Connection metrics exposed (active connections; pool
  utilization; latency p50/p95/p99 per host)
- Documentation: deployment recipe for sidecar-mediated
  outbound traffic

---

## Slices NOT planned

- **Cookie jar** — opt-in via separate kwarg; not its own
  slice
- **OAuth / auth flows** — sibling crate (`wat-http-oauth`?)
  if patterns settle
- **WebSocket client** — different shape; sibling arc
  (`wat-ws-client`?) if needed
- **HTTP/3 (QUIC)** — opt-in reqwest feature; revisit when a
  real consumer asks
- **Retry combinators** — caller's responsibility; sibling
  crate (`wat-http-retry`?) if combinators emerge as common
  patterns

---

## Honest accounting

This slice plan is **sketched, not sized**. The biggest
unknown is: how does wat-http-client's `Client` instance live
in a wat-vm? One global per vm? One per call (creating
expensive)? One per "logical app" with reqwest's pool
amortizing across calls?

Lean: one global reqwest::Client per wat-vm; configuration
applied once at startup; connection pool amortizes naturally.
But this needs to fit how arc 007 (RemoteProgram) wants to
build on top — if RemoteProgram needs per-peer client config,
the model needs to support that.

The four-questions discipline applies to each slice
independently. Each slice should answer all four with
honest checkmarks before declaring the slice done.