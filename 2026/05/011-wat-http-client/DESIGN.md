# wat-http-client — DESIGN

The other end of arc 009. A wat-side HTTP client built on
reqwest (which is built on hyper). Same Request/Response types;
same plaintext-handler discipline; same UDS-or-TCP transport
flexibility.

---

## The four questions are the design compass

- **Obvious?** A wat program calling out to an HTTP endpoint
  reads as a function call with kwargs.
- **Simple?** One signature; the same Request/Response types
  shared with arc 009 where possible.
- **Honest?** What's named matches what's there; failures land
  as typed errors at the call site; no hidden retries; no
  silent decompression failures.
- **Good UX?** Calling an HTTP endpoint should be the most
  readable line in the application; complexity hides in the
  reqwest layer where it belongs.

## The Rust ecosystem foundation

```rust
// reqwest gives us, free:
// - Connection pooling (per-host limits; idle reuse)
// - DNS resolution
// - TLS via rustls or native-tls (we'll use rustls)
// - Transparent gzip / brotli / zstd decompression
// - Redirect handling (configurable)
// - HTTP/1.1 + HTTP/2 (HTTP/3 via opt-in feature)
// - HTTP_PROXY / HTTPS_PROXY env var support
// - Timeouts (connect; read; write; total)
// - Cookie store (opt-in)
// - UDS upstream via custom connector (hyperlocal pattern)
```

We don't reinvent any of this. wat-http-client wraps reqwest
in a wat-side interface and dispatches via the wat-vm.

## The wat-side interface

### Client invocation — one function

```scheme
(:wat::http::client::call
  :method :get                              ; required
  :url "https://api.example.com/users/42"   ; required
  :headers (:wat::core::HashMap :String     ; optional
             "accept" "application/json")
  :body (:wat::http::serve::Body/empty)     ; optional; default empty
  :timeout (:wat::time::Duration/seconds 30) ; optional; default 30s
  :follow-redirects? true                   ; optional; default true
  :max-redirects 10)                        ; optional; default 10
;; => :Result<:wat::http::serve::Response, :ClientError>
```

This is the **only** public function. Everything else is types,
configuration, and convenience constructors. The kwarg-heavy
shape leans on arc 008 (wat-kwargs); auto-generated kwarg
variants from the function signature.

### Convenience constructors

```scheme
(:wat::http::client::get  :url "...")
(:wat::http::client::post :url "..." :body json-data)
(:wat::http::client::put  :url "..." :body json-data)
(:wat::http::client::delete :url "...")
;; etc.
```

These all expand to `(:wat::http::client::call :method :X ...)`.
Sugar.

### Shared types with arc 009

```scheme
;; SHARED with arc 009 — same struct, both crates use it
:wat::http::serve::Request
:wat::http::serve::Response
:wat::http::serve::Body
:wat::http::serve::Method
```

The types live in arc 009's `wat/http/serve/` module; arc 011
imports them. This is the right shape because:

- The Request a client builds is structurally the Request a
  server receives (modulo `remote_addr` etc.)
- The Response a client receives is structurally the Response
  a server sent
- One type per concept across both ends keeps the substrate
  honest

If asymmetry surfaces (e.g., client needs request fields a
server's request can't represent), we'll split via type aliases
or struct embedding rather than duplicate.

### Client error model

```scheme
(:wat::core::enum :wat::http::client::ClientError
  ;; Network / transport
  ((ConnectError    (cause :HolonAST)))     ; connect failed
  ((TlsError        (cause :HolonAST)))     ; TLS handshake failed
  ((Timeout         (kind :TimeoutKind)))   ; connect / read / write / total
  ((ResolutionError (host :String)))        ; DNS failed

  ;; Protocol
  ((InvalidUrl      (url :String)
                    (reason :String)))
  ((ProtocolError   (cause :HolonAST)))     ; malformed HTTP

  ;; Redirect handling
  ((RedirectLimit   (count :i64)))          ; too many redirects
  ((RedirectInvalid (url :String)))         ; bad redirect URL

  ;; Body / decompression
  ((BodyError       (cause :HolonAST)))     ; read failed mid-body
  ((DecodeError     (encoding :String)
                    (cause :HolonAST))))    ; gzip/br/zstd decode failed
```

The HTTP **status code is part of the Response**, not an
error. A 404 or 500 returns `Ok(response)` with `status: 404`
or `status: 500` — these are valid responses, not network
errors. Callers decide whether a 5xx is "an error" for their
application; that's not a transport-level concern.

This matches the failure-engineering discipline: errors are
typed; the type taxonomy makes the failure class structural;
caller decides what counts as recoverable.

## Connection pooling

reqwest manages a connection pool per `Client` instance. The
shim exposes one logical client per wat-vm (default) with
configurable limits:

```scheme
(:wat::http::client::configure
  :pool-max-idle-per-host 16
  :pool-idle-timeout (:Duration/seconds 90)
  :keepalive (:Duration/seconds 60))
```

For most apps this is set once at startup and never touched.

## TLS client configuration

```scheme
(:wat::http::client::configure-tls
  :verify-certs? true                  ; default true; never set false in prod
  :root-certs :system                  ; :system | :webpki | :custom
  :custom-roots [pem-string-1 pem-string-2 ...]
  :client-cert (:wat::http::client::Identity/pem cert-pem key-pem)
  :sni-hostname-override "internal.example.com"  ; for SNI override
  :alpn-protocols ["h2" "http/1.1"])
```

Defaults match what a production TLS client should do: verify
certs against the system trust store; offer ALPN for HTTP/2
negotiation. Power-user knobs available; rarely needed.

For wat-network mTLS (signed evals to peer wat-vms), the
client cert is loaded from the SPIFFE identity managed by
the istio sidecar. Most deployments delegate cert management
to the sidecar entirely and the wat-http-client does plain
HTTP to localhost; the sidecar wraps the call in mTLS.

## Transport — TCP and UDS

Symmetric to arc 009. The same client can dispatch over either:

```scheme
;; TCP (the default; URL host:port resolves via DNS)
(:wat::http::client::call
  :url "https://api.example.com/foo")

;; UDS (talking to a local service through a sidecar)
(:wat::http::client::call
  :url "uds:///var/run/sidecar-egress.sock/api.example.com/foo")
```

The `uds://` URL scheme tells the client to dial the given UDS
path; the rest of the URL becomes the HTTP `Host` and path.
This is a convention, not an HTTP standard — we adopt it
because it's the cleanest way to pass UDS path + HTTP
target through one URL.

For service-mesh deployments, the typical pattern is:

- App calls `https://api.example.com/...` as if it were going
  to the real internet
- iptables in the pod redirects outbound traffic to the local
  sidecar
- Sidecar handles mTLS, SPIFFE identity, retries, circuit
  breakers
- App is unaware

OR, more explicit:

- App calls `uds:///var/run/sidecar-egress.sock/...`
- Sidecar receives over UDS; same handling as iptables case
- Slightly faster (no iptables hop); explicit topology

Both patterns work; the shim supports both URL schemes.

## Compression — transparent decompression by default

reqwest handles `Content-Encoding: gzip / br / zstd` on
inbound responses automatically. The wat handler always sees
**decoded plaintext bytes** in `Response.body`. This matches
arc 009's plaintext-handler discipline.

For outbound request bodies:
- **Default: no compression.** Most APIs don't expect
  compressed request bodies.
- **Opt-in:** `:body-encoding :gzip` (or `:br` / `:zstd`)
  causes the client to compress the body and set
  `Content-Encoding: gzip`. Caller is responsible for
  knowing the destination accepts compressed bodies.

```scheme
(:wat::http::client::post
  :url "https://upload.example.com/data"
  :body large-payload
  :body-encoding :zstd)
```

## Retries — deliberately NOT included

wat-http-client **does not retry**. A failed call returns
`Err(ClientError::...)`. The caller decides:

- "Retry once on Timeout" — caller writes that loop
- "Exponential backoff with jitter" — caller writes that
  policy
- "Fail fast and surface the error" — that's the default

This is a **failure-engineering** position. Retry policies
are application concerns, not transport concerns. Different
endpoints have different retry semantics:

- A GET against an idempotent endpoint can be retried freely
- A POST that creates resources should NOT be retried without
  idempotency keys
- A payment API call has different retry semantics than a
  user lookup

Hardcoding retry into the transport layer makes the wrong
choice for some callers. Hardcoding **no** retry into the
transport layer forces every caller to make the right choice
for their endpoint. The four questions land on this
unambiguously: ✅✅ Honest because the failure model is
visible at every call site.

A future helper crate (`wat-http-retry`?) could provide
common retry combinators (exponential backoff; circuit
breaker) as opt-in middleware over wat-http-client. Sibling
arc material; not in scope for this crate.

## Timeouts

Every call has a total timeout. Default 30s. Configurable
per-call via `:timeout`. No special "infinite timeout"
mode — callers who genuinely want long-running connections
should think about why and pass an explicit large value.

reqwest also exposes connect / read / write timeouts
separately; we expose them via `:connect-timeout`,
`:read-timeout`, `:write-timeout` for power users. Most
callers should just set `:timeout`.

## Per the four questions

- **Obvious?** ✅ — `(:wat::http::client::get :url "...")` is
  the same shape as a Ruby `Net::HTTP.get` or Python
  `requests.get`; familiar pattern from every language
- **Simple?** ✅ — one function (`call`); kwargs for options;
  shared types with arc 009; no hidden state between calls
- **Honest?** ✅✅ — errors are typed and structural; status
  codes are part of the Response, not an error; no hidden
  retries; no silent decompression fallthrough; the failure
  model is visible at every call site
- **Good UX?** ✅ — kwarg-heavy ergonomics via arc 008;
  convenience constructors for common verbs; defaults match
  what production TLS clients should do

Strong shape. Honest is ✅✅ because the typed error taxonomy
+ no-hidden-retries is the right shape, but not triple
because the underlying reqwest still has implicit behaviors
(redirect following; pool reuse) that we expose as
configuration but inherit as defaults.

## Cross-references

- **arc 009 (wat-http-serve)** — shared Request/Response
  types live there; client imports them
- **arc 007 (RemoteProgram)** — could sit ON TOP of
  wat-http-client for the typed-wat-to-wat case (RemoteProgram
  serializes a typed call into an HTTP request; wat-http-client
  delivers it; result deserializes back). Or RemoteProgram
  could have its own transport. To be decided when arc 007
  firms up.
- **arc 010 (wat-http-route)** — server-side routing; clients
  don't route. No direct dependency.
- **WAT-NETWORK.md** — wat-http-client is a building block of
  the wat-network. Combined with the local sidecar's mTLS +
  SPIFFE identity, plain wat-http-client calls become
  cryptographically authenticated wat-network peer-to-peer
  calls without the application code knowing.

## Open architectural questions

A. **HTTP/2 default vs HTTP/1.1 default.** reqwest negotiates
   via ALPN automatically. For wat-http-client, lean toward
   "negotiate" (offer both) and let the server choose; HTTP/2
   wins where the server supports it.

B. **HTTP/3 (QUIC) support.** reqwest supports HTTP/3 via
   opt-in feature. Out of scope for v1? Lean: yes; revisit
   when the need is real.

C. **Cookie jar.** reqwest has an opt-in cookie store. For
   wat-http-client, lean toward "no cookie jar by default;
   opt-in via `:cookie-store true`" — most server-to-server
   API calls don't need cookies, and accidentally sharing
   cookies across calls is a bug source.

D. **WebSocket client.** Different shape; out of scope.
   Sibling arc if needed (`wat-ws-client`).

E. **Streaming response bodies.** For large downloads (file
   transfers; SSE; long-poll), buffering the entire body is
   wrong. Slice 2 territory — start with buffer-only;
   streaming when a real consumer needs it.

## What's NOT in scope

- **Server-side HTTP** — that's arc 009.
- **Routing** — clients don't route; that's arc 010.
- **WebSocket / SSE / streaming protocols** — different shape;
  sibling arc if needed.
- **Retry policies** — caller's responsibility; sibling crate
  if combinators emerge as common patterns.
- **OAuth / auth flows** — application-layer; user implements
  via wat-http-client calls + their own state machine. Could
  ship `wat-http-oauth` later if patterns settle.