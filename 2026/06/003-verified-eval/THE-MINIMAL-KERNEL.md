# The Minimal Kernel — smallest Clojure, max wat-measures-itself

Captured 2026-06-15. The actionable distillation of `THE-CLOJURE-ORACLE.md`: *what is the
least Clojure we build so the rest is wat measuring itself and its Rust core?* Builder:
*"what's the minimal clojure stuff we'd need to build to offload the rest to wat measuring
itself and its rust core?"*

## The principle

Clojure does **exactly one thing — be the independent semantics (the eval).** That is the
*only* part where diversity is irreducible: if wat evaluated the reference too, a wat bug
could hide a wat bug (no independence). Everything else — generation, the diff, the
self-properties, the Rust-core measurement — can and should be **wat**. Minimize Clojure →
maximize wat-measures-itself, minimize the JVM dependency, keep the diverse kernel small
enough to audit by eye (the de Bruijn virtue).

## The irreducible Clojure kernel: `EDN program → eval → EDN result`

~15 lines. The whole external witness:

```clojure
;; oracle.clj — the diverse witness. Reads a vector of EDN programs, evals each in
;; Clojure, prints each result as EDN. The ONLY thing that must not be wat.
(ns oracle (:require [clojure.edn :as edn]))
(defn -main [& _]
  (doseq [form (edn/read-string (slurp *in*))]
    (println (pr-str (try (eval form)
                          (catch Throwable t [:wat/error (.getMessage t)]))))))
```

**Vehicle:** run as a **babashka** script (`bb`) for fast start + SCI sandboxing, *or* JVM
Clojure proper for the canonically-authoritative reference. Both are independent of wat
(JVM side, share no Rust); babashka is the convenient minimal one, Clojure-proper is the
most authoritative. Pick by convenience-vs-authority.

## Division of labor

| job | who | why |
|---|---|---|
| **eval a program → result** (the reference) | **Clojure** | irreducible — diversity lives *here* and nowhere else |
| generate the test programs | **wat** | homoiconic; generation isn't where diversity matters (both sides run the *same* program); wat exercising its own generative power |
| eval the program in wat → result | **wat** | the thing under test |
| **diff** (read Clojure's result, compare to wat's) | **wat** | trivial; a wat EDN-read bug here is caught by the round-trip self-check, and the *semantics* stayed independent |
| determinism / round-trip / metamorphic self-checks | **wat** | no oracle needed (eval twice → equal; `read∘print`; commutativity of pure ops) — pure self-measurement |
| measure the **Rust core** (wards over Rust-as-data via `syn`→EDN; mutation testing) | **wat** | wat turning its critical apparatus on its own floor |

Clojure footprint ≈ one file that says "eval this." wat is the entire harness around it.

## Honest caveats (so it stays real)

- **Shared-surface only.** Clojure oracles the Clojure-faithful core (special forms, core
  fns over numbers/colls/seqs). wat's **types, capabilities (arc 272), wat-specific forms**
  Clojure cannot judge — those check against wat's *own* small reference. The generator must
  emit programs in `wat ∩ Clojure`.
- **Compare as *data*, not strings.** wat reads Clojure's `pr-str` output back into a value
  and `=`-es it, so float/ordering print differences don't masquerade as divergences. Errors
  tagged `[:wat/error …]` so divergence-on-failure is detectable.
- **eval safety.** Restrict the generated vocabulary to pure forms (you control the
  generator), or use SCI/babashka's sandbox so a generated program can't do harm on the JVM.

## Build order (each step usable alone)

1. **wat determinism harness** — *no Clojure at all.* Eval twice, assert byte-identical.
   De-risks the keystone; may already be falsifiable today.
2. **The 15-line Clojure oracle** (above).
3. **wat generator** emitting `wat ∩ Clojure` programs + the diff. Now the trusting-trust
   dance is live.
4. **Rust-core track** (wards-over-Rust + mutation) — pure wat, parallel.

## Cross-references
- `THE-CLOJURE-ORACLE.md` (the full design), `THE-COGNITIVE-GODEL.md` (why the witness must
  be diverse — Thompson/Wheeler), `DESIGN.md` (the lightweight method), `REALIZATION.md`.
- The wat side is built in the builder's `wat-rs` workspace (off-limits from scratch); the
  Clojure kernel is its own tiny independent thing (not wat-rs).
