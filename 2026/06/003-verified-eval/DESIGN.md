# Verified Eval — S3-style formal verification of wat, on itself

Captured 2026-06-15. The builder's framing, verbatim:

> *"so i was thinking something like 's3 has formally verified get-object is
> <foo-bar-whatever-term>'"*

Companion: `THE-COGNITIVE-GODEL.md` (this dir) — the conceptual spine (why this is
the project's central thesis made formal). **Status: NOT BUILT — capture to mull**,
in the lineage of `001-metered-evaluation` (which this underpins).

---

## The reframe that dodges the wall

The first pass at "can wat prove things about itself?" went straight to **Gödel's
second incompleteness theorem** (1931): a consistent, sufficiently strong formal
system *cannot prove its own consistency from inside itself*. wat is Turing-
complete, so a "wat proves wat is sound, whole" claim is either false or secretly
relativized. That's **item 3** of the earlier list, and it's a hard mathematical
wall, not an engineering gap.

But that is **not** what S3 does, and not what the builder means. The real question
is the **AWS automated-reasoning** question: *can a load-bearing operation carry a
formally-verified property against a spec* — "GetObject satisfies `<strongly-
consistent / crash-consistent / conforms-to-its-reference-model>`". That is:

- **item 1** — a proof *checker* (a deterministic verifier; a small trusted kernel), and
- **item 2** — proofs of *specific properties* of specific code, *relative* to a spec.

Items 1 + 2 sit on the **safe side** of Gödel. The whole trick AWS uses is that
they **never attempt item 3.** Every dollar of their formal-methods spend verifies
*relative* properties against a reference; none of it tries to prove the system's
own absolute consistency. **You don't fight Gödel — you route around him by
relativizing.** That's the move.

## The precedent, grounded (cite real names)

- **TLA+ at AWS** — Newcombe, Rath, Zhang, Munteanu, Brooker, Deardeuff, *"How
  Amazon Web Services Uses Formal Methods"* (CACM, 2015). S3 / DynamoDB / EBS
  *modeled and model-checked* at the protocol level. Found real, deep bugs.
- **S3 ShardStore** — Bornholt et al., *"Using Lightweight Formal Methods to
  Validate a Key-Value Storage Node in Amazon S3"* (SOSP 2021). The load-bearing
  word is **lightweight**: they did NOT prove S3 correct. They wrote an
  **executable reference model**, **conformance-checked** the real Rust storage
  node against it under massive **property-based testing**, and added **targeted
  proofs** of the few invariants that mattered (crash-consistency). *Pin the
  load-bearing properties; don't boil the ocean.* This is the template.
- The kernel discipline: **LCF / the de Bruijn criterion** (Milner 1972; de Bruijn /
  Automath) — a proof is trustworthy if a *small* checker accepts it; trust bottoms
  out in one tiny auditable thing.
- Self-application precedents (the elegant end): **Milawa** (Jared Davis, 2009) — a
  prover with a tiny kernel + a tower of reflective layers, each proven sound by
  the one below, until it checks a proof of *its own* soundness *relative to the
  kernel* (relativized, so Gödel-safe). **CakeML** (Myreen et al.) — an ML verified
  in HOL, bootstrapped and run on its own verified self.
- The honest cost of going *all* the way: **CompCert** (Leroy), **seL4** (Klein et
  al.) — closing the *implementation* gap (spec-vs-running-code) is person-decades.

## The wat target — and why it is the keystone, not gold-plating

The analog of "GetObject is strongly consistent" is:

> **wat-eval is deterministic and reproducible.** Same program + same input → byte-
> identical output, on any host.

This is not cosmetic. It is the **axiom the entire signed/metered/content-addressed
thesis rests on** (`001-metered-evaluation`, the wat-mcp verification market): a
**signed eval receipt is worthless if eval is not provably deterministic.** "Pay to
trust the oracle" collapses the instant eval can diverge. And the substrate already
*leans* on this without having proved it — `holon-rs`'s `vector_manager` guarantees
"same seed → same vector everywhere, enabling distributed consensus," which is a
determinism-for-consensus invariant hiding in plain sight. Formalizing it makes the
axiom honest instead of assumed.

### Property priority (each one a "GetObject is `<foo>`")

1. **Determinism / reproducibility of eval** — the trust root. Everything else is
   collateral on this. Do it first.
2. **Type soundness** — progress + preservation for the typed core (`wat-rs/src/check`).
3. **Macro hygiene / expansion confluence** — arc 249 territory.
4. **Termination** for the declared-*total* fragment (the macro-eval fence, the
   `is_pure_total` allow-list, already exist as a seed — see the intrinsic-boundary memory).

## The method — lightweight, not a Coq megaproject

Copy ShardStore, not seL4:
1. **A formal reference semantics for eval** — small-step operational semantics.
   (Note: **CEK makes this natural** — the `(C,E,K)` transitions *are* a small-step
   semantics; a recursive tree-walker is far harder to give a clean formal model.
   So verified-eval is most naturally **downstream of the CEK arc**, not before it.
   See `001/CEK-MIGRATION.md`.)
2. **Conformance-check** the real Rust `eval` against the reference under heavy
   property-based testing (the determinism property is trivially PBT-able: run twice,
   assert byte-equal; run on N hosts, assert agreement).
3. **Targeted machine-checked proofs** of the load-bearing invariants only.

## "On itself" — the elegant self-application

Because wat is **homoiconic**, the reference semantics can be **wat-in-wat**
(metacircular) — the encoding of wat's own AST as a wat value is free. So the
conformance check is literally *the Rust `eval` vs. a reference `eval` written in
wat*. **wat checks wat.** Trust still bottoms out honestly, and the doc must say
where: (a) the reference model's own correctness, (b) test coverage, (c) the
Rust-impl-vs-model gap. No magic; named floor. That floor *is* the de Bruijn kernel
— the one small thing you audit by eye.

## Where it would live / sequencing

- After the **arc-251 syntax flip** and the **CEK migration** (CEK gives the clean
  small-step semantics to verify against). v1 metered-eval does not need this; this
  is the *rigor* layer underneath the verification market.
- Reference semantics + conformance harness: a new home (e.g. `wat-rs/src/verify/`
  or a sibling `wat-spec/`); proofs in whatever prover fits (TLA+/Apalache for the
  distributed reproducibility story; a deductive tool for the eval invariants).

## Cross-references

- `THE-COGNITIVE-GODEL.md` (this dir) — the conceptual spine.
- `../001-metered-evaluation/` — the verification market this formally underpins;
  `CEK-MIGRATION.md` (the semantics this verifies against).
- `../002-the-second-system/` — System 1/2; wat as the deterministic System-2 oracle.
- Memory: `project_metered_eval_verification_market`,
  `feedback_does_a_macro_need_it_intrinsic_boundary` (the total-fragment seed),
  `feedback_no_magic_that_lets_llm_fake_correctness` (why the named floor matters).
