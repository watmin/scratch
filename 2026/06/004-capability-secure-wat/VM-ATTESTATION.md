# VM Attestation — signing something objectively forge-observable

Captured 2026-06-15 (session close). The trust question the builder has chewed most:
*"how can we sign something that's objectively forge observable?"* — a proof that a host
runs the canonical, untampered wat-VM, where forging *"I'm canonical"* while running a
hacked VM is cryptographically **detectable**, not "trust me." Companion to `NOTES.md`
(confinement / deny-by-absence / the host-could-roll-a-hacked-VM model). Part of the
verifiable-compute-market thesis (still bouncing, mostly uncaptured by intent).

## The crux: you can't attest your own honesty from inside yourself

A host controls its own software. So **no software the host runs can forge-observably sign
"I am honest"** — it can always run the hacked VM *and* run a separate "honest signer" that
claims canonical. Self-attestation is worthless. This is *"you cannot verify yourself from
inside yourself"* (the complementarity law / Gödel / Thompson — `../003-verified-eval/THE-COGNITIVE-GODEL.md`)
in the **security** domain.

**Forge-observability requires a trust anchor the host does NOT control. There are exactly
three:**

| anchor | mechanism | trust root | forge-observable because |
|---|---|---|---|
| **HARDWARE** | TEE remote attestation | chip vendor (fused key) | host can't produce a valid hardware-signed quote for a measurement it isn't running |
| **MATH** | zkVM proof of execution | mathematics (no vendor) | host can't produce a valid proof for a wrong execution |
| **OTHER PARTIES** | redundant re-exec + slashing | honest-majority + stake | other nodes re-run; divergence is caught; the liar is slashed |

Every real attestation scheme is one of these or a blend. Pick by *what you're proving*
(what's-running / right-result / no-snooping) and *what trust root you'll accept*.

## 1. HARDWARE — TEE remote attestation ("is the host on the canonical VM?")

- The CPU **measures** the code+data loaded into an enclave (a hash — SGX's MRENCLAVE etc.).
  A **hardware key** fused into the chip — never extractable, vendor-certified — signs a
  **quote** = (measurement ‖ nonce ‖ enclave-bound data). The verifier checks it against the
  vendor cert chain → (a) genuine hardware, (b) measurement == the expected VM hash.
- **Forge-observable**: the measurement is signed BY THE HARDWARE, not by host software. The
  host cannot forge *"I'm running VM-hash-X"* without the CPU actually loading code-X into a
  genuine enclave.
- **wat synthesis**: the measurement IS the **content hash of the canonical wat-VM (+ its
  extension set)** — i.e. your **272 KMS-signed manifest**. Verifier checks: quote-measurement
  == published-canonical-wat-hash. "What *should* run" (your signing infra) and "what *IS*
  running" (the hardware) **meet at the hash.**
- **Two musts**: **freshness** (a nonce, or the host replays an old quote); **channel binding
  (RA-TLS)** — bind the attestation to the session/TLS key, or a **relay/cuckoo attack** lets a
  genuine enclave attest while a hacked VM does the work over a relayed channel. Same key
  attests and encrypts ⇒ you're provably talking to the attested enclave.
- **Soft spot**: you trust the **chip vendor** (Intel/AMD/AWS) and unbroken silicon. SGX has a
  real **side-channel history** (Foreshadow, Plundervolt, SGAxe, ÆPIC). So TEE attestation is
  forge-observable *relative to the vendor + an intact chip* — strong, not math-absolute.
  (SGX / SEV-SNP / TDX / Nitro / ARM-CCA.)

## 2. MATH — zkVM proof of execution (vendor-free "right result")

- Instead of attesting WHAT runs, prove the RESULT: a succinct **ZK proof** that
  *wat-eval(P) = R*, verifiable without trusting hardware OR re-running.
- **Forge-observable**: you can't produce a valid proof for a wrong execution. Trust root =
  **math**, no vendor.
- **Precondition**: deterministic eval (`../003-verified-eval/`) — you prove a deterministic
  computation. Gives **integrity + input privacy**.
- **Cost**: heavy, dropping fast (RISC Zero, SP1, Jolt). The long-game that removes the TEE
  vendor trust-root.

## 3. OTHER PARTIES — redundancy + slashing (cheap "right result", no special hardware)

- Don't attest; assume honest, **re-run on N nodes** (determinism, `../003`, makes this
  possible), **divergence ⇒ slash** the liar. The Truebit verification-game / optimistic-
  rollup pattern.
- **Forge-*un*profitable** rather than per-job forge-observable: liars are caught on
  challenge/re-run, lying is negative-EV. Trust root = honest-majority of re-runners + stake.
- **Integrity only** — re-running does NOT stop the original host snooping. Cheapest path;
  no TEE.

## Which layer for which property

- **What's running is canonical** → HARDWARE (TEE attestation).
- **Result is right (integrity)** → MATH (zk, strongest) *or* OTHER PARTIES (redundancy+slash,
  cheapest). **Determinism is the precondition for both.**
- **No snooping (confidentiality)** → HARDWARE (TEE enclave the host can't peer into) *or* MATH
  (FHE — compute on ciphertext, gold-standard, brutally slow). The hard tier — and the
  **public-logic-vs-private-data bifurcation** makes it **optional for v1**: logic/premise-
  checking tolerates a curious host as long as integrity holds; only private-data workloads
  need the enclave.

## The wat synthesis (how it locks together)

deny-by-absence (`NOTES.md` — airtight *inside*: a program can't escape) **+** content-address
the VM + extensions (272 — the canonical hash is your signed manifest) **+** TEE attestation
(measures + signs that hash; host can't forge) **+** RA-TLS (binds it to the channel; kills
relay) **+** determinism (003 — backs integrity via redundancy or zk). Attestation meets your
existing signing infra **at the hash**; the market routes to *the cheapest attested image that
has what you need and that you trust*; a hacked VM **can't attest as canonical**, so betrayal
is structurally unprofitable (the anti-botnet shape again).

## Residual to chew

- The **TEE vendor trust-root + side-channels** is the soft spot of the practical path; **zk
  removes it but is heavy.** Near-term pragmatic: **TEE-for-confidentiality +
  determinism-redundancy-for-integrity**, zk as the long game.
- **Heterogeneous hardware** (SGX vs SEV vs none) ⇒ **attestation-capability becomes another
  market axis** — route by "can this host attest the way I need."

## Cross-references
- `NOTES.md` (confinement; the host-could-roll-a-hacked-VM model; the responsibility boundary).
- `../003-verified-eval/` — determinism (the precondition), `THE-COGNITIVE-GODEL.md` (the
  can't-verify-yourself-from-inside spine this *instances*), `THE-CLOJURE-ORACLE.md` (the
  diverse-witness redundancy).
- Coordinates: Intel SGX/TDX, AMD SEV-SNP, AWS Nitro, ARM CCA (TEEs); RA-TLS; RISC Zero / SP1
  / Jolt (zkVM); Truebit (verification game); FHE.
