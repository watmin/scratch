# wat-http-api-client — SLICE-PLAN

Sketch only. Not sized for shipping. Bar to graduate:

1. arc 014 (wat-http-api-spec) has shipped slice 1
2. arc 011 (wat-http-client) has shipped slice 1
3. arc 013 (wat-schema) has shipped slice 1
4. User signals "let's start"

---

## Slice 1 — Generate typed call functions

**Goal:** spec → client SDK with typed call functions for every
operation.

**Done when:**
- `wat-rs/crates/wat-http-api-client/` exists with arc-013 layout
- `(:wat::http::api::client::generate :name :spec :SpecName ...)`
  macro works
- Iterates spec operations; generates one call function per
  operation
- Function signature matches spec input + output + errors
- Path interpolation works (path params plug into URL)
- Query param encoding works
- Body serialization via wat-edn (EDN format default)
- Response deserialization + status-code-based variant decoding
- Output validation against spec output shape
- Error variant decoding into typed `:Result<...>`
- wat-tests covering: simple GET; POST with body; path params;
  query params; happy path + error variants

**Out of scope:**
- Auth header injection (slice 2)
- Strict output validation toggle (slice 3)
- Per-call config overrides (slice 4)
- mTLS auth variant (slice 5)

---

## Slice 2 — Auth header injection

**Goal:** auth declarations from spec generate the appropriate
header injection.

**Done when:**
- Static `:bearer-token` auth works
- `:bearer-token-from-env` reads at call time (allows token
  rotation via env var)
- `:bearer-token-from-fn` calls a refresher fn at call time
- `:basic-auth` (username + password) works
- `:api-key` with custom header name works
- 401 / 403 responses surface as typed errors per spec
- wat-tests cover each auth variant

---

## Slice 3 — Output validation + content-type negotiation

**Goal:** outputs validated against spec; both EDN and JSON
content-types work.

**Done when:**
- Strict mode (default ON in dev): output schema mismatch
  surfaces as typed error
- Loose mode (toggle): mismatch logs warning but returns
  whatever shape comes back
- Content-Type negotiation: `application/edn+wat` for wat
  peers; `application/json` for general clients (transparent
  via wat-edn)
- Content-Type override per call (escape hatch)
- wat-tests: matching schemas; mismatched schemas; both formats

---

## Slice 4 — Per-call config + retries deferred

**Goal:** call-time overrides for timeout, headers; explicit
retry-deferral pattern documented.

**Done when:**
- `:timeout` kwarg per call overrides client default
- `:headers` kwarg per call adds extra headers (e.g.,
  `X-Request-ID`)
- `:auth-override` kwarg per call swaps credentials (rare;
  multi-tenant scenarios)
- Documentation: explicit "no automatic retries" note;
  example retry loop pattern; pointer to potential
  `wat-http-retry` sibling crate

---

## Slice 5 — mTLS + wat-network integration

**Goal:** clients calling wat-network peers via mTLS work
seamlessly.

**Done when:**
- `:auth (:mtls :cert-pem ... :key-pem ...)` works
- Cert + key loaded at client creation; passed to
  wat-http-client's TLS config
- SNI + ALPN negotiation per spec hints
- Round-trip integration test against arc 015 server
  configured for mTLS
- Documentation: deployment recipe for wat-network
  client-side mTLS

---

## Slice 6 — Production hardening

**Goal:** wat-http-api-client is genuinely usable for
production SDKs.

**Done when:**
- Performance: call function overhead < 100µs (excluding
  network); spec lookup happens at compile time, not call time
- Memory: bounded validation; no leak on long-lived clients
- Metrics emitted (per call: latency p50/p95/p99, error rate
  by variant, retries-attempted-by-caller)
- Documentation: complete reference; calling cookbook;
  testing patterns (mock server in-process; contract tests);
  comparison to other generated-SDK approaches
- One concrete deployed application using a real
  wat-http-api-spec via this client

---

## Slices NOT planned

- **Streaming response support** — different shape; sibling arc
- **Mock server from spec** — sibling crate
  (`wat-http-api-mock`?)
- **Contract testing** — sibling crate
  (`wat-http-api-contract-test`?)
- **Retry combinators** — caller's responsibility; sibling crate
  later if patterns settle
- **OAuth flows** — application-layer; user implements via
  call functions + their own state machine
- **Cookie store / session management** — application-layer

---

## Honest accounting

Sketched, not sized. Biggest unknown: how does the generated
function dispatch handle large specs (100+ operations)? Lean:
each call function is independent; lookup happens at compile
time; runtime cost is just the function call. Should scale fine.

The biggest design question: when the call function fails to
deserialize a response, what's the user-facing error? Lean:
typed `:Err :ProtocolError` with the raw bytes attached so
callers can debug. Don't pretend the response was valid; don't
panic; surface the structural failure.

The four-questions discipline applies to each slice independently.
Each slice should answer all four with honest checkmarks before
declaring the slice done.
