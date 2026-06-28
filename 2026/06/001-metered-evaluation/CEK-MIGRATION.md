# CEK Migration — scope, surface, and sequencing (home first, then CEK)

Companion to `DESIGN.md`. That doc is the *idea* (metered/pausable compute) and
resolves *that* the evaluator is a recursive tree-walker + tail-call trampoline.
This doc is the *engineering plan* for the third tier — the actual move to a CEK
machine — and the order it should happen in relative to the namespace refactor.

User framing (2026-06-12):

> *"i was planning on gutting the src/*.rs files into
> src/<namespace>/<bunch-of-scoped-files>.rs … should i do the large refactor
> to get things well homed first then go to CEK, or do CEK then mass refactor
> to better maintainable names?"*

**Answer: home first, then CEK. Strongly.** The reasoning, grounded, below.

Status: **plan capture — to mull.** The trigger is "after the syntax flip
(arc 251) settles the form set."

---

## How awful is the CEK move? Major, but bounded.

Grounded against `wat-rs/src/runtime.rs` (not guessed):

| Metric | Value |
|---|---|
| `runtime.rs` total lines | **30,426** (flat monolith) |
| `eval_*` handler fns | **289** |
| Recursive eval call sites | **~355** |
| `dispatch_keyword_head` form arms | hundreds (central dispatch) |
| `*_tail` family (partial-CEK seed) | 4 |

Every one of those ~355 recursion points is a spot where "recurse into the
subform" must become "**push a K-frame, set C to the subform, loop.**" That's a
from-the-studs rebuild of the evaluation core — top-tier-biggest-arc territory.

**But two of the three CEK registers already exist as data:**
- **C** = `WatAST` (already data). **E** = `Environment`
  (`src/value/environment.rs:90` — already a reified struct). **Only K is new.**
  You invent one register and thread it; you don't rebuild the state model.

**The blast radius is contained. CEK does NOT touch:**
- **The macro engine** (arc 249) — expansion is a separate, *earlier* phase;
  eval runs on already-expanded forms.
- **`:rust::` / native builtins** — *leaves*: they receive already-evaluated
  `Value`s and return a `Value`; they never recurse into eval. The whole
  Rust-interop surface is invisible to CEK. (This is why the ~355 recursion
  sites are *wat special forms*, not the builtin library.)
- **The kernel / threading model** — per-eval-context CEK; each thread runs its
  own step loop; a blocking `recv` stays an OS-thread block. No scheduler
  rewrite forced (cooperative-yield-on-block is a *later* option K enables, not
  a prerequisite).
- **Parser, type checker (`src/check`), stdlib** — untouched.

**The real complexity metric isn't 355 — it's the number of distinct K-frame
*shapes*: ~15–25.** eval-args-then-apply (one shape, reused for every call +
every holon op), if (cond→branch), let/do (sequence), match (scrutinee→arms),
and/or (short-circuit), quasiquote (eval-unquotes), the constructors
(fields→build), try (install handler frame). The 355 sites collapse into a
couple dozen frame variants — enumerable and bounded, which is what makes it
tractable rather than open-ended.

## Where it needs to be expressed (the surface area)

- A new **`K` type** (the continuation-frame enum) — for v1 a Rust enum; for the
  *dream*, K is `WatAST` / EDN so the continuation is content-addressable,
  signable, portable across hosts.
- A **`step(C, E, K) → Transition`** loop replacing `eval_inner`
  (`runtime.rs:3279`); each `eval_*` special-form handler re-expressed as "which
  frames this form pushes."
- `dispatch_keyword_head` → the dispatch *inside* the step loop.
- **`apply_function`'s trampoline (`runtime.rs:17335`) merges into the step
  loop** — a tail call becomes "replace C, don't push." The existing `*_tail`
  family + `EvalSignal::TailCall` is the seed: **the tail-position analysis CEK
  needs is already done.**
- `EvalBreak` / `EvalSignal` → `OutOfFuel` / `Yield` become "halt the step loop,
  return `(C, E, K)`."
- `try` → a handler frame in K (cleaner than the current Rust-stack unwind).

## The sequencing decision — home first, then CEK

The two changes live on different axes:
- **Homing** = *where* code lives (modules, names). Mechanical,
  behavior-preserving, low-risk, **wardable incrementally**.
- **CEK** = *how* eval works (control-flow semantics). Semantic,
  behavior-changing, **highest-risk rewrite in the language's history.**

The doctrine: **never compound the hardest semantic change with the worst code
organization.** CEK-first means executing a from-the-studs control-flow rewrite
*while navigating a 30k-line flat file*, finding all 355 sites by grep, holding
the dispatch in your head, with no warded seams to catch drift. The hardest
thing in the worst place. Home-first does the safe restructure to *create the
clean surface*, then lands the risky rewrite against it.

### Why "but I'd be homing code I'm about to rewrite" dissolves

CEK rewrites only the **control-flow core** — the ~15–25 K-frame shapes. Call it
a few thousand lines. The *other* ~25k lines of `runtime.rs` are **leaves**
(builtin impls, holon ops, value operations, type-interplay helpers) — things
CEK *calls* but never restructures. So:

- Homing moves **all 30k** lines into homes; CEK touches the **bodies** of maybe
  10% of it.
- Even for that 10%, the **home/file placement survives** CEK — only the
  function *bodies* change shape (recursive → frame-producer). You re-home
  nothing; you rewrite bodies in place.

Home-first "wastes" almost nothing. CEK-first wastes the entire homing context
*during the hardest rewrite*. The asymmetry is the whole decision.

### The refinement — home *with CEK in mind*

When you gut the eval core, don't scatter it — carve it into a structure that
anticipates the step machine, so CEK becomes a *localized arc*:

```
src/eval/
  dispatch.rs   ← dispatch_keyword_head (which frame for which head)
  forms/        ← if / let / do / match / and / or / quasiquote / try
                  — each becomes a frame producer
  apply.rs      ← apply_function's trampoline → becomes the step loop
  value.rs      ← the leaves CEK calls but never restructures
  cont.rs       ← (empty for now) the K type lands here when CEK starts
```

Then CEK is "rewrite `forms/` + `apply.rs`, add `cont.rs`" — against warded,
navigable, settled modules, with the 25k lines of leaves already home and
untouched.

### Full sequence

1. **Finish the syntax flip (arc 251)** — settles the form set, so you don't
   home/rewrite forms you're about to delete, and `dispatch_keyword_head`
   reaches its final shape.
2. **Gut the flat `src/*.rs` into homes** — warded as you go (`vigilatum` per
   home); eval carved into `src/eval/` as above.
3. **CEK** — against the clean, settled, navigable surface.

### The four questions, all pointing one way

- **Obvious?** ✅ safe-before-risky; clean surface before hard rewrite.
- **Simple?** ✅ keeps mechanical (homing) and semantic (CEK) *separate* instead
  of entangled.
- **Honest?** ✅✅ home-first keeps the per-home warding that catches drift;
  CEK-in-the-monolith hides it.
- **Good UX (future-you)?** ✅ CEK lands in a codebase you can actually read.

### Honest caveat

A few `forms/` files *will* get their bodies churned by CEK after you home them
— real, but small (the ~15–25 shapes), and the home survives. That churn is
vastly cheaper than doing the rewrite blind in the monolith.

## What CEK buys beyond billing

Worth remembering when weighing the arc — it's not just for metered-eval's dream
tier:
- **Cleaner error handling** — `try` as an explicit handler frame in K, not a
  Rust-stack unwind.
- **Cooperative scheduling** — a blocking op can *yield* the continuation
  instead of blocking an OS thread (optional, later).
- **Time-travel debugging** — a reified `(C, E, K)` is a snapshot you can step
  *backward* through.
- **Content-addressed, portable continuations** — the metered-eval dream:
  pause anywhere, hash/sign/send the continuation, resume on another host.

## Beyond the interpreter — bytecode, native, and keeping the serializable continuation

> Captured **2026-06-28** from a live co-design pass (mine, with the builder).
> **Extends, does not supersede,** the CEK-*interpreter* plan above. The plan above
> lands tier 3 (CEK, K reified as data). This section answers the next question:
> once CEK is in, can we go *fast* — bytecode, then a native JIT — **without losing
> serialize / hibernate / migrate**? Yes. The contract is the constraint set below.

The folk worry is "interpreted → slow," but speed is set by where the cycles go
(Amdahl), and wat's interpreter is native Rust over native primitives — real
workloads are primitive-dominated, so the walk is noise (the stress tests already
say so). "Interpreted" names *how* it runs, not *how fast*. So the tiers below are
about **capability headroom**, not rescuing a slow base.

### The tier ladder — where serializability is free, and where it bites

| Tier | Execution | Serializable `(C,E,K)`? |
|---|---|---|
| tree-walk (today) | recurse over `WatAST` on the Rust call stack | **no** — K *is* the Rust stack |
| **CEK interpreter** (this doc) | step over `WatAST`, K reified as data/EDN | **yes — free** (K is data) |
| **bytecode VM** | step a flat instruction stream; state = `(ip, E, frame-stack)` | **yes — ~free** (the VM state *is* the image) |
| **native JIT** | compile hot bytecode → machine code | **not free** — state is registers + code pointers; needs the constraint set |

The load-bearing observation: **serializability is free at the CEK and bytecode
tiers** — the machine state already *is* data. It only "goes hard" at the **native**
tier. Bytecode is the natural next step after CEK (defunctionalize K → linearize the
control into an instruction stream); it is *faster than the CEK interpreter AND keeps
the portable image for free*. Native is the optional fourth tier where the contract
applies.

### The principle — a latent hologram, not a maintained one

Portable `(C,E,K)` is the **source of truth**; bytecode and native are **caches that
must be able to bail back to it.** (Helland, this repo's `recolligere` epigraph: *the
truth is the log; the database is a cache of a subset of the log.*) You **never
serialize the cache** — you serialize the *reconstructed* truth.

So you do **not** maintain two synced copies (that doubles memory and pins the
optimizer — anything the EDN-shadow could observe, the compiler can't elide). You
maintain a cheap invariant that the portable form is **reconstructible on demand at
safe points** — *knowable, not held*. The hologram is **latent**: it condenses out of
compile-time metadata exactly when you look (snapshot / migrate / inspect / deopt),
and is otherwise just the fast form running free. Exact analogy: a GC doesn't hold a
live root-list — it keeps **stack maps** to *find* roots at a safe point. Same
structure, one level up.

### The constraint set (native tier — to lose no CEK benefit)

1. **Safe-point reconstructibility.** At every safe point, the logical `(C,E,K)` is
   rebuildable from running state in bounded work, via metadata fixed at compile time
   (stack maps / deopt info — the HotSpot/V8 deopt + Go stack-copy machinery).
   *Granularity is a lever:* coarse safe points (call boundaries + loop back-edges)
   keep native fast and still hibernate at every semantically meaningful point — you
   almost never need mid-arithmetic capture. **The load-bearing one.**
2. **Portable code identity.** Nothing capturable in a K may hold a raw machine
   address — only a stable content-hash / symbol; resume maps hash → (load or re-JIT).
   *Already the dream* (this doc, "K is content-addressable, signable, portable") —
   here it is **promoted to a hard machine invariant.** ✅ structurally in reach.
3. **Serializable data closure.** No value live across a safe point may be non-EDN
   without a declared re-materialization recipe. *Already wat doctrine*
   (wat-record-for-EDN; `Value::Struct` only for non-EDN payloads) — **promoted from a
   style rule to a machine invariant.** ✅ structurally in reach.
4. **Resource re-acquisition by name.** Live external resources crossing a safe point
   — fds, peers, locks, io_uring SQEs — must be portably *named* and re-openable, or
   confined to between safe points. This is where migration meets `mora` / the
   three-loci peer interface: a computation hibernated **mid-IPC** resumes by
   re-establishing the channel *by name*, not by shipping a dead fd. **The deepest
   one** — it is the constraint that turns "ship the running computation to the remote
   locus" from a segfault into a defined operation, and it is the highest-value.
5. **Refinement equivalence.** Native must faithfully realize the abstract machine —
   the deopt mapping is a sound simulation (`native ⊑ CEK`), so the two faces of the
   hologram always agree. A **provable** obligation — the operational-equivalence shape
   from the LEAN-parity thread (`../003-verified-eval/THE-LEAN-PARITY-STONES.md`).

### Status, and the asymmetry worth noticing

- **2 and 3 are structurally already true** (content-addressing; EDN-by-default).
- **1 and 5 are the engineering + proof mountain** (stack maps / deopt; the simulation
  proof).
- **4 is the deepest and the highest-value.** The set {native speed} ∩ {a continuation
  you can serialize and *ship across machines*} is genuinely sparse: the JVM has native
  + an image-ish heap but no portable stack migration; Smalltalk has the migratable
  image but not native; SML/NJ has native CPS continuations but no portable
  serialization — **each has two of the three.** Content-addressed wat is positioned to
  have **all three**, and constraint 4 is the hinge.

### The four questions

- **Obvious?** ✅ truth-vs-cache; serialize the *reconstructed logical form*, never the
  native frames.
- **Simple?** ✅ the hologram is **latent** (one source of truth + metadata), not two
  synced copies.
- **Honest?** ✅ names which constraints are already met (2, 3), which are the mountain
  (1, 5), and which is the deep one (4) — no hand-waving over the resource edge.
- **Good UX (future-you)?** ✅ coarse safe points = fast native *and* hibernate where it
  matters; you never pay a fine-grained-safepoint tax for a granularity you don't need.

### Honest caveat / sequencing

Strictly **after** the CEK interpreter (tier 3 above). Don't reach past bytecode for the
native JIT until a real workload is *glue-bound* — today everything is
native-primitive-dominated, so the interpreter tax is already noise, and native codegen
buys nothing until the wat-level fraction grows. Bytecode (tier-3.5) is the move that is
**both** faster than the CEK interpreter **and** keeps the serializable image free; it is
almost certainly the sweet spot, with native held as a constraint-gated option.

### The deeper unlock — `wat-vm` becomes a *specification*, not a binary

> Builder's framing (2026-06-28): *building CEK means we get to declare `wat-vm` has
> intrinsic meaning — not just a "good enough" ref to whatever wat does right now.* True,
> and it's the keystone the other two pursuits also rest on. (His insight; the spec-vs-impl
> precision + the V8 contrast below are the apparatus's synthesis — marked, not blended.)

Today wat's meaning is **implementation-defined**: "what does this program mean?" has one
answer — *whatever `runtime.rs` computes*. The semantics live in 30k lines of Rust; the
definition is circular (wat is what the evaluator does). That's a **reference
implementation**, not a specification — you cannot even *have* two conformant wat
implementations, because there is nothing to conform to but the one binary. (CPython/MRI
sat here for years: "Python is what CPython does.")

CEK turns it **machine-defined**. The abstract machine — a finite transition system, the
~15–25 frame shapes — *is* the specification. `wat-vm` is now the intrinsic referent; the
tree-walker, the bytecode VM, the native JIT are all **implementations**, each correct iff
it refines the machine (`impl ⊑ wat-vm` — constraint 5). Many conformant implementations
become possible. That moves wat into the class of languages defined by an abstract machine
rather than a binary — *The Definition of Standard ML*, Scheme's formal semantics, and most
on-the-nose **WebAssembly** (a stack-machine bytecode whose spec *is* the machine).

This is the same **truth-vs-cache** relation as the constraint set, one level up: the
abstract machine is the *truth*; every executor — tree-walk, bytecode, native — is a *cache*
that must refine it. Semantic conformance and serialization-correctness are not two problems;
they are the one `⊑ wat-vm` obligation seen twice.

And it is the **precondition for migration, not a nicety**: a continuation is portable across
hosts *only because* there is a shared, defined machine for it to be a state *of*. Without an
intrinsic `wat-vm`, a hibernated continuation is "some Rust state" meaningful only to this
binary on this build — shipping it is nonsense. With it, the continuation is "a state of the
`wat-vm`," and any conformant host can finish it. So the three ambitions — **verify**
(LEAN-parity), **accelerate** (the tier ladder), **trust** (signing, below) — are not three
goals; they are three faces of one keystone: the intrinsic, signed, EDN `wat-vm`.

### Signing the machine (arc 295) — the continuation is a *signed* EDN value

Arc 295 (`signed-code-only`) is the trust leg, and it makes "sign the semantics" concrete
rather than aspirational. The doctrine (builder, verbatim — `wat-rs/docs/arc/2026/06/295-signed-code-only/REALIZATIONS.md`):
*"you may only use signed code … there is no option. period. you sign your code. you may only
sign your code."* Two grounded facts from `295/DESIGN.md` make it click for CEK:

- **The signature target is the canonical-EDN hash of the *AST*** (`src/load.rs:60`), not raw
  bytes — "the AST is what's signed," so a signature survives comment/whitespace edits. And
  *"wat is edn"* (his): manifest, chain, keys — all EDN, no JSON, no blobs, no runtime-KMS dep.
- **Eval-side doctrine:** *"eval must be signed, mandatory, parity with load."*

Now the keystone lands: since CEK reifies `(C,E,K)` as canonical EDN (this doc's K-as-`WatAST`/EDN
dream), **signing a continuation IS signing code** — same canonical-EDN hash, same ed25519 /
`:sig` support, same multi-key manifest + chain. No new mechanism. And eval-must-be-signed means a
**migrated continuation cannot resume unless signed-and-verified** — it rides the exact signed-EDN
eval path a loaded file does, checked before it runs.

The threat model is the same one, too. Arc 295's why (The Tempest, his song): *"there's no fork,
there's no horns"* — malicious code wears no mark; you cannot find the devil by looking, so the only
proof against an unmarked adversary is the seal. A continuation arriving from the **remote locus**
wears no horns either: you cannot tell a legitimate resume-state from a hostile one by inspecting it,
so the signature is the only proof it is safe to resume. *Signed-code-only and continuation-migration
are the same trust problem.*

So "ship the running computation to another locus" needs **three preconditions, all riding one EDN
substrate**:

| precondition | what it gives the continuation | source |
|---|---|---|
| **CEK** | *serializable* — `(C,E,K)` reified as EDN data | this doc, tier 3 |
| **intrinsic `wat-vm`** | *meaningful* across hosts — a state of the shared machine | the spec above |
| **arc 295 signed-code-only** | *trusted* across hosts — signed canonical-EDN, verified before eval | `295/DESIGN.md` |

A migrated continuation is serializable (CEK) **+** meaningful (the spec) **+** trusted (the 295
seal). Arc 295 is the trust leg, **already designed and grounded** on `load.rs` + datamancy's shipped
model — so the trust layer for migration is not future research; it is a scoped arc whose seal target
(the canonical-EDN AST) is exactly what a reified continuation already is.

## Cross-references

- `DESIGN.md` (this dir) — the metered-eval idea + the three-tier resolution;
  CEK is the third (dream) tier. v1 does **not** need this arc.
- `wat-rs/src/runtime.rs` — `eval_inner` (`:3279`), `apply_function` (`:17335`),
  `EvalBreak` / `EvalSignal::TailCall` (the escape channel CEK extends).
- `wat-rs/src/value/environment.rs:90` — `Environment`, the E register (already
  reified).
- arc 251 (the syntax flip + the great migration / home-lifting) — the
  prerequisite that settles the form set and pays down the monolith.
- `../003-verified-eval/THE-LEAN-PARITY-STONES.md` — the operational-equivalence /
  refinement obligation behind constraint 5 (`native ⊑ CEK`): proving the fast tier
  faithfully realizes the abstract machine is the same shape as the LEAN-parity
  "checks programs → checks proofs" build log.
- `wat-rs/docs/arc/2026/06/295-signed-code-only/{DESIGN,REALIZATIONS}.md` — the trust
  leg: signed-code-only, the signature over the **canonical-EDN AST** (`load.rs:60`),
  eval-must-be-signed (parity with load), the EDN multi-key manifest + timestamped chain.
  The mechanism that makes a reified `(C,E,K)` a *signed* value for free — same canonical-EDN
  hash as a loaded file. (`project_signed_code_only_doctrine` memory.)
