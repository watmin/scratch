# wat-http-api-server — SLICE-PLAN

Sketch only. Not sized for shipping. Bar to graduate:

1. arc 014 (wat-http-api-spec) has shipped slice 1
2. arc 010 (wat-http-router) has shipped slice 1
3. arc 013 (wat-schema) has shipped slice 1
4. User signals "let's start"

---

## Slice 1 — Generate routes + input validation

**Goal:** spec → server with routes registered + input
validation working.

**Done when:**
- `wat-rs/crates/wat-http-api-server/` exists with arc-013 layout
- `(:wat::http::api::server::generate :name :spec :SpecName)`
  macro works
- Iterates spec operations; registers routes on wat-http-router
- Input validation: body parsed via wat-edn, validated via
  wat-schema using the spec's input shape
- Path-param validation: typed extraction
- Query-param validation: typed extraction
- Handler signature derivation: matches spec input + output
- `define-handler` form binds user handler body to the spec
  operation; compile error if signature mismatch
- Compile error if any spec operation lacks a handler
- wat-tests covering: simple GET; POST with body; path params;
  query params; missing handler caught at compile time

**Out of scope:**
- Auth middleware (slice 2)
- Output validation (slice 3)
- Rate-limit middleware (slice 4)
- Cache headers (slice 5)
- Idempotency keys (slice 6)

---

## Slice 2 — Auth middleware

**Goal:** spec :auth declarations generate the appropriate
middleware.

**Done when:**
- `:none`, `:bearer-token`, `:basic-auth`, `:api-key`, `:mtls`,
  `:custom` auth variants all supported
- Auth context populated and made available to handler bodies
- 401/403 returned per failure mode
- `:scopes` enforcement for `:bearer-token`
- mTLS variant integrates with sidecar-forwarded cert headers
- wat-tests cover each auth variant

---

## Slice 3 — Output validation + error envelope

**Goal:** outgoing responses validated against spec output;
errors land in consistent envelope.

**Done when:**
- Output shape validated against spec's `:output` declaration
- Strict mode (default ON in dev): mismatch logs + 500
- Loose mode (toggle OFF in prod): mismatch logs but doesn't
  reject
- Error envelope generated per spec error variants
- Error variants serialize to the right HTTP status code
- `:request/id` and `:timestamp` wrapper fields
- Optional `:rfc7807` (Problem Details) format alternative
- wat-tests: every error variant from a spec → correct status +
  envelope

---

## Slice 4 — Rate-limit middleware

**Goal:** spec :rate-limit declarations generate working rate
limiting.

**Done when:**
- `:by` keyword variants: `:remote-addr`, `:auth-subject`,
  custom callable
- `:max <rate>` parsed (per-second / per-minute / per-hour)
- Sliding-window counter (in-process; pluggable for distributed
  later)
- 429 + `Retry-After` response on exceed
- `:tiered` rate-limit variants
- wat-tests: simple limiter; per-subject limiter; tiered

---

## Slice 5 — Cache headers + ETag emission

**Goal:** spec :cacheable? declarations emit standard cache
headers.

**Done when:**
- `Cache-Control` header per `:max-age` declaration
- `Vary` header per `:vary` keys
- ETag emission for responses with ID-shaped fields
  (heuristic; overridable)
- Conditional GET (`If-None-Match` / `If-Modified-Since`) → 304
- wat-tests cover each cache pattern

---

## Slice 6 — Idempotency-key handling

**Goal:** non-idempotent operations honor `Idempotency-Key`
headers.

**Done when:**
- Operations with `:idempotent? false` accept
  `Idempotency-Key` header
- Replay cache (in-process; pluggable to Redis later)
- Same key + same body → cached response
- Same key + different body → 422 conflict
- TTL on cache entries (default 24h; configurable)
- wat-tests: simple replay; conflict; expiration

---

## Slice 7 — Production hardening

**Goal:** wat-http-api-server is genuinely usable for production
APIs.

**Done when:**
- Performance: route lookup + validation < 1ms for typical specs
  with 100 ops; auth + rate-limit add < 500µs
- Memory: bounded validation + bounded cache
- Metrics emitted (per operation: requests/sec, latency p50/p95/p99,
  error rate by variant, rate-limit triggers, cache hits)
- Documentation: complete reference; deployment cookbook;
  middleware composition reference; common patterns
- One concrete deployed application using a real wat-http-api-spec

---

## Slices NOT planned

- **WebSocket / SSE / streaming responses** — different shape
- **Server-side rendering / templating** — application concern
- **Static asset serving** — sibling crate
- **Custom mid-stack middleware injection** — opinionated by
  design; use wat-http-router directly if you need it

---

## Honest accounting

Sketched, not sized. The biggest unknown: how does the
generation macro handle large specs (100+ operations)? Lean:
build-time generation; runtime cost is just route lookup
(O(log N) via arc 010's matcher). Should scale fine.

The biggest design question: where do the generated handlers
live? In the user's crate (compile-time generated stubs the
user fills in)? Or in the api-server crate (lookup table at
runtime)? Lean: compile-time stubs in user crate — better type
safety; clearer error messages; matches Rust ecosystem
conventions (axum's `Router`-builder pattern uses this approach).

The four-questions discipline applies to each slice independently.
Each slice should answer all four with honest checkmarks before
declaring the slice done.
