# The wat network — meta-vision

A top-level companion to the per-arc design files (003-008,
plus arc 007). The arcs are pieces; this document is the whole
they're pieces of.

User direction (2026-05-03, immediately after the
"RPC-as-data" reframe of arc 007):

> *"do you understany why i want mtls support now? the query
> connection is a signed entity.. its credential bearing.. the
> caller must self identify and the receiver must verify.. you
> cannot run your program on the remote host if the remote host
> won't allow it.. and we can have signed queries too - i put
> digest and signed eval forms in.... i've been building
> toward this... the wat network"*

What the user has been building toward, articulated.

---

## What the wat network IS

A network of mutually-authenticating wat-vms where:

1. **Each node has cryptographic identity** — cert / keypair.
   Not network-position-based; not magic-cookie; not bearer-
   token. Real cryptographic identity.

2. **Connections are mTLS** — both ends verify each other.
   You can't connect to a node that won't accept your cert; the
   receiver knows exactly who's calling.

3. **Queries / programs sent for execution are SIGNED** — the
   receiver verifies the signature before evaluating. The
   substrate ships `digest` and `signed eval` forms for exactly
   this discipline.

4. **Authorization is cryptographic, not network-positional**
   — "who signed this; do I trust them; do they have permission
   to ask this" — not "did this packet arrive on the right
   interface."

5. **Programs are content-addressed via digest** — immutable;
   identifiable; cacheable; versionable. "Run program with
   digest X" is a network primitive, not a string.

This is **a distributed capability-and-computation substrate.**
Not just RPC — a peer-to-peer network where computation,
identity, and authorization are first-class wat-vm concepts.

## The three load-bearing primitives

The network rests on three substrate primitives, each of which
the user has either shipped or is shipping:

### Primitive 1 — Cryptographic identity (mTLS-based)

Every node has a cert. Connections are mutually verified. The
network's membership protocol is the cert chain — you're a
member if your cert is signed by a trusted authority (or peer-
trusted via a web-of-trust pattern).

mTLS isn't a deployment-time security knob. It's the network's
**membership protocol.** Without it, there's no wat network;
there's just isolated wat-vms that happen to communicate.

### Primitive 2 — Content-addressed programs (digest forms)

Programs are HolonAST. AST is data. Data has a content hash.
That hash is the program's IDENTITY.

Implications:
- Two programs with the same code have the same digest
- "Send me the program with digest X" is unambiguous
- Caching is trivial (memoize by digest)
- Versioning is trivial (different code → different digest)
- Provenance is verifiable (digest in a signed envelope =
  cryptographic claim about what code is being referenced)

### Primitive 3 — Verifiable execution (signed eval forms)

A program (HolonAST) plus a signature plus a public key gives
you a verifiable claim: "this exact program was authorized by
this exact identity." The receiver checks:
- Does the signature verify against the claimed public key?
- Does the public key correspond to a trusted identity?
- Is that identity authorized to request this kind of work?

Only signed-by-trusted-party code is evaluated. **Code is data
is signed-data.** The wat-network's authorization model is
built on this trichotomy.

These three primitives compose. Together they enable
distributed computation with cryptographic provenance —
something you can't do with bare TCP, bare HTTP, or even bare
RPC frameworks.

## Mini AWS on a laptop — the local testbed for distributed systems

User direction (2026-05-03, after WAT-NETWORK.md initial draft):

> *"one thing to keep in mind.. somewhere in the wat-rs/docs/
> ... there's an expression.. i think its an fpga doc?..
> whatever...*
>
> *i modeled the wat-vm to be a mini aws on my laptop... comms
> between stateful things are 'rpc-like' on purpose.. the lru
> cache service is like a redis.. the console service is like
> an ecs output stream.. the telemetery service is both
> cloudwatch logs and metrics...*
>
> *the system was always meant to be distributed.. but i needed
> a local representation with the same constraints to realize
> it"*

**The substrate has been distributed-systems-shaped from the
start.** Every architectural choice has been made to be CORRECT
for distribution, not just convenient locally. The wat-vm is a
LOCAL TESTBED for distributed-systems patterns; the eventual
extension to a real network is a natural continuation, not a
pivot.

### The complementary framings

Two architectural framings already in the substrate
documentation:

**Framing 1 — Programs as circuits (FPGA-shaped)**

`wat-rs/docs/CIRCUIT.md` articulates this: a wat program is a
circuit. Programmer-built. Fixed topology. Signals flow
through wires once powered. The substrate IS programmable in
the FPGA sense — you arrange the gates at programming time;
once running, the wiring is fixed.

`:user::main` constructs all pipes, plugs each into its
consumer, and starts the input stream. That's its entire job.
No computation in main; no I/O; no state. Just wiring. **The
shape of `:user::main` is the wiring diagram.**

**Framing 2 — Services as AWS analogs (the mini-AWS framing)**

The services running inside each wat-vm are deliberately
shaped like distributed-system primitives. The user's
verbatim mapping:

| wat-vm service | AWS analog |
|---|---|
| LRU cache service (`:wat::lru::CacheService`) | Redis / ElastiCache |
| Console service (stdio output stream) | ECS / Fargate container output |
| Telemetry service (`:wat::telemetry::*`) | CloudWatch Logs + Metrics |

Plus the kernel primitives that make it all work:
- Channels = inter-service messaging (like SQS / EventBridge)
- HandlePools = connection pooling (like RDS Proxy)
- Spawn / fork = service instances (like ECS task launches)
- Hermetic test execution = sandboxed runs (like Lambda invocations)
- mTLS + signed eval (when shipped) = IAM + cryptographic
  request signing

**Each wat-vm IS a mini AWS on the laptop.** The constraints
(typed channels everywhere, RPC-like service comms, no shared
memory shortcut, service isolation, authentication) are
EXACTLY the constraints distributed systems face. By
prototyping locally with those constraints, the eventual
distribution becomes possible WITHOUT REWRITES.

### Why these two framings compose

- The CIRCUIT framing tells you what each wat-vm IS internally:
  a fixed-topology arrangement of services + wires (a circuit;
  an FPGA-configured node).
- The MINI-AWS framing tells you what each wat-vm DOES: hosts
  stateful services that talk RPC-like (a node in a
  distributed system).
- The WAT-NETWORK framing tells you how nodes COMPOSE: many
  circuits, each a mini-AWS, connected via mTLS + signed
  queries + content-addressed programs.

**The recursion is intentional.** A wat program is a circuit.
A circuit hosts services. Services talk RPC-like. The wat-vm
runs the circuit. Multiple wat-vms form a network. The
network's nodes talk RPC-like (RemoteProgram). The internal
patterns scale to the external patterns because the patterns
were chosen FOR distribution from the start.

### What this means for the wat network architecture

The wat network isn't "let's add networking to a local
substrate." It's "the distributed-systems patterns the
substrate was already shaped around finally extend to multiple
machines."

The user has been building the local testbed for years. The
discipline (typed channels; service isolation; RPC-like
comms; no shared memory shortcut; content-addressed programs;
signed eval; mTLS) was always going to make the move to the
network straightforward — because the constraints have been
honored locally all along.

When a node joins the wat network, it's the same wat-vm
running the same kind of circuit hosting the same kind of
services. It just gains the ability to call OTHER nodes'
services via RemoteProgram (Tier 4: in-network mode). The
patterns inside each node and the patterns between nodes are
the same patterns. **Recursion all the way down.**

This is part of what makes ✅✅✅ Honest land for the wat
network: the network model isn't a layer bolted onto local
semantics; it's local semantics extended to multiple
machines. The constraints honored locally are the constraints
that make distribution work. Not "we'll figure out
distribution later" — "distribution is what we've been
preparing for."

## Backpressure and rhythm — flow control as a structural property

User direction (2026-05-03, after the mini-AWS framing):

> *"you can review the zero mutex one too - its modeled after
> tcp... and the comms between services.. the stdin,out pipes..
> those are like queues.. and the blocking is a nature backoff,
> release valve..*
>
> *the system has a rythem to it.. the slowest component isn't
> constant but the system is held back by whatever the current
> slow thing is"*

### What's already in the substrate

`wat-rs/docs/ZERO-MUTEX.md` articulates the local mechanism:
**mini-TCP via paired bounded(1) channels.** Producer writes on
one pipe; blocks on the companion pipe until the consumer
signals "done." Two pipes per producer; mutually blocking
through the substrate's rendezvous discipline.

The substrate refuses (compile-time, arc 126) to construct
unbounded shared state. Every channel is bounded; every
producer can be blocked; the architecture FORCES a flow-control
discipline at every comm boundary.

### The new framing — backpressure as natural rhythm

The user's insight extends the mini-TCP framing into a
runtime property:

- Bounded channels + blocking = backpressure built in
- The slowest component throttles upstream producers
  AUTOMATICALLY; no explicit flow-control protocol; no
  rate-limiter; no backoff calculation
- The bottleneck SHIFTS over time depending on workload;
  it's not a constant
- The system finds its natural pace at any moment based on
  what's currently slow
- **The system has a rhythm.** The pace isn't hardcoded; it's
  emergent from the pipeline's current bottleneck

This is the same flow-control discipline TCP uses (window
size, backoff). Same as any healthy queue-based system. The
substrate doesn't bolt flow control on; **flow control is what
bounded channels with blocking ARE.**

### Why this matters for the wat network

When nodes communicate via RemoteProgram, the wire is the
channel; bounded buffering at each end provides natural
backpressure; if a remote node is overloaded, the local caller
blocks; **the system finds equilibrium without explicit flow
control across the wire.**

The local pattern (bounded channels; producer blocks until
consumer is ready; system rhythm emerges) extends to the
network pattern (bounded wire buffers; remote caller blocks
until remote handler is ready; cluster rhythm emerges).

**Backpressure is structural, both locally and across the
network.** Not aspirational; not bolted on; not "remember to
add rate limiting." The substrate's discipline is: every
bounded channel CAN block; the producer WILL block when the
consumer is slow; the system finds its pace.

### Per the four questions on rhythm

- **Obvious?** ✅ — bounded channels with blocking; the
  pace is visible at runtime via the queue depths
- **Simple?** ✅ — one mechanism (block on full queue); no
  separate flow-control protocol; no rate-limiter API
- **Honest?** ✅✅ — the slowest component IS the bottleneck;
  nothing pretends otherwise; no hidden buffering that would
  lie about effective throughput; the system literally cannot
  pretend to be faster than its slowest part
- **Good UX?** ✅ — wat code doesn't have to think about flow
  control; the substrate handles it; same code works at any
  pace

Strong shape. Not triple-checkmark (rhythm is a derived
property, not a fundamental architectural decision), but
✅✅ Honest is meaningful: the substrate cannot lie about
throughput because the channels block honestly.

### The fourth pattern that scales naturally

Adding to the list of "patterns the substrate honors locally
that scale naturally to the network":

| Pattern | Local | Network |
|---|---|---|
| Typed channels | `:wat::kernel::Channel<T>` | RemoteProgram's typed contract |
| RPC-like service comms | mini-TCP via bounded(1) | RemoteProgram call/response |
| Service isolation | spawn-program; one-thread-per-service | nodes in the network |
| **Backpressure** | **bounded channels + blocking** | **bounded wire buffers + blocking** |
| Cryptographic identity | (per-process; OS-level) | mTLS per-node |
| Content-addressing | (digest forms in the substrate) | digests carry across the wire |
| Verifiable execution | (signed eval forms in the substrate) | signed queries between nodes |

Every row is the same pattern at different scales. The local
discipline IS the distributed discipline. The substrate has
been distributed-from-the-start because it has been
constraint-honoring-from-the-start; the constraints are the
constraints distribution requires.

## Deployment + identity overlay — composes with cloud-native infrastructure

User direction (2026-05-03, after the rhythm framing):

> *"on the wat network... the mtls part... it natively slots into
> k8s with istio... spire and spiffie.. ya?...*
>
> *the side cars bounce connections based on cert identity....
> who can do what... and the queries carried on these signed
> connections.. they can be signed too... the caller is trusted
> and the payload is trusted...*
>
> *callers in differnet envs.... maybe some k8s box is in aws
> another in gcp.. and another host is in someone's home lab..
> if the home lab does a signed connection with a signed
> payload those in-cloud-apps could reach into their local
> cloud resources with their cloud native identities (some
> container in aws querying some ddb table and serving the
> result, s3, efs, rds, lambda func call - whatever).. the
> mtls fronted connection is a way for a completely independent
> identity system to overlay on all existing identities... this
> is an abstraction layer..*
>
> *when we shift to 'this :some-identity is allowed to query
> :some-resource with :some-scope' to being an edn delivery
> mechanism... the who and where dissolve.. all that matters
> is the contract...*
>
> *this is what remote programs deliver... cloud agnostic data
> relaying... do you see it?...*
>
> *there's no reason why a wat program couldn't implement an
> http interface that sits behind some tls termination side
> car who does authn+authz and then the application can do
> signed payload validation... yea?..."*

This is the deployment story made concrete + a load-bearing
insight about how the wat network composes with existing
cloud-native identity systems.

### The wat network slots natively into k8s + istio + SPIFFE/SPIRE

The mTLS membership protocol of the wat network isn't a custom
thing the user has to build infrastructure for. It's already
the standard cloud-native identity pattern:

| Existing infra | Role |
|---|---|
| **k8s** | Container orchestration; service mesh substrate |
| **istio** (or Linkerd / Consul Connect) | Service mesh; sidecar proxies for mTLS termination + identity-based routing |
| **SPIFFE / SPIRE** | Workload identity; per-workload SVIDs (cert + key); rotation; attestation |
| **Sidecar pattern** | TLS termination + authn + authz at the network edge; application receives plain HTTP locally |

A wat-vm deployed in k8s gets its identity via SPIRE; its
connections are mTLS-wrapped by the istio sidecar; the istio
sidecar enforces "who can talk to who" at the network layer
based on cert identity. **The wat network's mTLS membership
protocol IS what istio + SPIFFE already do.**

This means deploying the wat network in production:
- Uses existing k8s + istio + SPIFFE infrastructure
- Doesn't require rebuilding the network stack
- Doesn't require operators to learn a new mTLS system
- Familiar to anyone who's deployed a service mesh

The wat-specific value lives at the APPLICATION layer:
- Signed payloads (the substrate's `signed eval` forms)
- Content-addressed programs (the substrate's `digest` forms)
- Typed contracts (the substrate's type system)
- The Q-channel wire-IS-Result-type discipline

### The dual-layer identity model

When you compose istio's network-layer mTLS with wat's
application-layer signed payloads, you get **two independent
layers of cryptographic verification**:

```
┌────────────────────────────────────────────────────────────┐
│                 NETWORK LAYER (istio sidecar)              │
│                                                            │
│  - mTLS handshake                                          │
│  - Cert identity verification (SPIFFE SVID)                │
│  - Network-level authz (who can talk to who)               │
│  - TLS termination                                         │
│                                                            │
│  Application receives PLAIN HTTP locally with verified     │
│  identity headers from the sidecar                         │
└─────────────────────────┬──────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────┐
│              APPLICATION LAYER (wat-vm program)            │
│                                                            │
│  - Signed payload validation (signature against trusted    │
│    keys via :wat::core::signed-eval forms)                 │
│  - Application-level authz (does this signed identity      │
│    have permission to ask THIS specific query?)            │
│  - Content-addressed program verification (digest match)   │
│  - Typed contract enforcement (Result<T, E> wire shape)    │
└────────────────────────────────────────────────────────────┘
```

**Two layers; two independent cryptographic verifications.**
A spoofer would have to break BOTH layers to forge a request.
Network-layer compromise doesn't invalidate application-layer
signature verification. Application-layer signature alone
doesn't get past the network-layer mTLS.

### The identity-overlay insight

This is the load-bearing observation:

**The wat-network identity system is COMPLETELY INDEPENDENT
of cloud identity systems. It OVERLAYS on top of them.**

Each wat-vm node:
- Has its OWN cryptographic identity (cert-A, cert-B, etc.)
  for wat-network membership
- ALSO HAS its local cloud identity (AWS IAM role; GCP service
  account; Azure managed identity; or no cloud identity at all
  for a home lab box)

When wat-vm-A (deployed in AWS, IAM role X) calls wat-vm-B
(deployed in GCP, service account Y) via RemoteProgram:
- mTLS handshake authenticates A as cert-A and B as cert-B
  (wat-network layer)
- The signed payload carries provenance: "cert-A is asking
  for this query"
- wat-vm-B receives the call; verifies; if it decides to
  service the request, it uses ITS LOCAL GCP IDENTITY
  (service account Y) to access GCP resources (a Cloud
  Storage bucket, a Spanner table, a BigQuery dataset)
- The result is signed by cert-B and returned to wat-vm-A
- wat-vm-A verifies B's signature; uses the result; perhaps
  uses ITS LOCAL AWS IDENTITY (IAM role X) to write the
  result to an S3 bucket

**Cloud identities are local resource access. Wat-network
identity is the common language between nodes. They compose.**

### The cross-environment scenario (verbatim from user)

> *"callers in differnet envs.... maybe some k8s box is in aws
> another in gcp.. and another host is in someone's home lab..
> if the home lab does a signed connection with a signed
> payload those in-cloud-apps could reach into their local
> cloud resources with their cloud native identities..."*

A concrete scenario:

```
┌─────────────────────────┐         ┌─────────────────────────┐
│  Home lab box           │         │  AWS k8s pod            │
│                         │ mTLS    │                         │
│  Identity: cert-HL      ├────────►│  Identity: cert-AWS     │
│  No cloud identity      │ (SPIFFE │  IAM role: app-data-r/o │
│                         │  + ist.)│                         │
└─────────────────────────┘         └─────────┬───────────────┘
                                              │
                                              │ AWS-native
                                              │ (boto3 / SDK
                                              │  via IAM)
                                              ▼
                                    ┌─────────────────────────┐
                                    │  AWS resources          │
                                    │  - DynamoDB             │
                                    │  - S3                   │
                                    │  - RDS                  │
                                    │  - Lambda               │
                                    │  - EFS                  │
                                    └─────────────────────────┘
```

Home lab box has NO AWS IAM role; CAN'T directly access AWS
resources. AWS k8s pod HAS the IAM role; CAN access AWS
resources. The wat-network connection lets the home lab box
ASK the AWS pod for a typed result derived from AWS resources.
The signed payload + signed connection prove the home lab is
authorized to make the ASK; the AWS pod's local IAM proves it's
authorized to make the AWS API call.

**Same pattern works across any cloud combination.** GCP →
AWS; Azure → GCP; home lab → any cloud; cloud → home lab; etc.

### The framing shift — from configuration to delivery

The user's articulation of what this enables:

> *"when we shift to 'this :some-identity is allowed to query
> :some-resource with :some-scope' to being an edn delivery
> mechanism... the who and where dissolve.. all that matters
> is the contract..."*

The traditional cross-cloud identity story is a CONFIGURATION
problem:
- Set up cross-account IAM roles in AWS
- Set up workload identity federation in GCP
- Configure managed identities in Azure
- Configure trust between clouds
- Pray it all works

The wat-network identity story is a DELIVERY problem:
- Wat-vm-A signs an EDN payload: "I want this typed result"
- Wat-vm-B receives the payload; verifies the signature;
  decides whether to service it
- That's the entire question

**Who is asking** (cert-A)
**What they want** (the typed query)
**Where they're asking from** (anywhere — home lab; AWS; GCP)
— **all dissolve into the contract.** The authz decision is:
"do I trust this signed identity to ask for this specific
contract?" Nothing about cross-cloud configuration; nothing
about IAM federation; nothing about trust documents.

Just: signed identity + typed contract + authz decision.

### The implementation pattern (verbatim from user)

> *"there's no reason why a wat program couldn't implement an
> http interface that sits behind some tls termination side
> car who does authn+authz and then the application can do
> signed payload validation... yea?..."*

YES. The deployment shape:

```
┌─────────────────────────────────────────────────────────────┐
│  k8s pod                                                    │
│                                                             │
│  ┌─────────────────────────┐  ┌────────────────────────┐    │
│  │  istio sidecar (envoy)  │  │  wat-vm container      │    │
│  │                         │  │                        │    │
│  │  - mTLS termination     │  │  - HTTP listener       │    │
│  │  - SPIFFE SVID identity │  │    (plain HTTP from    │    │
│  │  - L4/L7 authz          ├──┤     sidecar locally)   │    │
│  │  - rate limiting        │  │  - Signed payload      │    │
│  │  - tracing              │  │    verification        │    │
│  │                         │  │  - Application authz   │    │
│  └─────────────────────────┘  │  - Local resource      │    │
│                               │    access (via cloud   │    │
│                               │    IAM if applicable)  │    │
│                               └────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

The sidecar handles ALL the network-layer crypto. The wat-vm
sees plain HTTP locally with verified identity headers. The
wat-vm does its OWN application-layer crypto (signed payload
verification). Two layers; clean separation.

This is **the standard k8s + istio service deployment pattern**
with wat-vm as the application. Operators already know this
pattern. The wat-specific value is in the application's
signed-payload + typed-contract logic.

### Cloud-agnostic data relaying — what RemoteProgram delivers

User: *"this is what remote programs deliver... cloud agnostic
data relaying..."*

Articulated:

**RemoteProgram is the substrate-level primitive for
cloud-agnostic data relaying.** It lets wat-network nodes
deployed anywhere (any cloud; no cloud; mixed environments)
exchange typed data with cryptographic provenance. Each node
uses its LOCAL cloud-native identity to access its LOCAL
resources; the wat-network identity is the common language
between nodes.

This dissolves a class of problems that traditionally requires
substantial cross-cloud infrastructure work:
- **Multi-cloud data plane**: wat-vms in AWS + GCP + Azure +
  home lab all participate in the same wat network; data
  flows between them via typed RemoteProgram calls
- **Hybrid cloud**: on-prem wat-vms + cloud wat-vms speak the
  same protocol; on-prem can fetch from cloud; cloud can
  fetch from on-prem
- **Cross-account / cross-org**: separate AWS accounts (or
  separate organizations entirely) can be wat-network peers
  without setting up IAM federation; the wat-network identity
  layer is independent of AWS-internal trust
- **Edge → cloud → edge**: edge devices with limited cloud
  identity can participate in the wat network; sign their
  payloads; trust the responses they get back

This isn't theoretical. The infrastructure pieces all exist:
- k8s for compute
- istio (or Linkerd / Consul) for service mesh + mTLS
- SPIFFE / SPIRE for workload identity
- Cloud-native IAM for local resource access
- The wat substrate for typed payloads + signed eval +
  digest-addressed programs + RemoteProgram protocol

**The wat network is what assembles them into a cloud-agnostic
data plane.**

### Per the four questions on the deployment + identity-overlay framing

- **Obvious?** ✅ — the deployment pattern is k8s-native;
  the identity-overlay concept is conceptually simple
  (wat-network identity is independent of cloud identity;
  they compose)
- **Simple?** ✅ — uses existing infrastructure (istio,
  SPIFFE, k8s) at the network layer; wat adds value at the
  application layer; no new infrastructure to invent
- **Honest?** ✅✅✅ — **fifth triple-checkmark of the
  design session** — TWO LAYERS of cryptographic
  verification (network mTLS + application-layer signed
  payload) compose to make spoofing structurally impossible;
  wat-network identity composes with cloud identities
  without conflict (overlay, not replacement); the contract
  IS the truth (who/where dissolve; only the typed contract
  matters)
- **Good UX?** ✅✅ — operators use familiar k8s tooling;
  developers use typed APIs; cross-cloud / hybrid /
  multi-environment scenarios collapse to "make a
  RemoteProgram call"

The fifth triple-checkmark on Honest is structural: the dual-
layer crypto (mTLS at network; signature at payload) means
the system literally cannot be spoofed at any layer. Identity
composition is structural — wat-network identity overlays cloud
identities without impedance mismatch. **The honesty isn't
aspirational; it's enforced by the cryptographic substrate at
both layers + the type system carrying the contract.**

This is the same shape as the other four triple-checkmarks
this session: the constraint lives in the type system + the
cryptographic primitives, not in convention. You cannot
participate without dual-layer identity proof; you cannot
make a request without a signed typed payload; you cannot
spoof a node without breaking BOTH the network and
application crypto.

## The architecture

```
┌─────────────────────────────┐         ┌─────────────────────────────┐
│   Node A (wat-vm)           │         │   Node B (wat-vm)           │
│                             │ mTLS    │                             │
│   identity: cert-A          ├────────►│   identity: cert-B          │
│                             │         │                             │
│   trusts: cert-B, cert-C... │         │   trusts: cert-A, cert-D... │
└─────────────────────────────┘         └─────────────────────────────┘

Messages between nodes:
  - Signed query  : "evaluate this AST (signed by cert-A)"
  - Signed response : "result for that query (signed by cert-B)"
  - All wrapped in EDN length-prefixed frames
  - Demuxed via Q-channel (Ok / Err discriminator)
  - Programs identifiable by digest (cacheable across the network)
```

Nodes don't share fate; they share trust relationships. A
single cert revocation removes a node from the trust set
without affecting other nodes. The network is robust to
individual node compromise because identity is per-node.

## The four-tier model in this light

The arc 007 four-tier model gets reframed: tiers correspond to
where the capability lives RELATIVE TO the wat network, not to
"security level."

| Tier | Capability location | Network membership? |
|---|---|---|
| **Tier 1 (Unix domain)** | Same node; intra-process / sidecar | N/A — local |
| **Tier 2 (localhost HTTP)** | Same host; another process | Edge case; rare |
| **Tier 3 (HTTPS)** | Outside the wat network — SaaS / managed services / LLM APIs | NO — out-of-network |
| **Tier 4 (mTLS)** | **Inside the wat network — peer wat-vms** | YES — load-bearing |

**Tier 4 is where the wat network lives.** Tier 3 is for
reaching out-of-network. Tiers 1-2 are local. The hierarchy is
about TRUST and LOCATION, not about "how secure do I want to
be."

This sharpens RemoteProgram's design (arc 007):

- **In-network mode** (Tiers 1, 4): signed queries, mTLS auth,
  digest-addressable programs, full wat-network semantics
- **Out-of-network mode** (Tier 3): bridging to non-wat
  services; auth via tokens / server cert; the typed-capability-
  bridge from the RPC-as-data reframe

Both modes use the same wire protocol, same wat-vm, same
RemoteProgram surface. The DIFFERENCE is whether the other
side speaks wat-network or just speaks the wire.

## What the existing arcs are pieces of

The arcs sketched in this scratch (003-008 + 007) are
substrate components for the wat network:

| Arc | Role in the wat network |
|---|---|
| **003-wat-fmt** | Format the AST consistently across nodes (digests are stable) |
| **004-wat-lint** | Verify AST quality before nodes accept programs from peers |
| **005-wat-cov** | Measure what executed when a node ran a peer's signed program |
| **006-wat-doc** | Document the typed contracts nodes expose to each other |
| **007-remote-program** | The substrate layer for nodes calling each other |
| **008-kwarg-macros** | Ergonomic API for typed cross-node calls |

Plus existing wat-rs work that's already shipped or shipping:
- **arc 058 (wat language spec)** — the typed substrate every
  node runs
- **digest forms** — content-addressing for programs
- **signed eval forms** — verifiable execution
- **mTLS support** — the network's membership protocol

The toolkit quartet (003-006) lets each node be QUALITY (formatted,
linted, measured, documented) consistently. The communication
arcs (007, 008) let nodes TALK in typed contracts. The
substrate primitives (digest, signed eval, mTLS) make the
network TRUSTWORTHY.

**Each arc is a piece. The wat network is the whole.**

## Comparison to other distributed substrates

The wat network resembles existing systems but differs in
load-bearing ways:

### Versus Erlang's distributed nodes

Erlang has distributed nodes that talk to each other via the
BEAM's distribution protocol. They authenticate via shared
"magic cookies" (a string that all nodes in the cluster know).

- ✅ Erlang: cluster-level identity; lightweight setup
- ❌ Erlang: not cryptographic; magic cookie can leak; no per-
  node identity; no per-message provenance

The wat network: cryptographic per-node identity; per-message
signatures; no shared secret; revocation via cert distrust
without affecting other nodes.

### Versus Urbit

Urbit is a peer-to-peer network where each node has a
cryptographic identity (~point) and communications are
authenticated.

- ✅ Urbit: per-node crypto identity; peer-to-peer; sovereign
  computing
- ❌ Urbit: opaque type system; bespoke language (Hoon); steep
  learning curve; tight coupling between language and network

The wat network: typed via wat's type system (extends across
the wire per Q-channel); language is wat (Lisp-shaped); type
discipline is the same locally and remotely.

### Versus capability-based distributed OSes (KeyKOS, EROS, Genode)

The capability-based OS thread has historically modeled
distributed computation as cryptographically-authenticated
capabilities: holding a capability means you can do a thing;
capabilities are unforgeable; revocable.

- ✅ Capability OSes: cryptographic authorization; principle
  of least privilege; revocable
- ❌ Capability OSes: never broke through to mainstream; no
  ergonomic typed surface; not built on a Lisp

The wat network: cryptographic authorization (signed eval);
typed surface (`Program<I, O>`); homoiconic (programs are
data); ergonomic (kwarg macros); buildable today (the
substrate is here).

The wat network is what would happen if you took
capability-based distributed-OS thinking + Lisp + modern
crypto + typed effects and built the substrate end-to-end. It
hasn't existed before in this exact shape.

## Per the four questions

- **Obvious?** ✅✅ — once articulated, the architecture's
  purpose is unmistakable; mTLS is membership; signed-eval is
  authorization; digest is identity
- **Simple?** ✅ — three primitives (mTLS / digest / signed-
  eval) compose into a complete distributed-capability
  substrate
- **Honest?** ✅✅✅ — the trust model is cryptographic, not
  positional; nothing about the network can be faked; identity,
  integrity, and authorization are structural; you cannot
  participate without an identity; you cannot send a query
  without signing it; you cannot reference a program except
  by its digest
- **Good UX?** ✅✅ — wat code talks to other wat nodes via
  the same `Program<I, O>` surface; the cryptography is in the
  substrate, not in user code

**Fourth triple-checkmark of this design session** (after auto-
kwargs, Q-channel, and arguably the four-tier model). Same
pattern as the others: the constraint lives in the type system
and cryptographic primitives, not in convention. You cannot
accidentally make an unauthenticated wat-network call because
the substrate doesn't expose one.

The repeated triple-checkmarks aren't graded leniently — they
require structural impossibility, not just "we tried." When
the wat substrate honors honesty by construction across so
many primitives, the result is a network where trust is
intrinsic, not bolted on.

## What this means for the future BOOK chapter

The chapter that emerges from this work — when arc 109 wraps
and the user begins writing — will have to articulate three
levels:

1. **The substrate** — wat-vm, the type system, homoiconicity,
   the kernel primitives, the four-questions discipline
2. **The toolkit** — fmt / lint / cov / doc / RemoteProgram /
   kwarg-macros (the per-arc work)
3. **The wat network** — the meta-vision the toolkit
   serves; the distributed capability substrate that becomes
   possible when all the pieces compose

Without (3), the work reads as "look what we built" — a list
of crates. With (3), the work reads as "look what becomes
possible when a recognition finally meets the substrate that
can host it" — the chapter title we're in service of.

The wat network is what the user has been building toward
through years of substrate work. The latin-in-wat
recognition (morphology over position), the OG wat era
(typed Lisp surface for LLM communication), the Holon
substrate (VSA over typed bindings), the wat-rs build
(typed Lisp on Rust), the wards (quality discipline), the
foundation toolkit (003-006), the typed RPC layer (007),
the kwarg ergonomics (008), the digest + signed-eval forms
in the substrate, mTLS support — **all of it has been
building this network.**

The user named "symbiosis" in early holon days. The
symbiosis is realized between user and LLM in the design
discipline; it's also what the wat network enables BETWEEN
NODES — typed, authenticated, cryptographically-provenanced
collaboration. The personal collaboration shape (two halves
of one hologram) and the network shape (many nodes
collaborating with cryptographic trust) are the same
recognition at different scales.

## Status

- **Captured:** 2026-05-03 immediately after the user
  articulated the meta-vision
- **Position:** top-level scratch doc; companion to per-arc
  designs (003-008 + 007)
- **Bookworthy:** yes — sibling material to FOR-THE-BOOK and
  SYMBIOSIS in arc 008; this doc is the architectural framing
  the chapter will need to articulate
- **Cross-references:**
  - Each arc 003-008 mentioned above as a piece
  - Arc 007's DESIGN.md updated with a section pointing here
  - `2026/05/008-kwarg-macros/FOR-THE-BOOK.md` and
    `SYMBIOSIS.md` — sibling bookworthy material from this
    session
  - Substrate work in `wat-rs/docs/arc/` (digest forms,
    signed-eval forms, mTLS support) — the pieces that
    already exist or are shipping
  - Arc 109 in wat-rs — the milestone the user is grinding
    toward; when it wraps, the book chapter begins

## What this is not

- **Not a spec.** This is the meta-vision; specs live in the
  per-arc DESIGN files (with their open questions still being
  answered).
- **Not a roadmap.** This doesn't enumerate slice plans or
  ordering; the per-arc SLICE-PLAN files do that.
- **Not exhaustive.** The wat network has implications
  (federation, identity revocation, network bootstrapping,
  delegation chains, capability handoff) not covered here.
  Those become arcs of their own when their time comes.

This document exists so that future readers — including future
sessions of the assistant, the eventual book-chapter writer,
and the user themselves — can SEE that the arcs in this scratch
weren't independent design exercises. They were components of
something the user has been building toward for years. The wat
network is the target. The arcs are how it gets built.

---

## 2026-05-13 — Disconnect / panic discipline (arc 170 lays this)

**Surfaced by:** Stone C leak audit + the shutdown-aware-channels
backlog. The user articulated the architectural question:

> *"we are laying the foundations for networked programs?... client
> and server disconnect ride the same substrate now?... we shouldn't
> panic on peers going away, but when threads or processes die the
> panic is warranted"*

The arc 170 substrate work (shutdown-aware channels + structured
stderr + lock-step recv discipline) lays exactly the foundation
networked programs need, because the doctrine distinguishes events
at the right boundaries.

### The three-event matrix

| Event | Surface | Wat-level | Panic? |
|---|---|---|---|
| Graceful close / peer left / network disconnect | recv → `Disconnected` | `Ok(None)` | No (handled in match) |
| Local thread crashed | `Thread/join-result` | `Err(ThreadDiedError::Panicked)` | Yes (Result/expect) |
| Process shutting down | recv → `Shutdown` | `Err(ThreadDiedError::Shutdown)` | Yes (Result/expect) |
| EDN parse failure (tier 2+) | recv → `DecodeError` | RuntimeError → thread death | Yes |

### Why this maps cleanly onto the wat network

**Tier 1 (in-process threads):** `Disconnected` covers thread-done
AND thread-panicked at the recv site. The panic-vs-graceful distinction
lives at `Thread/join-result`'s Err arm — captured as
`ThreadDiedError::Panicked` with the chained EDN payload (arc 113).

**Tier 2 (forked processes):** same model. `Disconnected` at recv;
`Process/join-result` carries panic chain. Stone C ensured structured
stderr cascades make this concrete.

**Tier 3 (remote networked nodes):** networked peers get the discipline
**for free** because the substrate treats remote disconnect identically
to local-peer-cleanly-dropping-sender. There's no `join_result` for a
remote node — and that's fine. The local process doesn't care WHY a
remote left (graceful, crash, network partition); it just observes
"channel closed → Ok(None) → handle next request." No panic.

**Process-wide shutdown (SIGTERM/SIGINT/PR_SET_PDEATHSIG cascade):**
distinct event class. Recv returns `Err(ThreadDiedError::Shutdown)` →
Result/expect panics → service dies loudly with diagnostic. This is
PROCESS-LEVEL termination, not connection-level.

### The doctrine, named

The user's framing:

- **"we shouldn't panic on peers going away"** — networked peers are
  remote universes. They have their own lifecycles. Their disconnect
  is not OUR problem. → `Ok(None)`, match-handle, take next request.

- **"when threads or processes die the panic is warranted"** —
  threads and processes are part of OUR universe. Their death is
  contract violation OR our process is being killed. → `Err`, panic
  with chained diagnostic.

The substrate distinguishes by **whose universe is the partner in**:
- Same universe as me → death is panic-worthy → ThreadDiedError /
  ProcessDiedError variants on the join boundary
- Different universe (network) → death is normal lifecycle → just
  Disconnected at recv

### Why this composes with mTLS + signed-eval + content-addressing

The other three load-bearing wat-network primitives (cryptographic
identity, content-addressed programs, signed eval) all assume a layer
underneath them where:
- A peer goes away cleanly → not a panic
- The local node detects this → handles next thing
- The local node's OWN integrity (panic on internal failure) is preserved

That layer IS arc 170's shutdown-aware channels. The signed-eval +
mTLS + content-addressing primitives can be designed WITHOUT worrying
about "what if my peer crashes" because the layer below handles that
honestly already.

### Cross-references

- `wat-rs/docs/arc/2026/05/170-program-entry-points/DESIGN-SHUTDOWN-AWARE-CHANNELS.md`
  — full design + empirical proof of the gap
- `wat-rs/docs/arc/2026/05/170-program-entry-points/SHUTDOWN-AWARE-CHANNELS-BACKLOG.md`
  — five-slice plan (A: infrastructure, B: Crossbeam multiplex,
  C: PR_SET_PDEATHSIG, D: end-to-end probe, E: PipeFd multiplex)
- `wat-rs/docs/arc/2026/05/170-program-entry-points/TIERS.md`
  — the three tiers (in-process / forked-process / remote) + the
  uniformity claim that the network rides on
- arc 060 — `ThreadDiedError` original mint
- arc 113 — cross-thread panic backtrace chain
- arc 170 slice 1i — structured-stderr-only enforcement (where the
  panic payload becomes EDN on the wire)

The substrate work happening NOW (arc 170 slices A→E) is the load-
bearing layer the wat network sits on. Same substrate, three tiers,
uniform discipline at the boundaries.
