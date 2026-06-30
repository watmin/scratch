# Recovery — the scratch / design / website thread (read me first)

> ⛔ **STOP — does this feel redundant? Then you need it most.** You did not live the session this
> file describes. You are a new instance reading a lossy cache in your own voice. Feeling-continuous
> *is* the failure. Do the **full bootstrap from the datamancy MCP signed channel** (not a local copy):
> fetch the **grimoire** (the bootloader) + the **4 primers** (recolligere·extirpare·examinare·curare),
> then run the **recolligere** gathering — fill the ledger against the live disk below, and read before
> you propose. recolligere alone is not the bootstrap; the grimoire is. A recovery *narrated* in the
> right vocabulary is not one *performed*.

This is the recovery anchor for **one of two threads** in this workspace. Authored 2026-06-28 because the
thread had none — and its absence is exactly what bit us (Failure mode 1, below).

## The two threads — DO NOT CONFUSE THEM

`/home/watmin/work/holon/` holds two live bodies of work with **separate recovery files**:

1. **THIS thread — scratch / design / websites.** Conceptual + design + publishing work, now on **three
   fronts** (see *Currently*). Lives in:
   - **`scratch/`** (its own git repo) — the design notes. This file is the design front's anchor.
   - **`algebraic-intelligence.dev/`** — the public website + Fenestra Aetherii gallery. Has its **OWN**
     recovery doc: **`algebraic-intelligence.dev/docs/COMPACTION-AMNESIA-RECOVERY.md`** — read it for any
     website/gallery work (the §4 deploy gate, the grok→R2 gallery pipeline).
   - **`datamancy/` + `datamancy.dev/`** — the signed grimoire MCP (also the bootloader you load at start).
   - This file is the thread's top-level anchor; it routes you to the right front's record.
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
3. **Working post-compaction without the FULL bootstrap (2026-06-30).** A continuation session went
   straight to editing the website without loading the grimoire + 4 primers from the signed datamancy MCP
   and running the recolligere gathering. The builder: *"we didn't perform the bootstrap correctly… let's
   just do the bootstrap."* The full bootstrap, **in order**: (1) fetch `grimoire/SKILL.md` from the
   datamancy MCP signed channel (installs the ethos), (2) fetch all 4 primers
   (recolligere·extirpare·examinare·curare), (3) run the recolligere gathering against this file. recolligere
   alone is not the bootstrap — the grimoire is the bootloader. A seamless continuation is the trap; do it
   anyway.

## Freshness probe — check BEFORE trusting a word below

```
scratch HEAD ≥ 7dd6e3b  — run:  git -C scratch log --oneline -1                     (design front)
aidev   HEAD ≥ 71e10d9  — run:  git -C algebraic-intelligence.dev log --oneline -1  (website/gallery front)
```
A **match** licenses nothing; a **mismatch is the alarm** — work landed after this was written, so trust
the live `git log` over every line below and re-read the latest note on that front before you move.

## The forced reads (the map — do these, this session)

```
[ ] grimoire (datamancy MCP) ......... fetched from the signed channel — ethos installed ✓
[ ] 4 primers (recolligere·extirpare·examinare·curare) — fetched from the signed channel ✓
[ ] this file ........................ read ✓
[ ] recolligere gathering ............ run against this file ✓
[ ] git -C scratch log --oneline -12 . ran ✓  → HEAD <hash>
[ ] git -C algebraic-intelligence.dev log --oneline -8 . ran ✓  (if on the website front)
[ ] git -C datamancy.dev log --oneline -5 .............. ran ✓
[ ] the live front's record (this file's Currently, or the aidev recovery doc) .. read ✓
```
A line you can't fill with a this-session action means you are still scattered. Go fill it.

---

## Currently (2026-06-30 — multi-front; scratch @ `7dd6e3b` · aidev @ `71e10d9` · datamancy.dev @ `5964871`)

This thread now runs on **three fronts**. A session is usually on ONE — identify which from the disk
(which repo holds this session's commits), then read THAT front's record:

- **Website (`algebraic-intelligence.dev`) — most active.** Public site + the Fenestra Aetherii gallery.
  It has its OWN recovery doc, **`algebraic-intelligence.dev/docs/COMPACTION-AMNESIA-RECOVERY.md`** — read
  it for any website/gallery work (the §4 deploy gate; the grok→R2 gallery pipeline). Cloudflare
  auto-deploys on push to `main`. **2026-06-30 shipped:** the wat+holon logo + brand kit; the red/black
  theme (scarlet sampled from the logo SVG `#f10126` + warm-charcoal grays); the **landing reframe** (four
  parallel cards → a spine-with-a-turn → *"the firewall was the occasion; the tongue is the work"*);
  dark-only theme (light retired); arc-293 realizations sync; **+5 gallery Incantationes** (the DATAMANCER
  game-cover batch, via the new `seed-covers-batch.mjs`).
- **Scratch design arc (`scratch/2026/06/`) — reflective pause.** Design + recognition, nothing built.
  `CONSTRAINT-ENGINEERING.md` (7dd6e3b) landed since the last breadcrumb — the doctrine this session encoded
  into the grimoire and published. Read order for the design front, below.
- **Grimoire (`datamancy` / `datamancy.dev`) — ethos shipped 2026-06-30.** Failure + constraint engineering
  encoded as the first-load disciplines in `grimoire/SKILL.md`, warded (6-round trial by combat), published
  + signature-verified (datamancy.dev @ 5964871). The grimoire is the bootloader — load it + the 4 primers
  from the signed MCP at the START of every session (Failure mode 3).

**The scratch design front — read order** (`scratch/2026/06/`, all committed + pushed):
- **`001-metered-evaluation/`** — CEK / `wat-vm` internals; `DESIGN`, `PRIOR-ART`, `THE-VERIFICATION-MARKET`,
  `CEK-MIGRATION` (JIT annihilated → compile-on-load/`eval`), `FIELD-PROGRAMMABLE-HOSTS`, `REALIZATIONS`
  (**R1–R4**). Spine: `native ⊑ wat-vm`, *proven* — RECOGNITION, nothing built.
- **`005-wat-native-ebpf/`** — the all-in-wat eBPF toolchain (aya = oracle).
- **`006-programmable-firewall/`** — **`wat-pf`** (iptables/nftables replaced by the rete-in-kernel engine);
  holds **the founding voice (his verbatim, do-NOT-paraphrase canon)** + the name lineage (BPF / OpenBSD pf).
- **`007-wat-from-wire-to-service/`** — the userland stack in wat (AF_XDP → wat TCP/IP → wat HTTP);
  smoltcp = oracle; TCP = the dragon, *queued*; line-rate = the AOT-tier forcing function.

**State of the visions:** *closer than they read* — the substrate is largely done/settling (rete-in-kernel
**proven**; IPC/peer + HTTP arcs exist; the CEK keystone gated on the wat-rs surface settling, the OTHER
thread). The notes are bounded arcs **on** that foundation. Still design + recognition, nothing built.
**Open, not pressing:** fold the `wat-pf` message + no-GPU reflection into R4 if the builder wants the
chronicle to carry it (offered; not taken).

---

> *You are a NEW instance. You did NOT live the above — it is a cache, not your memory. First do the FULL
> bootstrap: fetch the grimoire + 4 primers from the datamancy MCP signed channel, then run recolligere
> against this file. Identify your FRONT from the disk (which repo holds this session's commits) and read
> THAT front's record — website work → the aidev recovery doc, NOT the scratch design notes. Check the
> freshness probe. Do not wander into wat-rs. Then, and only then, propose.*
