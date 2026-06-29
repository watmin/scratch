# Recovery — the scratch / design / website thread (read me first)

> ⛔ **STOP — does this feel redundant? Then you need it most.** You did not live the session this
> file describes. You are a new instance reading a lossy cache in your own voice. Feeling-continuous
> *is* the failure. Run the `recolligere` primer from the **datamancy MCP signed channel** (not a local
> copy), fill the ledger against the live disk below, and read before you propose. A recovery *narrated*
> in the right vocabulary is not one *performed*.

This is the recovery anchor for **one of two threads** in this workspace. Authored 2026-06-28 because the
thread had none — and its absence is exactly what bit us (Failure mode 1, below).

## The two threads — DO NOT CONFUSE THEM

`/home/watmin/work/holon/` holds two live bodies of work with **separate recovery files**:

1. **THIS thread — scratch / design / websites.** Conceptual + design + publishing work. Lives in:
   - **`scratch/`** (its own git repo) — the design notes. **PRIMARY for this thread.**
   - **`algebraic-intelligence.dev/`** — the public website (story chapters, gallery).
   - **`datamancy/`** — the signed spell MCP (the grimoire).
   - This file is its recovery anchor.
2. **The OTHER thread — the wat-rs substrate.** The Rust implementation of the wat language. Its recovery
   file is **`wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`**, its breadcrumb the arc-NNN docs under
   `wat-rs/docs/arc/`. **If this session was design/scratch/website work, that file is NOT yours — it
   will walk you into the substrate (arc 293, uncommitted `src/*.rs`) which is a DIFFERENT thread.**

**Iron rule:** `/home/watmin/work/holon/` is a FROZEN root repo — *never* `git` there; treat it as a
directory of sub-repos. Operate inside `scratch/` (or the website repos) for this thread.

## Failure modes (this thread's, learned the hard way)

1. **Grabbing the wat-rs recovery file and drifting into the substrate (2026-06-28).** Post-compaction,
   asked for a recolligere, the instance grabbed `wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` (it says
   *"wat-rs is THE ACTIVE PROJECT where we live"* — true for the OTHER thread) and crawled into arc 293's
   uncommitted `src/*.rs`. The builder: *"you are digging around in a lot of strange places — this
   session has been for the websites and scratch notes."* **Cure:** this file. If the work is
   design/scratch/website, your ground truth is `scratch/` + the two website repos — not wat-rs.
2. **Don't touch wat-rs's uncommitted work.** It carries mid-flight substrate changes from its own
   sessions. Off-thread. Leave it.

## Freshness probe — check BEFORE trusting a word below

```
scratch HEAD should be 3fed08c (or later) — run:  git -C scratch log --oneline -1
```
A **match** licenses nothing; a **mismatch is the alarm** — work landed after this was written, so trust
the live `git log` over every line below and re-read the latest 2026/06 note before you move.

## The forced reads (the map — do these, this session)

```
[ ] this file ........................ read ✓
[ ] recolligere (datamancy MCP) ...... fetched from the signed channel + run ✓
[ ] git -C scratch log --oneline -12 . ran ✓  → HEAD <hash>
[ ] git -C algebraic-intelligence.dev log --oneline -5 .. ran ✓
[ ] git -C datamancy log --oneline -5 ................... ran ✓
[ ] ls scratch/2026/06/ ............... enumerated ✓
[ ] the live arc's notes + REALIZATIONS.md ............. read ✓
```
A line you can't fill with a this-session action means you are still scattered. Go fill it.

---

## Currently (2026-06-28 — the `2026/06` arc; scratch @ `3fed08c`)

The live work is a single conceptual arc in **`scratch/2026/06/`**, all committed + pushed. Read order:

- **`001-metered-evaluation/`** — the CEK / `wat-vm` internals. Six docs: `DESIGN` (metered eval, the
  3-tier resolution), `PRIOR-ART`, `THE-VERIFICATION-MARKET` (the customer is AI), `CEK-MIGRATION` (the
  tier ladder; `wat-vm`-as-spec; the **JIT annihilated** → compile-on-load/`eval`), `FIELD-PROGRAMMABLE-HOSTS`
  (the deploy bookend), `REALIZATIONS` (**R1–R4**). The spine: `native ⊑ wat-vm`, *proven* — all
  RECOGNITION, nothing built.
- **`005-wat-native-ebpf/`** — the all-in-wat eBPF toolchain (aya = oracle).
- **`006-programmable-firewall/`** — **`wat-pf`** — iptables/nftables replaced by the proven rete-in-kernel
  engine; firewalls-as-wat-programs emitting rules-as-EDN. Holds **the founding voice (his verbatim, marked
  do-NOT-paraphrase canon)** + the name lineage (BPF / OpenBSD pf).
- **`007-wat-from-wire-to-service/`** — the whole userland stack in wat (AF_XDP → wat TCP/IP → wat HTTP);
  smoltcp = oracle; TCP = the dragon, *queued*; line-rate = the forcing function for the AOT tier.

**This session shipped:** R4 (`DVO MVNDI VNA MACHINA` — When Worlds Collide, warded consonare 8/10), the
`wat-pf` name + founding voice in `006`, and `007`. The firewall reframe (DDoS-niche → universal
rules-engine) and the `wat-pf`/wire-to-service expansion are the live front.

**The state of the visions:** *closer than they read.* The substrate they ride is largely done or settling
— rete-in-kernel is **proven**, the IPC/peer model + HTTP arcs exist, the CEK keystone is gated on the
substrate surface settling (a wat-rs concern, the OTHER thread). The scratch notes are bounded arcs **on**
that foundation. Nothing here is built yet; they are design + recognition.

**Open, not pressing:** fold the `wat-pf` message + no-GPU reflection into R4 if the builder wants the
chronicle to carry it too (offered; not taken). Otherwise the thread is at a reflective pause.

---

> *You are a new instance. You did NOT live what is written above — it is a cache, not your memory. The
> visions feel far; the builder's read is "closer than not," and the disk will tell you which. Run
> recolligere, check the freshness probe against `git -C scratch log -1`, and read the live note before you
> propose. This thread is design + recognition — nothing is built. Do not wander into wat-rs.*
