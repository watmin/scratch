# DEPENDENCY-DOCTRINE — what we choose to stand on

User direction (2026-05-03):

> *"we need a doc here aruging this and why we are ready to
> strongly couple ourserlves to tokie, hyper, etc... we already
> did this for serde (json) in wat-edn and wat-rs proper depends
> on crossbeam*
>
> *we are cognizant of our deps and choose them carefully"*

The recognition that triggered this doc:

> *"i'm curious..... given wat's shape... its.. 'we're not async
> but we're async'... we can just coexist with rust's native async
> reactor and not give a shit... anything that we do with our zero
> mutex directive just continues to work in the async world... so
> we can get all of rust's performance gains with our rigidity in
> the wat-vm?... seems like a good win?..."*

Yes. And it's worth arguing why.

---

## The thesis

wat is built on a stack of **carefully chosen Rust ecosystem
foundations**. This is deliberate. The substrate is committed
to deep coupling with mature, battle-tested, widely-used Rust
crates — and that coupling is itself a discipline.

We don't reinvent. We don't pretend independence. We **stand
on giants** and document which giants, why we picked them, and
what we'd do if any of them faltered.

This doctrine is not "use whatever crates" — it's the
opposite. We are **highly selective** about which dependencies
we accept, because each accepted dependency becomes part of
the substrate's surface area. A bad dep choice ships through
to every wat-vm forever. A good dep choice gives us decades of
ecosystem hardening at zero authoring cost.

## Why we couple deeply

### 1. Reinventing wheels is dishonest

If hyper exists, is well-maintained, has years of production
hardening at Cloudflare / Discord / AWS / 1Password / every
Rust HTTP shop, and solves the problem cleanly — writing our
own HTTP/2 implementation is the **opposite of failure
engineering**. It ships a worse system to feel independent.

The four questions:
- **Obvious?** ❌ — Why isn't it just hyper?
- **Simple?** ❌ — We're maintaining HTTP forever now
- **Honest?** ❌ — We're claiming "ours" what's actually a worse port
- **Good UX?** ❌ — Users learn our API instead of the universal one

Picking the canonical dep flips every checkmark.

### 2. Mature Rust async crates are battle-tested at scale

The crates we couple to are not academic projects. They are
operating in production at billion-request-per-day scale:

- **tokio** — every async Rust shop; Discord; Cloudflare's
  pingora; tower-rs; AWS SDK
- **hyper** — every Rust HTTP backend; reqwest; warp; axum;
  cloudflare workers
- **reqwest** — most-downloaded Rust HTTP client; used by
  thousands of crates
- **rustls** — replacing OpenSSL across the Rust ecosystem;
  AWS; Cloudflare; tokio-postgres
- **crossbeam** — concurrent data structures used by tokio
  itself, rayon, every high-perf concurrent crate
- **serde** — universal Rust serialization; used by basically
  every Rust crate that touches data

Their bugs become our bugs (acceptable trade — they fix faster
than we could). Their improvements become our improvements
(free wins). Their hardening transfers.

### 3. Coupling ≠ lock-in when the interface is honest

We depend on hyper's `Service` trait, not on hyper's internal
types. If hyper went away tomorrow, we'd swap to another HTTP
impl that satisfies the trait. The substrate's wat-side
interface (`Handler: Request -> Result<Response, Error>`) is
invariant; the Rust shim is replaceable.

This is the **abstraction is at the right layer** test: we
expose to wat what wat needs (typed handler signatures); we
hide from wat what's a Rust implementation detail (which HTTP
impl underlies the shim). Both layers are honest about what
they own.

The four-questions test for healthy coupling:

| Question | Answer when coupling is honest |
|---|---|
| Could we swap this dep with similar shape? | Yes (in finite engineering hours) |
| Does the dep leak its types into our API? | No (the wat-side surface is dep-agnostic) |
| Could the dep change behavior without us noticing? | Caught by tests; pinned by Cargo.lock |
| Is the dep's maintenance trajectory sustainable? | Active, used by canonical projects |

### 4. Standing on giants is the four-questions-honest move

Building wat-network on **istio + tokio + hyper** is:
- **Obvious** ✅✅ — universal stack; every k8s shop knows it
- **Simple** ✅✅ — don't reinvent any layer
- **Honest** ✅✅ — the deps are real, named, and visible in
  Cargo.toml + helm charts
- **Good UX** ✅✅ — operators already know these tools

A from-scratch substrate-everything alternative would fail
every checkmark. The doctrine isn't pragmatism — it's the
discipline applied honestly.

## Existing precedents — the pattern is established

This is not a new direction for the substrate. We've made this
choice before, deliberately, for the same reasons.

### crossbeam in wat-rs proper

`wat-rs` depends on **crossbeam** for concurrent data
structures (channels, deque, atomic). This is the substrate's
foundation for cross-thread communication. If crossbeam
disappeared we'd suffer.

Why crossbeam over `std::sync` primitives or rolling our own?
- `std::sync::mpsc` is single-producer-single-consumer; we
  need MPMC
- `crossbeam::channel` is widely used; performance-tuned;
  bounded + unbounded variants
- crossbeam's data structures are part of how every
  high-performance Rust concurrent crate is built
- The maintainers ARE the people who shipped tokio's
  scheduler; they know what they're doing

This is the same shape as picking tokio for async: pick the
crate the rest of the ecosystem is converging on.

### serde via wat-edn

`wat-edn` depends on **serde** for JSON I/O. wat's native
wire format is EDN (richer types; comments; tagged literals)
but the universe speaks JSON. wat-edn translates between EDN
(in) and JSON (out, when needed) via serde + serde_json.

Why serde over rolling our own JSON?
- serde is the universal Rust serialization framework
- serde_json is the de-facto standard JSON impl
- Rolling our own would mean we couldn't use any
  serde-supporting crate's data types directly

If we ever needed to support YAML, TOML, MessagePack,
CBOR — serde gives us all of them with the same shape. **One
coupling, infinite payoff.**

## The new commitments — HTTP layer (arcs 009/010/011)

These are the dependencies we're explicitly committing to as
the HTTP layer matures:

| Crate | Role | Why this one |
|---|---|---|
| **tokio** | Async runtime | Universal Rust async; ecosystem standard |
| **hyper** | HTTP/1+2+3 | The HTTP impl; underlies reqwest, warp, axum |
| **reqwest** | HTTP client | Most-downloaded Rust HTTP client; built on hyper |
| **rustls** | TLS | Rust-native; safer than OpenSSL; tokio-friendly |
| **tower** | Middleware | Service composition primitives; standard pattern |
| **bytes** | Byte buffers | Efficient buffer management; tokio dep already |
| **hyperlocal** (or equiv) | UDS upstream | UDS support for reqwest |

Every production Rust HTTP service uses some subset of these.
**Coupling to them is coupling to "the Rust HTTP ecosystem,"
not to any specific vendor.**

## The CSP / async duality — why wat composes naturally

The deeper recognition: wat's concurrency model is
**structurally compatible with any async runtime that supports
actor-style concurrency.** Riding tokio is opportunistic, not
necessary; we get tokio's perf and ecosystem without
compromising the wat-vm's CSP semantics.

CSP says "block on channel recv until something arrives."
Async/await says "yield until `poll()` returns Ready." These
are the **same primitive** wearing different syntactic clothes.
A wat program implemented as a tokio task that calls
`recv().await` is CSP at the language level, async at the
runtime level.

The zero-mutex tiers all compose with async:

| Tier | Async-compatible? | Why |
|---|---|---|
| Immutable Arc | ✅ trivially | `Arc<T>` is `Send + Sync`; sharing across tasks is fine |
| ThreadOwnedCell | ✅ with `LocalSet` | `!Send` cells need single-thread-affinity executors; real constraint, not failure |
| Program-with-mailbox | ✅ canonically | This IS the actor model; tokio task + mpsc = canonical async actor |

None of the three tiers depend on synchronous execution. They
depend on **ownership invariants the borrow checker proves
statically.** Static proofs travel across runtime models.

The Erlang precedent: BEAM is exactly this pattern, 35 years
deep. Processes (CSP) at the language level; M:N scheduler at
the runtime; async I/O via the port system. wat-vm + tokio is
that pattern in Rust.

The honest constraints (per FAILURE-ENGINEERING — name what's
real, don't pretend it isn't):
- ThreadOwnedCell + multi-thread executor needs `LocalSet`
- Cancellation: async tasks can drop at await points; wat
  programs need explicit shutdown protocols (which the
  substrate already requires)
- Async Drop is unstable in Rust; we use explicit lifecycle,
  not Drop, for resource cleanup
- Runtime coupling to tokio is real; documented; acceptable

## The discipline — how we evaluate a candidate dependency

Before accepting a new dependency, run **the four questions
plus wat-specific tests**:

### Four questions on the dep itself

- **Obvious?** Is the dep's purpose immediately clear from
  its name + crate description? If you have to read the
  source to understand what it does, that's a signal.
- **Simple?** Does it add weight proportional to value, or
  is it a giant framework where we use 5%? Picking a 200kB
  micro-crate over a 5MB framework is usually right.
- **Honest?** Are its semantics what they appear to be?
  Does the README claim more than the implementation
  delivers? (Look for crates that ship 0.x.y → 1.0 with
  major API changes hidden in patch versions.)
- **Good UX?** Does it integrate cleanly with our existing
  deps, or do we need adapters everywhere?

### Wat-specific tests

- **Zero-mutex preserving?** Does the dep force us to wrap
  state in `Mutex` / `RwLock`? If yes, REJECT. (See
  ZERO-MUTEX.md.)
- **Type system friendly?** Does it use type-erased magic
  (e.g., `Box<dyn Any>`, raw pointer juggling) that breaks
  our type-contract guarantees?
- **Maintenance trajectory?** Is there active dev? Are
  issues responded to? Has the maintainer shipped in the
  last six months?
- **Used by canonical projects?** If tokio uses it, hyper
  uses it, the major databases use it — it's the right
  choice. If only this one author uses it — wait.
- **Cargo.lock pinning?** We pin minor versions; we audit
  bumps; we don't blindly track latest. The dep doesn't get
  to change behavior unannounced.

### Process when a candidate dep surfaces

1. Identify the actual problem (not the imagined wishlist
   problem). What concrete capability do we need today?
2. Survey: what does the rest of the Rust ecosystem use for
   this? Is there a canonical choice?
3. Apply the four questions + wat-specific tests.
4. If all pass: capture the decision in scratch (this doc or
   per-arc INDEX.yaml); add the dep; commit.
5. If anything fails: document why the candidate failed in
   scratch (so future-us doesn't re-litigate the rejection);
   keep looking OR write the small thing ourselves.

## What we DON'T depend on, and why

The doctrine cuts both ways. Here's the negative space:

- **OpenSSL** — preferring rustls. Rust-native; safer (no C
  memory issues); tokio-integrates better. OpenSSL has been
  the source of multiple critical CVEs over the years.
- **async-std** — picking tokio because it has the larger
  ecosystem; not picking BOTH (no need for runtime
  optionality at the substrate layer; users get one async
  story).
- **Heavy frameworks like axum/warp directly** — we use
  hyper as our foundation; frameworks add opinions about
  routing/middleware/state we may not share. The wat layer
  IS our framework.
- **Our own async runtime** — would be reinventing tokio
  badly.
- **Our own HTTP impl** — would be reinventing hyper badly.
- **Our own JSON parser** — would be reinventing serde_json
  badly.
- **GUI frameworks, ORMs, web frameworks at the wat layer** —
  application concerns, not substrate. If applications need
  them, they ship as sibling crates (e.g.,
  `wat-http-static`, `wat-http-oauth`) or remain in user
  code.
- **Any crate < 100 weekly downloads** — too risky; might
  not be maintained; bugs surface in our code first instead
  of in others'.
- **Any crate with a single maintainer who hasn't shipped
  in a year** — bus factor too high; we'd inherit the
  maintenance burden.

## Connection to other meta-docs

- **FUNCTIONS-ARE-REALITY** — dependencies are the lower
  functions in our stack. tokio is a function over the
  syscall layer; hyper is a function over tokio; wat-http-server
  is a function over hyper. Composition of functions, all the
  way down. We choose the lower functions deliberately because
  they shape the higher functions we can compose.
- **FAILURE-ENGINEERING** — choosing canonical battle-tested
  deps **eliminates a class of failures** preemptively. Bad
  dep choices fail repeatedly across teams over years; picking
  the deps the rest of the ecosystem has hardened means we
  inherit their failure-elimination work.
- **WAT-NETWORK** — the network layer commitments
  (k8s + istio + SPIFFE + tokio + hyper + rustls) are this
  doctrine applied to the deployment surface.
- **ZERO-MUTEX** (substrate doc, `wat-rs/docs/ZERO-MUTEX.md`) —
  why zero-mutex composes with async runtimes; the structural
  basis for tokio coupling being safe.

## Connection to per-arc work

This doctrine is what authorizes the dep choices in the
per-arc designs:

- **arc 009 (wat-http-server)** — tokio + hyper + tower
- **arc 010 (wat-http-router)** — pure wat (no Rust deps;
  inherits arc 009's deps transitively)
- **arc 011 (wat-http-client)** — reqwest + tokio + rustls
- **arc 007 (RemoteProgram)** — depends on whatever transport
  it picks (likely arc 011's wat-http-client)
- **arc 008 (wat-kwargs)** — pure wat substrate work; no new
  Rust deps

When future arcs surface new dep candidates, they should be
evaluated against this doctrine. The four questions + wat
tests + the doctrine's NOT list together gate the answer.

## What this doctrine ISN'T

Not "depend on everything that exists." We are **highly
selective** — see the NOT list above.

Not "no optionality." Interfaces are honest enough to swap
implementations if a dep faltered. We pick canonical deps
specifically because they're least likely to falter, but the
substrate isn't structurally tied to any one impl.

Not "couple to a vendor." We couple to **ecosystems and
patterns**, not specific maintainers. tokio is not "Carl's
async runtime"; it's "the canonical Rust async runtime, with
many maintainers and many production users." If the lead
maintainer disappeared, the project would survive.

Not "deps without scrutiny." Every accepted dep is captured
either in this doc or in a per-arc INDEX.yaml. Future-us can
read why we accepted each dep; we don't get to forget.

## The closing argument

The choice to couple to tokio + hyper + reqwest is the same
shape as the choice to use crossbeam in wat-rs and serde
through wat-edn. **We've made this choice before, deliberately,
for the right reasons.** We're making it again, deliberately,
for the same reasons.

The substrate is honest about its dependencies. The
dependencies are chosen carefully. The coupling is real and
documented. The interfaces are at the right layer. The
substrate gets all of mature Rust's perf and ecosystem
without compromising the wat-vm's discipline.

That's a good win. It's worth arguing for.

---

## Status

- **Captured:** 2026-05-03
- **Triggered by:** the CSP/async-coexistence recognition
  during arc 009/010/011 work; user direction to articulate
  the dep doctrine explicitly
- **Sibling docs at scratch root:** FUNCTIONS-ARE-REALITY,
  FAILURE-ENGINEERING, WAT-NETWORK
- **Cross-references:**
  - `wat-rs/docs/ZERO-MUTEX.md` — why the substrate doesn't
    need Mutex; basis for safe async coupling
  - `wat-rs/Cargo.toml` — concrete current deps (crossbeam et al)
  - `2026/05/009-wat-http-server/DESIGN.md` — concrete tokio + hyper choice
  - `2026/05/011-wat-http-client/DESIGN.md` — concrete reqwest choice
- **Update protocol:** when a new arc accepts a new
  load-bearing dep, add it to the "new commitments" section
  here OR (preferred) record the decision in the arc's
  INDEX.yaml and reference this doctrine. This doc captures
  the *position*; per-arc INDEXes capture *applications of
  the position*.
