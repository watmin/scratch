# wat-native eBPF — line-rate packet filtering, the whole stack in wat (aya as oracle, not dependency)

> **Provenance / status (2026-06-28, co-design).** The **thesis and the sharpened target are the
> builder's** — eBPF/XDP authored in wat; *"i do not build an aya layer in wat — we build all of it in
> wat — aya is a reference impl we satisfy."* The **grounding (the lab IS aya today) and the synthesis
> (the trust-boundary argument, the kernel-as-living-field-programmable-host, the typed-verifier-passing
> win, the oracle structure) are the apparatus's**, marked. **Status: thesis capture — vision-level, not
> modeled-to-build.** Rides the compiler foundation in `../001-metered-evaluation/`.

---

## The thesis

**Line-rate packet filtering — the XDP fast path AND the userspace brain — authored entirely in wat.**
The DDoS lab (`holon-lab-ddos`) is where the whole project *began* (the epilogue: *"i want to wrap up my
DDoS ideas. A firewall."*); this is it reborn in the language the firewall made him build. **Not a wat
wrapper over aya/libbpf — the WHOLE toolchain in wat.** Aya is the **reference implementation we satisfy**
(the oracle), never a dependency we bind.

## The current ground — what we replace (grounded 2026-06-28)

The lab **already runs aya**: the XDP program (`veth-lab/filter-ebpf/src/main.rs`) is **Rust compiled to
eBPF bytecode via aya**; the **sidecar** (`veth-lab/sidecar/`) is Rust using aya's userspace side to load
it and talk through maps; the deny path is an **eBPF tail-call tree**, deployed **blue/green** atomically;
the brain is the holon/VSA anomaly engine (holon-rs). So "aya in wat" is not a metaphor — it is the
*literal* bridge: replace aya's two halves (Rust→eBPF codegen, and the Rust userspace loader) with wat.

## The target — ALL of it in wat (the sharpened call)

Not an aya layer. Build the whole stack:

- **The `wat-bpf` dialect + the wat→eBPF compiler** — a restricted, *total* subset of wat (the verifier's
  constraints, below) lowered to **eBPF bytecode** by wat's own backend (rides the AOT compile-on-load path,
  `../001-metered-evaluation/CEK-MIGRATION.md`).
- **The bytecode emitter** — the eBPF **ISA encoding**, the **ELF** object, and **BTF** (BPF Type Format)
  for the verifier — all emitted by wat.
- **The loader — raw `bpf()` syscalls** (`BPF_PROG_LOAD`, `BPF_MAP_CREATE`, `BPF_PROG_ATTACH`, …), **no
  libbpf.** wat is a Linux language; it talks to the kernel directly.
- **The map / ringbuffer interface** — raw syscall + `mmap`, in wat (read perf/ringbuffer events the XDP
  signals; read/write the maps the XDP reads).
- **The daemon** — the holon anomaly engine, *full* wat.

**Aya = the reference impl we satisfy (the oracle). The kernel verifier = the ultimate judge.**

## Why ALL in wat — the trust-boundary argument (not harder-for-its-own-sake)

Wrap libbpf/aya and you place **unsigned Rust/C inside the trust boundary**: the thing that compiles and
loads your *signed* wat XDP program is itself foreign, unverified code in the loop. Build it all in wat and
**the whole chain — the XDP program, the compiler that lowers it, the loader that installs it — is wat:
signable, content-addressed, inside the verified substrate.** A signed wat daemon compiles a signed wat XDP
program and loads it via raw `bpf()` into the kernel, with **no foreign code in the path.** This is the
field-programmable-host model (`../001-metered-evaluation/FIELD-PROGRAMMABLE-HOSTS.md`) applied to the
kernel, end to end. *That* is why the hard case is the right case — the trust story stays pure.

## The architecture — preserve the lab's split

- **XDP fast path (kernel):** restricted `wat-bpf` → eBPF bytecode → **kernel verifies + JITs to native** →
  runs at the NIC, line rate. Parse headers, map lookups, drop/pass. The deny tree.
- **The brain (userspace):** full wat daemon + the holon/VSA anomaly engine — derives rules → writes the
  eBPF maps the XDP reads; reads the ringbuffer/perf events the XDP signals. Blue/green tree swaps are an
  admin op.
- They talk via **maps + ringbuffers** — the lab's existing shape, kept. The fast path stays small (the
  verifier's limits); the heavy thinking stays in the daemon.

## The foundation — tonight's compiler, pointed at the kernel

- **Loading eBPF IS compile-on-`eval` into the kernel** — the trusted host. The wat XDP program is compiled
  (wat→eBPF) and *eval'd* into the kernel, which verifies then JITs it.
- **The kernel's eBPF verifier is the field-programmable-host's verify-before-run — already running in every
  Linux box.** It does the `native ⊑ spec` proof for the eBPF subset **for free**. The thing we said we'd
  have to build, the kernel already *is*; wat just has to speak to it directly.
- **The typed-verifier-passing-by-construction win.** wat's type system can make a `wat-bpf` program
  verifier-legal **by construction** — bounds, heap-freedom, stack size, helper legality encoded in the
  types — so it *passes the verifier because it cannot express an illegal program*, instead of fighting the
  verifier the way C/aya programs do. "The type system is armor," pointed at the kernel. This is the genuine
  edge over aya, not parity.
- **wat→eBPF is a BOUNDED backend** — a tiny fixed ISA (~100 opcodes, 11 registers), no x86-scale register
  allocation, with prior art to crib (the LLVM eBPF backend, aya, bpf-linker). Smaller than the x86 AOT
  path, because **the kernel does the final native JIT half.**

## The oracle — aya, and the kernel (the dual-impl doctrine)

Aya is the mature Rust reference; the kernel verifier is the judge. The differential: run the same XDP
program through **aya's flow** and **wat's flow** — the kernel must verify-and-load both, the loaded bytecode
must be behaviorally equivalent, the same packets must drop/pass at line rate. *Same input → same
kernel-accepted program → same behavior.* This is 278 R9's dual-impl doctrine (the rete engine's
wat-oracle-vs-Rust-kernel; 291 R2's loopback oracle) turned on the packet path: aya is the known-good
reference; the kernel is the un-foolable judge; wat is the impl under test. **Satisfy aya; satisfy the
kernel.**

## The honest hard edges

- **`wat-bpf` is a RESTRICTED total subset, not full wat in the kernel.** Bounded loops, ≤512-byte stack,
  no heap, restricted helper set, provable termination — the verifier's law. Full wat runs in the *daemon*;
  the kernel gets the restricted dialect. The type system enforces the line.
- **BTF / CO-RE is the fiddliest piece** — the BPF Type Format + compile-once-run-everywhere relocations the
  loader/verifier want for portability. **There is a simple first path:** direct bytecode load without CO-RE
  (pinned to a kernel version), then add BTF/CO-RE for portability once the loop runs.
- **The verifier is finicky** (every aya/C author fights it). The typed approach converts *fighting* into
  *passing-by-construction* — but encoding the verifier's constraint set into wat's types is the real work,
  and it is where most of the design effort goes.
- **The ISA / ELF / `bpf()`-struct encodings are bounded, documented work** — you emit a documented format
  and call documented syscalls; you invent nothing. Mechanical, not research.

## Reachability — a component map, no person-years

- **The daemon** (full wat + raw syscalls + the existing holon engine) — *near.* A raw-syscall primitive, the
  `bpf()` struct encodings, the map/ringbuffer reads, the anomaly logic you already have.
- **The wat→eBPF compiler + emitter + loader** — a real, *bounded* project on tonight's compiler foundation;
  the loader and ISA are the most mechanical, BTF/CO-RE the fiddliest, the typed-verifier-encoding the
  deepest.
- **Bridge drawn; build ahead.** The kernel hands you the hard half (verify + native JIT) for free.

## The convergence — the loop, fully closed

The epilogue (2026-06-01) already named the destination: *"`wat-schema` — the WAF replacement — the firewall
I started with, reborn in the language the firewall made me build."* The DDoS lab is the **origin** of the
entire project. Wrapping aya would close the loop with a Rust dependency still in it; **all-in-wat closes it
cleanly — the firewall reborn *entirely* in wat, no foreign code in the trust path.** The origin, returned
to, with the tools it spent the project forging. *Iam scriptum est.*

## Cross-references

- `../001-metered-evaluation/CEK-MIGRATION.md` — the AOT compile-on-load / compile-on-`eval` compiler the
  wat→eBPF backend rides; the JIT annihilation (the kernel JITs eBPF to native; wat only emits the bytecode).
- `../001-metered-evaluation/FIELD-PROGRAMMABLE-HOSTS.md` — the kernel as the trusted host that
  verifies-before-it-runs; the field-programmable model, here found already living in Linux.
- `holon-lab-ddos/veth-lab/` — the current lab: aya/Rust XDP (`filter-ebpf`) + Rust sidecar; the reference
  architecture and the thing replaced. Aya = the oracle.
- `algebraic-intelligence.dev` epilogue — *"the firewall I started with, reborn in the language the firewall
  made me build."*
