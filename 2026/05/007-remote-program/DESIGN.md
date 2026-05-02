# RemoteProgram — DESIGN

The four-tier model + locked decisions + seven open questions
enumerated for one-at-a-time work. Each question is a section
ready to receive its answer.

---

## What RemoteProgram actually IS — typed capability access

**Reframe captured 2026-05-03 mid-flow** after the user
questioned why RemoteProgram exists at all:

> *"i questioned... why do i even want remote programs.... its
> an obvious next step .. but the protocol around it.. why do
> i want something to run on a remote host..*
>
> *its because that remote host can do something i can't... it
> could be a database.. it could be a file system i can't
> access.. its a way to do data retrieval when no such local
> access exists...*
>
> *we could 'run prgrams' that are just something like a ddb
> query.... where its basiclaly just an s3 get object... where
> we need to do an EFS read on something we don't have
> mounted... this is a way to communciate RPCs as data?.."*

**RemoteProgram is RPC-as-data.** The wat-side surface is a
typed function call. What's on the other side fulfills the
typed contract. The fact that the implementation lives
elsewhere is secondary to the fact that the wat-vm needs
typed access to a capability that lives elsewhere.

### Two framings — pick the right one

- **Remote-execution framing** (the wrong one): "I want my
  wat code to run elsewhere." Implies the other side runs wat;
  emphasizes wat-program portability.

- **Typed-capability-bridge framing** (the right one):
  "I want typed access to capabilities that live elsewhere."
  Implies the other side is whatever fulfills the contract;
  emphasizes wat as a universal client.

The second framing is much more useful. The first consumer
isn't "wat-mcp talking to LLMs" specifically — it's
**whatever capability the user needs typed access to first.**
That could be:

- An S3 / DynamoDB / EFS shim
- An LLM API (Anthropic / OpenAI / similar)
- A local sidecar service
- A pre-existing REST/gRPC API the user wraps in a wire shim
- A peer wat program (the one case where the other side IS wat)

**The wat consumer doesn't care which.** They see
`:wat::remote::Program<:I, :O>`. The typed contract is what's
promised. Implementation is implementation detail.

### What this means architecturally

You're building **a typed-capability-bridge substrate**, not a
remote-execution substrate. The implications:

1. **The wat-vm becomes the universal client.** Any capability
   gets a typed bridge; wat code talks to bridges, not
   services. The SDK problem (every external system has its
   own SDK with its own conventions) dissolves into the wire
   protocol — one wire; many bridges.

2. **The other side is polyglot.** Anything that speaks the
   wire protocol can serve a RemoteProgram call. wat program;
   Rust shim wrapping `aws-sdk-rust`; Python service; C
   process — all valid implementations of the typed contract.
   The consumer sees only the contract.

3. **The error taxonomy (Q6) gets richer naturally.** Each
   capability brings its own application errors (table-not-
   found, throttling, quota-exceeded) that ride the Err
   channel via the E enum. Transport errors (connection-lost,
   timeout, mTLS-rejected) ride the same Err channel.
   Wire-format errors (malformed-response-from-shim) too. All
   one channel; rich variants in the application's E enum.

4. **The four-tier model gains semantic clarity.** Each tier
   corresponds to where the capability lives:
   - **Tier 1 (Unix domain)**: capability lives in another
     process on the same host (sidecar pattern; local services)
   - **Tier 2 (localhost HTTP)**: capability lives in another
     process on the same host but uses HTTP framing (legacy
     services; container-network sidecar pattern)
   - **Tier 3 (remote HTTPS)**: capability lives on a remote
     host with managed TLS (managed services; SaaS APIs)
   - **Tier 4 (remote mTLS)**: capability lives on a remote
     host in a zero-trust network (peer services; service
     meshes)

   Each tier corresponds to a real category of "where capabilities
   live in modern systems." The four-tier model wasn't arbitrary
   — it was naming the actual deployment topology.

### Per the four questions on the reframe

- **Obvious?** ✅✅ — once "RPC as data" lands, the API
  shape is unmistakable
- **Simple?** ✅ — the wat-side surface is the same
  regardless of what's on the other side
- **Honest?** ✅✅ — the type contract is what's promised;
  transport is honest about being transport; bridges are
  honest about being bridges
- **Good UX?** ✅✅ — wat code accesses external systems via
  typed APIs without learning N SDKs; one mental model

Same shape as the auto-kwargs (arc 008) and Ok/Err
(this arc, Q-channel) triple-checkmark wins: **the constraint
moves into the type system, where it belongs.** The wire is
a typed conduit; the other side fulfills the type or doesn't;
nothing else matters at the wat level.

### Naming holds; semantics sharpened

"RemoteProgram" still works — the conceptual model is
program-shaped (typed input → typed output) regardless of
what implements it. The user might call shim implementations
"RemoteService" / "Bridge" / "Adapter" but the wat-side type
stays `:Program<:I, :O>`.

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

## ✅ LOCKED — Q-channel: Multiplexed Ok/Err channels — the wire IS Result<T, E>

User direction (2026-05-03):

> *"the thing i don't know how to handle at all and i think is
> the largeset unlock... [...] for remote programs... i think
> we need a transport protocol that delivers stdout and stderr
> over the 'the stdout' pipe..."*

> *"the transport protocol is labelled... when data arrives its
> either on the Ok channel or the Err channel -- we must model
> it as a multiplex... the contract is every emission must be
> declared by the sender as to what kind of emission this is"*

> *"http, tls, mtls can all build their nuanced complexity on
> top of this... the unix domain socket unlocks them all"*

This is the load-bearing protocol decision. Single wire carries
both channels; every emission labeled at the channel level;
binary classification (Ok or Err); contract enforced — sender
MUST declare which channel each emission belongs to.

### The asymmetry being solved

Three execution boundaries; three diagnostic-channel shapes:

| Boundary | input/output | diagnostic channel |
|---|---|---|
| **Threads** (in-process) | pseudo-pipes (in-memory) | panics caught in-vm |
| **Processes** (fork) | real pipes (libc) | real stderr pipe |
| **Remote** (socket) | single wire | **NOTHING — must solve** |

Without solving the remote case, RemoteProgram is broken at
the diagnostic boundary — panics either disappear silently or
corrupt the response stream. Both unacceptable for a serious
abstraction. The multiplexed Ok/Err protocol is the unlock.

### The wire IS Result<T, E>

Every emission gets a channel label. Two channels. Binary.

```edn
;; Ok-channel emission (sender's "normal output" path)
{:channel :ok :payload <T-as-edn>}

;; Err-channel emission (sender's "diagnostic/error" path)
{:channel :err :payload <E-as-edn>}
```

The handler's signature `:I -> :Result<:O, :E>` extends to the
wire literally: T is the Ok-channel payload type; E is the
Err-channel payload type. **The type system extends across the
boundary unchanged.** No impedance mismatch.

Per the four questions:

- **Obvious?** ✅✅ — every frame self-declares; receiver has
  ONE rule (read the channel field)
- **Simple?** ✅✅ — binary classification; no hierarchy of
  severities; no convention-by-content
- **Honest?** ✅✅✅ — the wire IS the Result type; the type
  system extends across the boundary by construction; an
  emission cannot exist without a channel label (contract
  enforced)
- **Good UX?** ✅ — receivers have ONE dispatch rule instead
  of N

**Triple checkmark on Honest.** Same shape as auto-kwargs
(arc 008): structural impossibility of being wrong because the
constraint is in the type system, not the convention. The path
is carved; the protocol enforces what it claims.

### Diagnostic categories live IN the E type, not in the wire

The wire doesn't proliferate frame types for "info" / "warn" /
"panic" — those are variants of the application's E enum:

```scheme
(:wat::core::enum :wat::remote::Diagnostic
  ((Info     (message :String)))
  ((Warn     (message :String) (location :Span)))
  ((Error    (message :String) (cause :HolonAST)))
  ((Panic    (trace :PanicInfo))))
```

Then the handler's signature becomes:

```scheme
(:wat::core::define
  (:my-handler
    (input :MyInput)
    -> :Result<:MyOutput, :wat::remote::Diagnostic>)
  body)
```

Or if the handler returns its own error type, the runtime
wraps panics into the standard Diagnostic enum. Either way:
the CHANNEL is binary (Ok/Err); the SUB-TYPE within Err is the
application's enum. **The wire stays simple; the language
carries the categorization.**

### Frame ordering during a call

For request-response (one-shot RPC):
- 0..N Err-channel emissions during the call (logs, warnings,
  intermediate diagnostics)
- Followed by EXACTLY 1 Ok-channel emission (the final response)
  OR a terminal Err-channel emission (panic, or handler returned
  Err)

For streaming (`:RemoteStream<I, O>`, deferred per Q5):
- 0..N Err-channel emissions interleaved
- 0..M Ok-channel emissions over time (each a value in the stream)
- Eventually a terminal frame (close-channel signal, or terminal
  Err)

The Ok/Err contract holds in both shapes. **The channel
discipline is the same regardless of one-shot vs streaming.**

### Why this unlocks the other tiers

Per user direction: *"http, tls, mtls can all build their
nuanced complexity on top of this... the unix domain socket
unlocks them all"*

The protocol layering is now:

```
LAYER 3 — TYPED PROGRAM    :wat::remote::Program<I, O>
                            (call / serve / handle lifetime)
LAYER 2 — WIRE             EDN length-prefixed frames with
                            Ok/Err channel discriminator
                            ✓ TRANSPORT-AGNOSTIC
                            ✓ THE LOAD-BEARING PROTOCOL
LAYER 1 — TRANSPORT        per-tier connect/listen primitives
                            (Unix socket / HTTP listener /
                             HTTPS / mTLS handshake)
                            ✓ ONLY THIS DIFFERS PER TIER
```

Once Layer 2 is right for Unix domain, Layers 2 and 3 are the
same across ALL four tiers. Layers 2+3 are written ONCE; Layer
1 swaps in per tier. **HTTP / HTTPS / mTLS just add their
transport-specific complexity (HTTP framing, TLS handshake, mTLS
cert auth) WITHOUT changing the upper layers.**

### Implications beyond remote — the bigger unification

The labeled-frame protocol naturally applies to threads and
processes too:

| Boundary | Labeled-frame protocol applies? |
|---|---|
| **Threads** | YES — gains a real Err channel they don't have today |
| **Processes** | OPTIONAL — could ride over native pipes OR use the protocol over one combined pipe (uniformity) |
| **Remote** | YES — the trigger; only viable shape |

If all three boundaries adopt the protocol, **wat code is
write-once / run-anywhere across local thread, forked process,
and remote**. The transport changes; the protocol stays. The
wat code that calls `:wat::io/log` works identically in all
three contexts.

That's the second-order unlock. RemoteProgram forces solving
this; the solution generalizes; the discipline propagates back
to local execution. (Documented as future-work; not gated on
RemoteProgram shipping.)

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

## ✅ ANSWERED — Q8: Who is the first consumer?

Per the 2026-05-03 reframe (see top-of-file
"What RemoteProgram actually IS"): **the first consumer isn't
a specific service — it's the typed-capability-bridge pattern
itself.** Whatever capability the user needs typed access to
first becomes the slice 1 deliverable.

The reframe shifted the question from "which wat program runs
remotely first?" to "which external capability needs typed
access first?" Candidates of the SECOND form are richer:

- An S3 / DynamoDB / EFS shim (AWS data plane)
- An LLM API (Anthropic / OpenAI / similar)
- A local sidecar service
- A pre-existing REST/gRPC API wrapped in a wire shim
- A peer wat program (the one case where the other side IS wat)

**The arc designs for the GENERAL pattern.** Whatever specific
shim ships first is a slice 1 deliverable — the user picks
based on what they need. No need to lock the specific shim at
design time; the design accommodates all of them via the
typed-capability-bridge framing.

**The consumer-specific slice 1 still determines:**
- Which tier ships first (probably the one the chosen consumer
  needs — Unix domain for local sidecars; HTTPS for AWS/LLMs;
  mTLS for service-mesh peers)
- Connection model details (Q2; persistent baseline holds)
- Streaming need or one-shot (Q5; depends on capability shape)

But these are slice-1 IMPLEMENTATION choices, not arc-shaping
DESIGN questions. The general pattern accommodates all.

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
