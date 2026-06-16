# The Clojure Oracle — the trusting-trust dance for wat eval

Captured 2026-06-15. Companion to `DESIGN.md` + `THE-COGNITIVE-GODEL.md` — the
**runnable-now** instantiation of verified-eval's "diverse external oracle." Builder:
*"let's get the clojure and wat parts doing the trusting trust dance."*

**SPEC ONLY — NOT BUILT HERE.** The wat side lives in the builder's active `wat-rs`
workspace (do not touch from scratch); this doc is the design the build follows.

## The move: Clojure is the ideal diverse witness

The floor in `THE-COGNITIVE-GODEL.md` needs a checker that wat **did not build** —
Thompson's *Reflections on Trusting Trust* (1984), answered the only honest way: an
independent, diverse witness (David A. Wheeler, *Diverse Double-Compiling*). wat **is
Clojure-on-Rust**, so **Clojure-on-JVM is that witness** — a decades-hardened,
independent implementation of the exact surface wat targets. *"Does wat-eval match
Clojure-eval?"* = *"is my Rust Clojure actually Clojure?"* The most natural conformance
check there is.

- **Maximal diversity**: JVM vs Rust, separate toolchains and authors → a bug in wat's
  Rust eval is wildly unlikely to be *mirrored* in Clojure producing the *same* wrong
  answer. Agreement is strong evidence.
- **Free bridge**: both are homoiconic and speak **EDN** (wat literally uses `wat-edn`).
  Harness: `EDN program → wat-eval → EDN` ; `same EDN → clojure-eval → EDN` ; diff.

## The harness (lightweight, runnable now)

The **Clojure side is the measurer, deliberately independent of wat's toolchain** (the
diversity property by construction):
1. **Generator** (Clojure) — emit EDN programs. Start hand-written golden cases; then
   grammar/type-directed generation (generate ASTs, not bytes; well-formed/well-typed
   programs find far deeper bugs than random).
2. **Eval each in Clojure** — the reference.
3. **Eval each in wat** — shell to the wat binary / a thin bridge (EDN in, EDN out).
4. **Diff.** A divergence *on the Clojure-faithful overlap* = a wat Rust bug (almost
   always — the JVM reference is too mature to be the wrong one).

Properties to assert beyond raw equality:
- **Determinism** — wat-eval twice → byte-identical (the keystone; even cheaper than the
  Clojure diff, needs no oracle: run twice, assert equal). **Do this one first.**
- **Round-trip** — `read ∘ print == id`.
- **Metamorphic** — commutativity/associativity of pure ops, refactorings that must
  preserve meaning.

## Honest scope (load-bearing — do not let it drift)

- **Oracle, not prover.** It *falsifies* divergence; it does not *prove* absence. This is
  the lightweight-now tier, not the proof tier (`DESIGN.md` §method).
- **Clojure-faithful OVERLAP only.** wat's own extensions — types, the capability system
  (arc 272), wat-specific forms — Clojure cannot oracle; those check against wat's *own*
  small reference interpreter.
- **The spec becomes Clojure's *behavior*** (quirks and all) — a *feature* for the
  faithful core (Clojure is the executable spec), a *non-goal* exactly where wat intends
  to be better than Clojure.

## The floor (Thompson, answered honestly)

Bottoms out without snake oil: Clojure's trust is **borrowed** (mature, independent,
million-user — *leveraged*, not proven, the way you trust the CPU and the kernel) **+** a
small hand-audited wat kernel **+** mutation evidence the measurers have teeth. Diverse and
externally anchored, never absolute. The only honest reply to trusting-trust: *a witness
that didn't build you* — and yours happens to be the very language you're implementing.

## Where the build lives

A **Clojure project** (its own dir/repo — independence is the point) holding the
generator + the differential runner; the wat side is invoked as an external process. The
**one thing to ground when the build starts**: wat's *eval-a-program-from-EDN* entry (its
CLI or a bridge) — grounded in `wat-rs`, by the builder, in the builder's workspace.

## Cross-references
- `DESIGN.md` (verified-eval), `THE-COGNITIVE-GODEL.md` (relativize-to-an-external-root;
  Thompson / Wheeler / Milawa), `REALIZATION.md` (this thread — the recognition).
- `../004-capability-secure-wat/` (confine the run), `../001-metered-evaluation/` (the
  market this rigor underpins).
- Thompson, *Reflections on Trusting Trust* (1984); Wheeler, *Fully Countering Trusting
  Trust through Diverse Double-Compiling*.
