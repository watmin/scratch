# RemoteProgram — DESIGN

The four-tier model + locked decisions + seven open questions
enumerated for one-at-a-time work. Each question is a section
ready to receive its answer.

---

## The four questions are the design compass

Per the established discipline (carried from the foundation
arcs 003-006):

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape
  per concept.
- **Honest?** What's named matches what's there; invariants
  are enforced by the type system, not by convention.
- **Good UX?** A user can do the right thing without ceremony.

Obvious + Simple + Honest must hold before Good UX is even
considered.

## The four-tier model — locked

User direction (2026-05-03):

> *"alright.. RemoteProgram... there's tiers to this...
> unix domain
> localhost http
> remote https
> remote mtls
> -- that's it - we don't allow clear text over the network"*

| Tier | Transport | Auth | Network? | Encryption |
|---|---|---|---|---|
| 1 | Unix domain socket | Process boundary (file perms) | No | N/A (local) |
| 2 | Localhost HTTP | None / local trust | No (loopback) | N/A (local) |
| 3 | Remote HTTPS | Server cert verify | Yes | TLS |
| 4 | Remote mTLS | Mutual cert verify | Yes | TLS + client auth |

**Conspicuously absent: remote HTTP** (clear-text over the
network). The no-clear-text invariant is enforced at the TYPE
SYSTEM level — there's no constructor for remote-without-
encryption. **Honest by construction.**

Per the four questions:
- **Honest?** ✅✅ load-bearing — the type system makes the
  invariant unbreakable; users cannot accidentally make a
  clear-text remote call because the constructor doesn't
  exist
- **Obvious?** ✅ — four constructors, one per tier; the
  user sees what their dependency is using
- **Simple?** ✅ — one rule (no clear text over network);
  enforced by absence (no constructor)
- **Good UX?** ✅ — security right by default; no
  configuration mistake can break the invariant

## Sketch type signatures (for illustration)

Subject to revision when Q1 (transport-as-config vs
transport-as-type) is answered:

```scheme
;; The type
:wat::remote::Program<:I, :O>

;; Construction — one constructor per tier
(:wat::remote::Program/local-unix
  :path :Path
  -> :Result<:Program<:I, :O>, :ConnectError>)

(:wat::remote::Program/local-http
  :host :String
  :port :i64
  -> :Result<:Program<:I, :O>, :ConnectError>)

(:wat::remote::Program/remote-https
  :host :String
  :port :i64
  -> :Result<:Program<:I, :O>, :ConnectError>)

(:wat::remote::Program/remote-mtls
  :host :String
  :port :i64
  :client-cert :Path
  :client-key  :Path
  :ca-cert     :Path
  -> :Result<:Program<:I, :O>, :ConnectError>)

;; Invocation — same shape regardless of tier
(:wat::remote::Program/call
  (program :Program<:I, :O>)
  (input :I)
  -> :Result<:O, :RemoteError>)
```

Whether all four constructors return the same `:Program<:I, :O>`
or distinct types is Q1's question.

## ✅ LOCKED — Q3: Wire format = EDN

User direction (2026-05-03):

> *"the first one... edn is the transport protocol..."*

**Wire format: EDN over the wire.** Length-prefixed EDN frames;
round-trips through `wat::edn::write` / `wat::edn::read`. Same
shape wat-edn already provides for other consumers.

**Frame format** (sketch; details for slice-time):
```
[4 bytes: u32 big-endian length] [N bytes: EDN payload]
```

Length prefix lets the receiver allocate exactly; EDN payload
parses via the existing `wat-edn` machinery.

**For heterogeneous-server scenarios** (server isn't wat-aware
and speaks JSON instead): the same `wat-edn` crate provides
`edn_to_json` / `json_to_edn`; clients can negotiate over
content-type or use the JSON formatter explicitly. JSON is
secondary; EDN is canonical.

Per the four questions:
- **Obvious?** ✅ — one wire format; same payload shape
  client-side and server-side
- **Simple?** ✅ — reuses existing wat-edn infrastructure;
  no separate codec
- **Honest?** ✅ — EDN faithfully represents typed values;
  no impedance mismatch between wire and in-memory
- **Good UX?** ✅ — wat developers already know EDN; no
  separate wire format to learn

---

## Open questions — work one at a time

Each section below is a question waiting to be answered.
Ordered as Q8 first (the consumer grounds everything), then
Q1, Q2, Q4 (load-bearing architecture), then Q5, Q6, Q7
(downstream specifics).

---

## ❓ OPEN — Q8: Who is the first consumer?

Knowing the first concrete consumer grounds every other
decision. SimpleCov-style: design for the real use case;
generalize when a second consumer surfaces.

**Candidates I can imagine** (user knows better):
- Trading lab calling exchange APIs (HTTPS / mTLS for some
  venues)
- wat-mcp talking to LLM APIs (HTTPS to Anthropic / OpenAI)
- Inter-service comms in a future wat-microservices shape
  (Unix domain or mTLS)
- Distributed wat-vm calls (Unix domain locally; mTLS across
  machines)
- A telemetry collector / shipping pattern (mTLS to a remote
  sink)

**The consumer determines:**
- Which tiers ship in slice 1 (probably whichever the first
  consumer needs)
- Persistent vs per-call connection model (Q2)
- Whether server-side ships in v1 (Q4)
- Streaming need or one-shot (Q5)

**Awaiting user answer.**

---

## ❓ OPEN — Q1: Transport-as-config vs transport-as-type

Are all four tiers the same `:wat::remote::Program<:I, :O>`
(transport is configuration; constructors differ but type
unifies), or do they get distinct types
(`:UnixProgram<:I, :O>`, `:HttpsProgram<:I, :O>`,
`:MtlsProgram<:I, :O>`)?

**Option A — Single type, transport-as-config:**
- One `:Program<:I, :O>` type
- Four constructors that all return the same type
- Consumer code is transport-agnostic (calls `Program/call`
  regardless of which tier the program was constructed with)
- Hides transport behind the type contract

**Option B — Distinct types per tier:**
- `:UnixProgram<:I, :O>`, `:HttpsProgram<:I, :O>`, etc.
- Four constructors, four types
- Consumer code KNOWS which tier (and can refuse e.g.
  `:UnixProgram` if it requires `:MtlsProgram`)
- Tier-specific operations possible (e.g.,
  `:MtlsProgram/renegotiate`)

**Trade-offs:**
- A is simpler (one type to reason about) and matches "transport
  is implementation detail"
- B is more honest (the type tells you what you're connected to)
  and matches "security tier is a contract"
- A composes with libraries that don't care about transport; B
  forces consumers to be transport-specific

**Awaiting user answer.**

---

## ❓ OPEN — Q2: Connection model — persistent or per-call?

Three candidates:

**A — Persistent**: `connect` returns a long-lived program;
many `call`s reuse the connection; explicit `disconnect` to
release. Matches wat-rs's existing `Channel` /
`ProgramHandle` patterns.

**B — Per-call**: every `call` is a fresh request (HTTP-style).
Stateless from the caller's view; transport handles
connection pooling internally.

**C — Both**: explicit choice at construction. `connect-once`
returns a per-call wrapper; `connect-persistent` returns a
long-lived program with explicit lifetime.

**Trade-offs:**
- A is consistent with kernel patterns; lower per-call latency
  for many calls; requires explicit lifetime management
- B is simpler API; matches HTTP mental model; higher per-call
  latency (handshake every time)
- C is most flexible; user picks per-use-case; more API surface

For Tier 1 (Unix domain), persistent is natural (sockets are
long-lived). For Tier 3/4 (HTTPS/mTLS), the cost of TLS
handshake per call argues for persistent. For Tier 2
(localhost HTTP), either works.

**Awaiting user answer.**

---

## ❓ OPEN — Q4: Server side — in scope for v1?

Client side is unambiguous: construct, call, get result. Server
side is the dual: accept incoming connections, type-check
input, dispatch to handler, return typed output.

**Should server-side ship in v1, or as a separate arc?**

If server-side IS in scope:
```scheme
(:wat::remote::Program/serve
  :transport :unix-or-https-or-mtls
  :path-or-host :Path-or-String
  :handler (fn (input :I) -> :Result<:O, :HandlerError>)
  -> :Result<:Server, :ServeError>)
```

The serve-side mirrors the construct-side; same four tiers
(unix / localhost-http / remote-https / remote-mtls); same EDN
wire format; same type contract.

**Trade-offs:**
- v1 client-only: smaller scope; faster to ship; assumes some
  external server (could be another wat program in a different
  process, or a non-wat service)
- v1 client + server: complete loop; can build wat-to-wat
  service ecosystems immediately; larger scope

**Awaiting user answer.**

---

## ❓ OPEN — Q5: Streaming — sibling abstraction or included?

Per `:wat::kernel::Channel<T>`, duplex streaming is a substrate
primitive. `RemoteProgram<:I, :O>` is request/response (one
input → one output).

**Should `:RemoteStream<:I, :O>` ship as a sibling abstraction?**

- Same four-tier transport model
- Bidi streaming over the same wire format
- Channel-shaped from the consumer's view: send inputs, receive
  outputs, both can flow continuously

If shipped as a sibling: separate type
(`:wat::remote::Stream<:I, :O>`); separate constructors;
shares EDN wire-format machinery with `:RemoteProgram`.

If NOT shipped: streaming is a future arc; v1 is one-shot only.

**Trade-offs:**
- Including streaming widens v1 scope but answers the streaming
  use case from day one
- Excluding makes v1 simpler; streaming arc opens later when
  a real streaming consumer surfaces

**Awaiting user answer.**

---

## ❓ OPEN — Q6: Error taxonomy — shape of `:RemoteError`

`:RemoteError` likely needs to distinguish multiple failure
modes:

- `Network` — couldn't reach (connection refused, DNS failure,
  unreachable network)
- `Timeout` — reached but took too long (configurable timeout)
- `Auth` — authentication rejected (mTLS handshake failed,
  bearer-token rejected, etc.)
- `Server` — server returned an error response (the handler
  returned `Err`)
- `TypeMismatch` — server returned data not of declared type O
  (wire format / EDN parsed but didn't match expected type)
- `Encoding` — wire format errors (malformed EDN, truncated
  frame, etc.)

**Candidate shape:**
```scheme
(:wat::core::enum :wat::remote::RemoteError
  ((Network (details :String)))
  ((Timeout (timeout-ms :i64)))
  ((Auth    (details :String)))
  ((Server  (details :String) (server-err :SomeServerErrorEnum)))
  ((TypeMismatch (expected :String) (got :String)))
  ((Encoding (details :String))))
```

**Open variants of the question:**
- Should `Server` carry a typed error payload (parameterized
  on the server's error enum), or just an opaque String?
- Are there tier-specific error variants (e.g.,
  `MtlsHandshakeFailed` distinct from `Auth`)?
- Should the enum be extensible by consumers, or sealed?

**Awaiting user answer.**

---

## ❓ OPEN — Q7: Hermetic / sandbox interaction

Per arc 058-035 (fork-substrate), wat-rs has hermetic test
execution via fork. Each hermetic child is a forked process.

**Can a hermetic child make remote calls?** Should it?

**Considerations:**
- Forked child inherits parent's network state (open sockets,
  routing tables, etc.) on most OSes
- Tests that depend on remote services are FRAGILE (network
  flakiness); some test discipline says "tests should never
  hit the network"
- Some tests genuinely NEED remote calls (integration tests
  against a known-good staging endpoint)
- Local unix-domain calls are different — they're not flaky;
  they're a contract test against another local process

**Possible stances:**
- **A — Allow all tiers in hermetic children.** User
  responsibility to control what the test calls.
- **B — Allow only Tier 1 (unix domain) in hermetic children
  by default; require explicit opt-in for network tiers.** The
  default is "tests don't hit the network."
- **C — Forbid remote calls entirely in hermetic children.**
  Network calls must happen at the parent level; forks can't
  make them.

**Awaiting user answer.**

---

## When the design firms up

Once Q8 + Q1 + Q2 + Q4 are answered, the architecture is
solid enough to draft SLICE-PLAN.md. Q5 + Q6 + Q7 can be
answered before or after slicing without changing the
fundamental shape.

Status: design in-progress. Edit this file directly as
questions get answered; flip ❓ OPEN to ✅ LOCKED with the
verdict and reasoning.
