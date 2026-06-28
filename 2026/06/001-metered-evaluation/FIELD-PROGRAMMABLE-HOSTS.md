# Field-Programmable Hosts — wat as the attested deployment substrate (the k8s-trust answer)

> **Provenance (2026-06-28, co-design).** The **thesis is the builder's** — field-programmable hosts,
> wat-network, signed-eval-into-a-TEE, "provisioning is signed eval into a trusted host," and the two
> scope calls (user-code-correctness is out of scope; the provisioning plane is low-worry). The **honest
> hard edges, the prior-art placement, and the "verified guts are the product" synthesis are the
> apparatus's analysis**, weighed — marked, not blended (per arc 295 R2, VENTRILOQUISM). **Status:
> thesis capture — vision-level, not modeled-to-build.** The deployment-layer bookend to
> `CEK-MIGRATION.md`'s internals roadmap.

---

## The thesis (the builder's)

A host is booted minimal and generic. You **stream a signed wat program into it**; a **TEE-attested
wat-vm daemon with a measured/signed launch** verifies it and *runs it precisely as written* — and the
host **becomes** that service: *"oh, I'm a db now, let me do that."* **Provisioning = signed eval into a
trusted host.** No image, no shared-kernel container, no trust-by-registry — a signed, semantically
meaningful program evaluated by an attested runtime.

This is a **direct answer to supply-chain attacks.** Signed-code-only at the *eval* boundary (arc 295)
means tampered or unauthorized code **cannot run at all** — the *"there's no fork, there's no horns"*
threat model (an adversary that wears no mark; the seal is the only proof) answered structurally, not by
convention.

And it is **native** (the CEK → bytecode → JIT ladder), so the assurance does not cost an interpreter tax
on the hot path.

## The trust chain — why it beats the container norm

| Layer | wat field-programmable host | k8s / container norm |
|---|---|---|
| Hardware | **TEE remote attestation** — verifier confirms genuine enclave on genuine HW | host trusted operationally |
| Runtime | **measured/signed launch** — attests *the real wat-vm* is what's running | container runtime trusted by convention |
| Workload | **only signed canonical-EDN evals** — unsigned cannot run | image trusted by *registry* (+ optional bolt-on signing) |
| Isolation | per-program in one verified VM | shared kernel (namespaces/cgroups; escape is a real class) |
| Deploy artifact | a **signed, attestable, semantically-meaningful program** | an opaque tarball of layers |

The chain is **attested hardware → attested runtime → signed workload**, end to end. In the container
world the path from *what I deployed* to *what is actually executing* is trust-by-convention with signing
bolted on; here it is a verifiable chain whose every link is measured or signed.

## Scope — what wat guarantees vs what it does NOT (the builder's boundary)

**IN scope — wat's job (substrate guarantees):**
- **Provenance** — signed-code-only; the eval boundary rejects anything unsigned/tampered (arc 295).
- **Type safety + the language's correctness tooling** — the type checker, the test surface.
- **The runtime's *own* correctness** — `native ⊑ wat-vm` (the load-bearing one; see hard edges).

**OUT of scope — the user's job (application correctness):**
- **Whether the user's code is *good*.** Signing proves *who* + *untampered*; type-checking proves
  *well-typed*; **neither proves *correct*.** *"just because it type checks doesn't mean it's good — users
  must test their code."* (builder) wat ships the tools (types, tests); it does **not**, and will not,
  vouch for application logic. **This is the right division, not a gap:** substrate guarantees provenance
  and a faithful runtime; the application owns its own correctness. *"not my problem."*

## The honest hard edges (these stay IN wat's scope)

1. **The TEE is the trust root, and TEEs are an active battlefield.** SGX has fallen repeatedly
   (Foreshadow/L1TF, Plundervolt fault-injection, ÆPIC, micro-architectural side channels); SEV-SNP / TDX
   are newer but the arms race is live. So this is a **far better root, not an unbreakable one** — the
   attestation is worth exactly the TEE's integrity, which is a moving target. (Builder's domain; named,
   not hand-waved.)
2. **The wat-vm is the *entire* TCB inside the enclave — and it is a JIT.** Once "the real wat-vm is
   running" and "only signed code evals" are attested, the residual attack surface is the TEE **plus the
   wat-vm's own bugs** — JIT miscompilation, a memory-safety slip, a co-tenant sandbox escape. Therefore
   **the wat-vm's correctness *is* the security boundary.** The attestation proves the runtime is
   *running*; it proves nothing about the runtime being *right*. This is why **verification
   (`native ⊑ wat-vm`, the LEAN-parity work) is load-bearing here, not optional** — the whole deployment
   chain ultimately cashes out as a proof obligation on the runtime itself.

## The provisioning / orchestration plane (status: known, low-risk, deferred)

The wedge this thesis drives is k8s's **trust + supply-chain** layer — replaced with proof. k8s *also*
does scheduling, service discovery, autoscaling, self-healing reconciliation, storage orchestration,
network policy, the operator ecosystem — and the field-programmable model is a better **trust + provisioning
substrate**, not a drop-in orchestrator; the orchestration plane is separate work. **Builder's assessment
(recorded as his call, not elaborated):** the provisioning service is *trivial to build in wat*, barely
modeled, not built, **must be solved**, but **the lowest-worry of the open problems** — *"one of the most
trivial problems to solve in the wat."* A host-that-becomes-what-it's-told is itself a different scheduling
primitive worth chasing.

## Positioning — the sharper unification of a real frontier

This is not a lone-genius fantasy; it is the **sharper, unified version of where the serious frontier is
already heading**, which is the evidence it is *right*:
- **Confidential Computing** (the CCC; SEV-SNP/TDX; confidential containers) = attested TEEs for workloads.
- **Wasm edge compute** (Wasmtime, Fermyon/Spin, edge platforms) = signed, sandboxed, portable modules —
  and wat's bytecode-VM is **wasm-shaped**.

The industry converges on *TEE-attestation + capability-secure portable modules* from two directions.
wat's **differentiators**: **verified** signed-eval (not just sandboxed — *provably faithful*), the
**serializable continuation**, and **EDN-native everything on one substrate**. The convergence is
validation, not threat: the same "we've written this" as arc 295 — wat-network was built before the enemy
(supply-chain / attestation) was named, and turned out to be the answer.

**The migration angle the norm lacks:** because a continuation is *serializable* (CEK) + *meaningful* (the
`wat-vm` spec) + *trusted* (signed canonical-EDN) — the three pillars from `CEK-MIGRATION.md` — a running
service can **hibernate on one attested host and resume on another**. Not RPC: *computation migration
across a trusted fleet.* Containers and wasm modules don't carry a live, signed continuation; wat can.

## The synthesis — the surface is the unlock; the verified guts are the product

The entire deployment vision terminates in **one link**: *"…and the runtime is provably faithful to its
spec."* Strip that link and the attested chain — beautiful as it is — is anchored to sand: a signed
program, attested onto real hardware, run by a runtime nobody proved correct. So the builder's
guts-obsession (*"attack everything imperfect in the guts"*) is **not a side-quest** from the
k8s-trust-answer — **the verified runtime is what makes the answer *mean* something.** The surface
(signed-eval, the field-programmable host) is the unlock; the **proven guts (`native ⊑ wat-vm`) are the
product.** Verify, accelerate, trust, *deploy* — all four cash out on the same three pillars.

## Cross-references

- `CEK-MIGRATION.md` (this dir) — the internals roadmap (CEK → bytecode → JIT) + the three pillars
  (serializable `(C,E,K)`, signed canonical-EDN, the `wat-vm` spec) this deployment thesis rests on.
- `wat-rs/docs/arc/2026/06/295-signed-code-only/{DESIGN,REALIZATIONS}.md` — the trust leg: signed-code-only,
  the signature over the canonical-EDN AST, eval-must-be-signed.
- `../003-verified-eval/THE-LEAN-PARITY-STONES.md` — the refinement obligation (`native ⊑ wat-vm`) the
  whole chain cashes out as — the link that makes the attestation mean something.
- *(a wat-network / provisioning doc, if/when — the orchestration plane, builder-low-worry.)*
