# wat-http-serve — SLICE-PLAN

Sketch only. Not sized for shipping. The bar to graduate this
arc into a real `wat-rs/docs/arc/...` arc is:

1. RemoteProgram (007) wire protocol decisions firm enough
   that wat-http-serve can either expose plain REST or the
   wat-wire format coherently
2. wat-fmt slice 1 has shipped (so the established
   crate-shipping pattern is real)
3. User signals "let's start"

When that happens, this slice plan gets re-sized against the
substrate-as-it-then-is. The slices below are the rough
shape; numbers are placeholders.

---

## Slice 1 — Echo server

**Goal:** end-to-end echo server. wat handler receives a
request; returns a response. Rust shim handles the network
boundary.

**Done when:**
- `wat-rs/crates/wat-http-serve/` exists with arc-013 layout
- `wat-http-serve::Handler` / `Request` / `Response` / `HandlerError`
  types defined in wat
- Rust shim using tokio + hyper accepts HTTP requests on a
  configurable port; converts to `wat-http-serve::Request`;
  invokes a wat handler via wat-vm; serializes response
- Echo handler in wat-tests; integration test using a real
  HTTP client (reqwest) against the real shim
- Buffer-only body for v1 (streaming deferred)
- Plain HTTP only (TLS termination is istio's job)

**Out of scope for this slice:**
- Middleware composition (slice 2)
- Multiple concurrent requests (slice 3)
- Q-channel wat-wire format (slice 4)

**Estimated:** 1-2 weeks of substrate work; user-driven.

---

## Slice 2 — Middleware composition

**Goal:** middleware as handler-wrappers compose cleanly.

**Done when:**
- `wat-http-serve::Middleware` type alias for handler → handler
- `wat-http-serve::compose` combinator folds middleware list
  around a leaf handler
- Three convenience middlewares ship in the crate:
  - `middleware::log` — request/response logging
  - `middleware::compress` — gzip/brotli response compression
  - `middleware::cors` — CORS headers
- wat-tests showing 3-deep middleware composition
- Each middleware is a wat function; nothing requires Rust

**Estimated:** Slice 1 + a week.

---

## Slice 3 — Concurrent request handling

**Goal:** the Rust shim handles many concurrent requests
correctly via the tokio runtime; the wat-vm dispatches each to
an independent handler invocation without state corruption.

**Done when:**
- `wat-http-serve::serve` spawns hyper service on tokio runtime
- Concurrent integration test (100+ simultaneous requests)
  passes with no state corruption
- Per-request timeout configurable
- Connection limit configurable
- Graceful shutdown signal (SIGTERM) drains in-flight
  requests before exit

**Estimated:** Slice 2 + 1-2 weeks; depends on wat-vm
re-entrancy story being firm.

---

## Slice 4 — Q-channel wat-wire format

**Goal:** wat-http-serve apps can serve BOTH plain REST AND
the wat-wire format on the same listener; content-type
negotiation determines which.

**Done when:**
- `application/edn+wat` content-type recognized
- Request bodies in wat-wire format are deserialized via
  wat-edn into typed wat values
- Response bodies serialize back via wat-edn; Q-channel
  Ok/Err discriminator set per response
- Cross-references RemoteProgram (007) wire protocol
- wat-tests demonstrating same handler serving both
  formats

**Depends on:** RemoteProgram (007) shipping its wire
protocol primitives.

**Estimated:** Slice 3 + 2 weeks; gated on arc 007 progress.

---

## Slice 5 — Production deployment validation

**Goal:** wat-http-serve runs in a real k8s pod behind istio
for at least one concrete application.

**Done when:**
- Dockerfile + helm chart shipped in `wat-rs/deploy/wat-http-serve/`
  (or a sibling repo if appropriate)
- Pod runs istio sidecar in front; wat-http-serve container
  receives plain HTTP locally; signed payload validation in
  application layer
- One concrete deployed application; smoke tests pass
- Resource consumption documented (CPU / memory / latency
  baseline)
- Documentation: deployment recipe in `wat-rs/docs/`

**Depends on:** an actual application that wants this
deployment shape.

**Estimated:** Slice 4 + open-ended (driven by application
need).

---

## Slices NOT planned

- WebSocket / SSE / HTTP/2 push — different shape; sibling arc
- Body streaming — added when a concrete consumer needs it
- Built-in `/healthz` / `/metrics` — left to user; provide
  example middleware in docs
- TLS termination in-process — istio's job

---

## Honest accounting

This slice plan is **sketched, not sized**. The substrate work
required for slice 1 alone may surface unknowns that reshape
slice 2-5 entirely. The pattern from prior arcs holds: the
slice plan is a shape; reality teaches us what we missed; the
arc gets restructured as it ships.

The four-questions discipline applies to each slice
independently. Each slice should answer all four with
honest checkmarks before declaring the slice done.
