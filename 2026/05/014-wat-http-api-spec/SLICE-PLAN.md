# wat-http-api-spec — SLICE-PLAN

Sketch only. Not sized for shipping. Bar to graduate:

1. arc 013 (wat-schema) has shipped slice 1
2. arc 010 (wat-http-router) has shipped slice 1
3. User signals "let's start"

---

## Slice 1 — Spec declaration DSL

**Goal:** users can declare a spec with operations, shapes, and
errors; spec compiles to a typed value.

**Done when:**
- `wat-rs/crates/wat-http-api-spec/` exists with arc-013 layout
- `(:wat::http::api::spec::define :Name ...)` macro works
- Operations support: method, path, input, output, errors,
  description
- Shapes delegate to wat-schema; type aliases work
- Error variants typed with status code + body shape
- Compilation checks: reachability, uniqueness of operation
  names, valid HTTP methods, valid path patterns
- Compiled `Spec` value exported from the spec crate
- wat-tests covering: simple spec with a few ops; reachability
  errors caught at compile time

**Out of scope:**
- Auth declarations (slice 2)
- Rate-limit declarations (slice 2)
- Cacheability declarations (slice 2)
- Decorations beyond basics (slice 2)

---

## Slice 2 — Decorations (auth, rate-limit, idempotency, caching)

**Goal:** spec declares non-functional contracts; consumers can
read them.

**Done when:**
- `:auth (:bearer-token :scopes [...])` and other auth variants
  registered
- `:rate-limit (:by ... :max ...)` declarations work
- `:idempotent? bool` declarations work; default per HTTP method
- `:cacheable? (:max-age N :vary [...])` declarations work
- Tags / examples / description / metadata
- `:Spec/operations` accessor returns operations with all
  decorations attached
- wat-tests covering: each decoration; defaults applied correctly

---

## Slice 3 — Type bridging surface

**Goal:** server and client crates can read the spec uniformly.

**Done when:**
- `:Spec/operations spec` returns iterable of operations
- `:Operation/method op`, `:Operation/path op`, `:Operation/input op`,
  `:Operation/output op`, `:Operation/errors op`, `:Operation/auth
  op`, etc. all defined
- `:Spec/error-variants spec` returns the typed error union
- `:Spec/shape spec :ShapeName` looks up a shape by name
- All accessors return typed values; consumers don't reach into
  the spec's internal representation
- Documentation: spec consumer reference

---

## Slice 4 — Path pattern + parameter handling

**Goal:** path params are typed; path interpolation works for
client; path matching works for server (via arc 010).

**Done when:**
- `:path-params (:shape (:id :uuid) (:slug :string))`
  declarations work
- Spec-validation rejects path patterns with undeclared params
- Path patterns match arc 010's pattern syntax (compatibility)
- Type-bridging surfaces path-param shape per operation
- wat-tests: simple paths; nested paths; wildcards; missing
  declarations

---

## Slice 5 — Spec composition (multi-module)

**Goal:** large APIs split across multiple files.

**Done when:**
- `(:include :other-module-spec)` works
- Type aliases reusable across modules
- Compilation handles cycles (or rejects them with a clear error)
- One concrete multi-module spec demonstrates composition
- Documentation: organization patterns for big APIs

---

## Slice 6 — Production hardening

**Goal:** spec compilation is solid; consumers can rely on it.

**Done when:**
- Performance: spec compilation < 100ms for 100-operation specs
- Error messages: clear; pinpoint the source location of the
  problem
- Versioning: spec carries semver; consumers can check compat
- Documentation: complete reference; cookbook; migration patterns

---

## Slices NOT planned

- **Doc generator** — sibling crate (`wat-http-api-doc`?) later
- **Mock server generator** — sibling crate (`wat-http-api-mock`?)
- **Contract test generator** — sibling crate
  (`wat-http-api-contract-test`?)
- **Cross-language SDK generator** — sibling crate (Smithy-style;
  future need-driven)
- **Streaming responses (SSE; WebSocket; HTTP/2 push)** — different
  shape; sibling arc if needed

---

## Honest accounting

Sketched, not sized. Biggest unknown: how does the spec value
get exported and re-imported across crates? Likely a Rust
`pub static` or `pub fn spec() -> Spec` pattern. Need to confirm
this matches how wat-rs handles cross-crate value sharing.

The compilation-time guarantees (drift impossibility) require
that BOTH consumer crates compile against the same `Spec` value,
not separate parses of the spec source. This is a Rust-ecosystem
property; standard cargo build does it for free.
