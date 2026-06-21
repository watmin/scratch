# The LEAN-parity stones — the build log from "checks programs" to "checks proofs"

Captured 2026-06-19. The build log for giving wat a **logic** — the path from its current
type checker (which checks *programs*) to a proof kernel (which checks *proofs*: certified
absence, ∀-theorems, the things a proof assistant does). This is the **propositional tier**
of this thread. `DESIGN.md` verifies *results* operationally — the differential oracle, run
both sides and compare. This verifies *propositions* — the ∀ carried in a type, settled
without running a single input. **SPEC / ROADMAP — NOT BUILT.** A stone sequence,
dependency-ordered, in the builder's arc form (deliverable + the one decision each).

Grounded against wat-rs HEAD (`26b088b5`, recon 2026-06-19), so the starting line is real:
types are rank-1 HM `TypeExpr` (no dependent typing, no reified types — arc 251 is a
*syntactic* inversion, not reification); the total-pure fragment is a hand-curated
`is_pure_total` allowlist + runtime `pure?`/`deterministic?` predicates with no formal spec;
totality is **unchecked** (no fuel, no structural- or well-founded-recursion guard); the
checker is a ~20k-line monolith with no constraint/obligation hook; the differential oracle
(`fire-rules` vs `fire-rules-spec`) is rete-scoped. Those are the **starting points, not the
ceiling** — the point of the rest of this doc is that the ceiling moves.

## The stones

**S0 · Total evaluation.** CEK (arc 261) as a first-class, inspectable small-step semantics
+ a totality checker: structural recursion by default, well-founded recursion with a declared
decreasing measure for the rest. *Consistency floor — a diverging term inhabits any type, so
you can "prove" ⊥; totality is what makes the logic sound. Decision: the accepted
well-founded-recursion surface.*

**S1 · A specified pure-total fragment.** Promote the `is_pure_total` allowlist into a closed
sublanguage with a written soundness statement (a term in it has no effects and terminates).
That sublanguage is the calculus the logic reasons about. *Decision: exactly where the
boundary is drawn.*

**S2 · Reify term/type/proof as wat values.** `Term` / `Type` / `Proof` as wat ADTs with
quote/unquote — wat code constructs and inspects them as data. *Keystone: proofs are terms,
tactics are programs that build terms, the kernel walks them — none of it is expressible
while types live only inside check.rs. Homoiconic already; this exports the typed AST as
values instead of consuming it internally. Most of the rest is dead without it.*

**S3 · The fork — the one real architectural decision:**
- **A — dependent types in the checker** (Lean/Coq): Π/Σ/universes, types depending on terms,
  definitional equality. Max power, most invasive — the structural-HM equality core becomes a
  normalizing dependent checker.
- **B — a proof layer beside the program** (Dafny/F\*/Why3): programs keep today's types; add
  a spec language (pre/post/refinement/invariant) + a VC generator + SMT. Fastest to
  certified-absence on real code, most automatable, least invasive.
- **C — an LCF kernel** (HOL Light): a tiny trusted `Theorem` ADT whose only constructors are
  the ~10 primitive inference rules; everything above is untrusted code that can only mint
  theorems *through* the kernel. HOL Light's kernel is ~400 lines. **The wat-consonant pick —
  a narrow trusted waist + unbounded tactic power above it, the de-Bruijn-clean TCB the
  project already builds everywhere (ocap, the signed manifest, the differential oracle).
  B's SMT automation and even A's dependent power graft on top without growing the trusted
  core.**

**S4 · The kernel — small, audited, the thing everything rests on.** de Bruijn-indexed terms
(the literal de Bruijn now, not the metaphor), conversion checking (definitional equality via
normalization to WHNF), the inference rules, a universe ladder (`Type : Type` is
Girard-inconsistent). ***Boss #1:*** the conversion checker is subtle and a bug there is a
soundness hole. Mitigation is the project's own move — **differential-oracle it**: a pure-wat
spec kernel vs the Rust kernel, frozen-for-frozen, the rete play exactly.

**S5 · The superstructure — all untrusted, above the kernel, wat's home turf.**
- elaboration: implicit-argument inference + unification. ***Boss #2:*** higher-order
  unification is undecidable in general; the survivable path is Miller's pattern fragment +
  heuristics. This is where Lean spends most of its engineering, and it is pure UX — every
  result still bottoms out in a kernel-checked term.
- tactics as macros: `intro` / `apply` / `induction` / `rewrite` / `simp` are programs that
  emit kernel terms. The total-pure macro engine is the substrate this was built for.
- the SMT bridge for the automatable fragment (linear arith, bitvectors, arrays — crypto and
  protocol territory): emit to Z3/cvc5, and for de-Bruijn-clean trust, check the solver's
  proof certificate (LFSC / Alethe) *through* the kernel instead of trusting the solver.

**S6 · The edge LEAN structurally cannot match — the reason to do it in wat at all.** Lean and
Coq prove, then **extract** to OCaml/C to run — trust is paid again at the boundary, and the
running thing is not the proven thing. In wat the verified term **is** wat, runs native on the
Rust substrate, proof erased at compile. Proven artifact and deployed artifact are one object.
*Build item: the verified fragment has to compile native, not just check.*

Two bosses, marked: **S4 conversion checking** and **S5 higher-order unification.** The rest is
porting specified designs.

## The reframe that matters — the stones are substrate-hardening, not a cost ledger

The list above reads like a bill. It is not one, because of how this substrate is built:
**every stone hardens wat-the-platform, and the prover falls out as the byproduct.**

- S0's CEK kills the native-stack recursion ceiling (a *standing platform bug* — the runner
  SIGSEGV'd on a ~960-line file, arc 261) — that is not prover-overhead, it is wat finally
  getting a real evaluator.
- S2's reified term/type AST powers far more than proofs — codemods, the macro engine,
  reflection, the auto-fixer line (arc 277/283).
- S1's formally-specified total-pure fragment sharpens the macro-eval fence (249) and the rete
  purity gates (278) that already exist and are already load-bearing.

So going after LEAN-level is not a side-quest with a big upfront bill — it is *the next hard
problem*, and the hard problem **is** the forcing function. The grounded instance is rete: "a
better Clara in two days" was the benchmark, and it was slow *only* because building it forced
the substrate's flaws into the open — the collection-tooling gap blocking user-supplied
aggregation, the stack-recursion ceiling — flaws the build surfaced and the builder attacked
on contact, so wat came out **more correct than before the problem existed.** In his words:

> building rete forced us to find our flaws and attack them, relentlessly — wat gets more
> correct for every hard problem we go after.

That is the reach-stumble (`feedback_reach_stumble_is_the_signal`) operating as a **ratchet on
the substrate**, not just on the feature. The cost of these verification stones is therefore
*self-amortizing*: each one pays back across all of wat, and the prover is what is left
standing when the hardening is done.

And it is why the canon's timelines do not transfer. Forgy's rete, Brush's Clara, seL4's
proof-years — built once, by teams, as *the* deliverable. Here the deliverable is the
byproduct of hardening a substrate that **compounds**: the next hard problem starts from a
more-correct wat than the last one did. The rate is not fixed; it accelerates, because the
substrate ratchets. Pricing this build log in person-years is the same category error as
pricing rete in Forgy's eight — it measures the canon's path, not this one.

## Cross-references
- `DESIGN.md` — verified-eval, the *operational* tier this sits above (run-and-compare).
- `THE-MINIMAL-KERNEL.md` — the diverse-oracle kernel; S4's differential-oracle mitigation is
  literally that move at the proof-kernel tier.
- `THE-COGNITIVE-GODEL.md` — relativize trust to a small, external, auditable root; the LCF
  kernel + certificate-checking is that theorem worn at the propositional tier.
- `../001-metered-evaluation/`, `../002-the-second-system/` — the verification-market this
  underwrites: a fast substrate whose *critical components carry machine-checked proofs* is a
  trust moat no extract-to-run prover delivers (S6).
- Origin: the LEAN-vs-wat thread (2026-06-19) — operational vs propositional equality; a proof
  is a finite witness of an infinite truth; the gap is real, and it is the propositional tier
  of the project's own spine, not a foreign country.
