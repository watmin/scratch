# The Conformant Surface — handling wat≠Clojure + strong typing

Captured 2026-06-15. Sharpens `THE-CLOJURE-ORACLE.md` / `THE-MINIMAL-KERNEL.md` against the
real edges. Builder: *"wat deviates from clojure in several forms but it does overlap
extensively - i'm not shooting for full parity (wat is also strongly typed) so there's some
concerns here to deal with."* The handling: make the surface an explicit artifact, make the
outcome multi-valued, and lean on the types as extra signal.

## 1. "Shared surface" → the conformant surface (an explicit positive whitelist)

Not "whatever overlaps" — a **curated list of forms + fns where wat *promises*
Clojure-identical behavior.** Two jobs:
- the **generator's vocabulary** (emit programs only over it), and
- a **spec artifact** in its own right — wat's *Clojure-fidelity contract*, documented and
  versioned (it grows/changes with the arc-251 migration).

Anything wat deliberately deviates on simply **isn't on the list**, so Clojure never gets a
vote there (those check against wat's *own* reference). A divergence *on* the surface is a
bug; a divergence *off* it is by-design and invisible to the oracle. **The list IS the
boundary between "Clojure is my oracle here" and "I'm my own thing here."**

## 2. Typed-vs-untyped → the type checker is an asymmetric gate

The pipeline forks *before* the diff:
- generate over the conformant surface → **type-check in wat**.
- **type-check fails** → wat won't run it → *not a differential case* (Clojure would
  dynamically run it, but there's no wat value to compare). A wat-type-system concern,
  handled separately.
- **type-check passes** → run in wat *and* Clojure → compare.

So only **well-typed wat programs** are differential-tested — exactly the space where
agreement is claimed. **Type-directed generation** (emit *well-typed* programs) is efficient
and finds deeper bugs than random forms that bounce off the checker.

## 3. The outcome is four-valued, not match/mismatch

This is how typed divergence is honored — it's a *bucket*, not a *failure*:

| bucket | meaning | action |
|---|---|---|
| **AGREE** | both ran, same value | ✓ |
| **DIVERGE** (on conformant surface) | both ran, different value | **real bug — auto-fail** |
| **WAT-REJECTS** | wat type-checks/errors out, Clojure runs | type system doing its job — *triage* (is it over-strict? a checker bug?), NOT a conformance failure |
| **WAT-RUNS, CLOJURE-ERRORS** | wat accepted+ran what Clojure errors on | review — a wat soundness bug, or a legit extension |

Only **DIVERGE-on-surface** auto-fails. The other two are triage signals, not red builds.

## 4. The flip — the type system is a SECOND oracle, not a tax

Typing isn't only a complication; it's extra signal. Every well-typed generated program
carries a **statically-predicted result type** from wat's own checker. So each program is
checked *twice, independently*:
- **Clojure** says what the *value* should be (external witness).
- **wat's type checker** says what the *type* should be (internal soundness —
  progress + preservation).

Agreement on *both* — right value (Clojure) AND right type (wat) — is far stronger than
either alone, and the two oracles are independent. The strong typing rides alongside the
Clojure value-oracle as a free soundness oracle.

## 5. The divergence classes to DECIDE (builder's call — wat's semantics, not guessed here)

For each, choose: *in-surface-as-is* / *in-surface-modulo-a-normalization* (compare after a
documented transform) / *excluded* / *wat-specific assertion* (vs wat's spec, not Clojure):
- **numeric tower** — Clojure auto-promotes int→bigint; typed wat may wrap / fix-width (the
  classic divergence).
- **equality semantics** — `=` across types.
- **nil / false truthiness.**
- **collection iteration order.**
- **arity / dispatch** behavior.

Enumerating these IS defining the conformant surface — same artifact.

## Net

Clojure oracles a **bounded, explicit, well-typed slice**; the type checker gates it *and*
adds a second oracle; the four-valued outcome lets typed divergence be a bucket, not a
failure; everything off the conformant surface is wat's own reference's job. The concern is
handled by: **a maintained surface artifact + a multi-valued outcome + the types as signal.**

## Cross-references
- `THE-CLOJURE-ORACLE.md` (the design), `THE-MINIMAL-KERNEL.md` (the 15-line eval witness +
  division of labor), `THE-COGNITIVE-GODEL.md` (why the witness must be diverse), `DESIGN.md`.
