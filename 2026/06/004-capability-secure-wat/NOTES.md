# Capability-secure wat — confinement out of primitives

Captured 2026-06-15. Notes from a design thread. Builder:

> *"how can I do my own chroot? … if my lang doesn't allow the syscalls then it's a
> moot point. … a boot mode that just disables features at runtime … someone could
> try to run the prog but they fault on parse. … wat is user extensible — if someone
> builds their own wat server with their rust goodies, they own any fuck ups they
> allow."*

Companion to the **in-repo** banked note (the arc-local capture):
`wat-rs/docs/arc/2026/06/272-rendezvous-inherited-capability/NOTE-confinement-horizon.md`.
This scratch note holds the cross-cutting synthesis, the sharpened model, and the
responsibility boundary. **NOT BUILT — notes / horizon.**

## It's already built (foundation) + banked (horizon)

Arc 272 shipped the object-capability spine — confinement isn't a new subsystem, it's
*generalizing the powerbox*:
- **The capability waist** (`wat-edn.cap`, `src/capability/registry.rs`) — a frozen wire
  contract + generic encode/decode + a registry. A new capability = one registry row;
  the core never moves. The doc names the future registrants: *"`Address'`, a future
  `Grant`, `Lease`, `Token`…"* — **`Dir'` / `File'` slot right in.**
- **The trust door** (`edn_shim::decode_trusted_wire`) — *a capability reconstructs ONLY
  off the trusted wire, handed over a lineage channel, **never forged from parsed
  data.*** This IS "you can't conjure authority by naming it in source." A confined
  program writing `#wat-edn.cap/Dir "/etc"` in its source gets nothing.
- **The powerbox** (`CommsPolicy`, `src/capability/policy.rs`) — Mark Miller's powerbox;
  admits from kernel-verified `SO_PEERCRED`; `OnlyMyPeers` / `AnyOfMyUser`, shaped to grow
  to `fn(PeerCred) -> bool`.
- **The horizon** (`NOTE-confinement-horizon.md`) — already states it: confinement = the
  powerbox generalized from "which peers" to **all authority** (files, net, syscalls); a
  `ConfinementPolicy` at the `clone3` seam; ns/seccomp/Landlock; Capsicum as prior art;
  *do not build the forcing function.*

## The sharpened model — language ocap is the ceiling, OS is the floor

1. **The environment IS the capability set.** "If wat doesn't bind `exec`, it's moot" =
   pure ocap: deny by *absence*, not policy. No syscall surface to escape because the
   program can't *name* the door. This is the **primary** fence.
2. **fs is a HANDED `Dir'` capability, not ambient `open`/`slurp`** (the WASI preopen
   model), riding the waist + trust door already built. A confined spawn is handed a
   `Dir'` scoped to `/jail`; the only file op is "open a path *under this `Dir'`*." There's
   no global `open` to lock down because there isn't one. Ambient `slurp` that reads
   anything is exactly the ambient authority ocap exists to delete.
3. **Fault at resolve/CHECK, not literal parse** (a Lisp reads `(exec …)` as a uniform
   list regardless). Better: the type checker (`src/check`) can **statically reject** a
   program that references a not-granted capability → confinement-violating programs are
   *uncompilable in that profile*. The extirpare top rung: unrepresentable, fail-closed at
   the earliest honest phase.
4. **OS floor = defense-in-depth** (the note's `clone3` seam): namespaces / seccomp /
   Landlock contain a leaky native intrinsic or a runtime bug under the language ceiling.
   Firecracker / gVisor only for genuine *strangers* (hardware-grade).

## The responsibility boundary (the honest scope — load-bearing)

**wat is user-extensible, and the confinement guarantee is scoped to what wat vends.**
The guarantee: *a confined program holds only the capabilities it was handed* — true for
wat's own surface. But if you register your **own** Rust intrinsic that grants ambient
authority (your `open-anything` builtin) and bind it into a confined environment, *you*
granted that authority — **you own the hole.** wat's capability machinery is sound; the
security of YOUR server = the capabilities YOU choose to grant.

This is the **vended-primitives-never-deadlock creed applied to confinement**
([[feedback_vended_primitives_never_deadlock]]): the platform's vended tools never fuck
up; users can deadlock their *own* logic. Same here — wat's confinement primitives are
airtight; the Rust goodies you bolt on are *your* audit, *your* responsibility. POLA
handed to the operator: least authority by default + the machinery to keep it; what you
widen is on you. **No magic that pretends an arbitrary extension is safe**
([[feedback_no_magic_that_lets_llm_fake_correctness]]).

The extension seam (register-your-own-Rust-cap) **is** the trust boundary — that's where
the audit lives: *which of my intrinsics touch fs/net/exec/clock, and which do I bind into
a confined env?* wat can't (and shouldn't pretend to) answer that for you; it gives you a
capability-secure default and the narrow waist to stay inside it.

## Status / when it graduates

NOT BUILT — notes / horizon (the in-repo note keeps the spawn surface general; don't build
the forcing function). Graduates to its own arc when a real "run untrusted wat safely"
caller appears — the security-platform / anti-botnet work ([[user_career_anti_botnet]]) is
the obvious one. Then: `ConfinementPolicy` + `Dir'`/`File'` as waist registrants + a confined
root-environment constructor + the static check + the clone-seam floor + a probe that a
confined child genuinely cannot `open`/`socket`.

## Cross-references

- wat-rs arc 272 — the waist (`capability/registry.rs`), trust door (`edn_shim`), powerbox
  (`capability/policy.rs`), `NOTE-confinement-horizon.md`.
- `../001-metered-evaluation/` — running strangers' programs for pay *needs* confinement;
  `../003-verified-eval/` — verify the *result*; confinement contains the *execution*. The
  two halves of "trust untrusted wat": verify the output, confine the run.
- Memory: `feedback_vended_primitives_never_deadlock`, `feedback_no_magic_that_lets_llm_fake_correctness`,
  `project_rendezvous_inherited_capability` (arc 272), `user_career_anti_botnet`.
