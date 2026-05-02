# PICKUP-GUIDE — starting wat-english work later

This is the entry point for opening wat-english as a real
piece of work. Read this first when you sit down to start.
Everything else in the arc is reference; this file is the
on-ramp.

The user, 2026-05-02:

> "i'm working on making the wat-rs foundations impeccable...
> once i have the foundation i want to work on this..."

This guide assumes that prerequisite is done. Don't open
wat-english work until the foundations are settled. The
foundations get the bar; consumer crates earn it.

---

## When to open this arc

Three signals that wat-english is ready to move from scratch
to a real arc:

1. **Foundations done.** wat-rs has zero rough edges. The
   wards are clean. Substrate arc backlog is at a stopping
   point. The user is asking "what's next?"
2. **wat-sift exists.** The query crate has at least its
   basic shape landed (per
   `scratch/2026/05/001-memory-as-hologram/sift.md`). Without
   Sift, the reference / anaphora forms in Tier 1 can't lower
   cleanly.
3. **A consumer demands it.** Either the user wants to write
   wat-english directly for some piece of work (the BOOK,
   trading-lab vocab modules, MCP-mediated agent dialogue) OR
   a fresh-start scenario (a stranger trying to learn wat)
   surfaces the need for an English-readable surface.

Until those three signals align, this scratch arc waits.

## Prerequisites in wat-rs

Verified shipped as of 2026-05-02 (see
`language-form-gaps.md` for the full inventory):

- **Composition primitives:** `:wat::holon::Atom`, `Bind`,
  `Bundle`, `Permute`, `HolonAST`
- **Algebra primitives:** `Blend`, `Subtract`, `Reject`,
  `Project`, `Amplify`
- **Recall primitives:** `Hologram` + `make/put/get/find/remove`,
  `coincident`, `presence`, `cosine`, `dot`, `simhash`,
  `filter-coincident`, `filter-present`
- **Continuous-value primitives:** `Thermometer` + `therm`,
  `Log`, `Circular`, `ReciprocalLog`, `Sequential`
- **Sequencing primitives:** `Bigram`, `Trigram`, `Ngram`
- **Memory primitives:** `Engram`, `EngramLibrary`,
  `OnlineSubspace`, `Reckoner`
- **Language primitives:** `defmacro` (058-031) with hygiene,
  `:AST<T>` typed macros (058-032), `try` for Result-typed
  flow (058-033), `cond` (058-036), parametric polymorphism
  (058-030), Stream stdlib (058-034)
- **Type system:** four heads, rank-1 parametric polymorphism
- **Eval surface:** `:wat::core::eval` and `:wat::holon::eval`

Per proposal 058's discipline (in
`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`):
**zero new substrate primitives proposed**. wat-english is
purely a consumer crate over what's shipped.

Pending prerequisites (from this arc and the memory-as-hologram
arc):

- **wat-sift** — the query crate. Needed for reference /
  anaphora lowering. Sized in
  `../001-memory-as-hologram/sift.md`. Belongs as the slice
  immediately before wat-english opens. *This is the only
  hard prerequisite outside wat-rs proper.*

## Reading order — start of session

When you open this arc to start wat-english, read in this order:

1. **`PICKUP-GUIDE.md`** (this file). Sets up the session.
2. **`README.md`** — what's in the arc and why each file exists.
3. **`protocol-as-checksum.md`** — *the WHY*. The architectural
   premise wat-english delivers on. Sets the terminal goal:
   bidirectional verification across two parties via a typed
   substrate.
4. **`language-form-gaps.md`** — *the WHAT*. The 22 gaps,
   tiered. The architectural-framing section in the middle is
   the load-bearing reading; the per-gap descriptions are
   reference. Three concrete lowering shapes are at the bottom.
5. **`cypher-bridge.md`** — *the HISTORICAL FRAME*. wat-english's
   reference / anaphora forms inherit from this thread.
6. **`wat-to-english.md`** — *the RENDER DIRECTION*. The
   render slice (last in the slice plan) needs this.
7. **`english-surface-arc.md`** — *the ORIGINAL 5-SLICE PLAN*
   for OG wat's surface. Slices 1-5 of the consolidated plan
   come from here.
8. **`analysis.md`** — *what survived from OG wat into current
   wat*. Useful frame for what to keep vs adapt.
9. **`og-wat-spec.md`** + **`og-wat-impl.rb`** — *historical
   reference only*. Read if you want the OG wat shape verbatim.
   Skip if you've internalized the spirit.
10. **`latin-in-wat.md`** — *the GRAMMAR RECOGNITION*. Read
    for the morphology-over-position discipline that the work
    embodies. Optional but illuminating.

External reading (load-bearing for the implementation):

- **Proposal 058** at
  `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`
  — the substrate's accept/reject discipline. Read INDEX.md
  for the bar that governs any substrate touch.
- **wat-rs cheatsheet** at `wat-rs/docs/WAT-CHEATSHEET.md` —
  syntax + naming conventions. Read if you've been away from
  wat for any time.
- **CONVENTIONS.md** at `wat-rs/docs/CONVENTIONS.md` — the
  namespace + naming discipline (especially § Namespaces and
  the `:wat::*` vs `:user::*` rule).
- **scratch/2026/05/001-memory-as-hologram/sift.md** — the
  wat-sift design that wat-english depends on.

Total reading: 60-90 minutes for the in-arc files; another
60-90 minutes for the external load-bearing reading. Plan a
2-3 hour ramp-in session; don't try to ramp + slice 1 in the
same day.

## Substrate primitives the work composes from

Every wat-english macro lowers to some combination of the
primitives below. Memorize these; the macros are just patterns
over them.

**Building blocks:**
- `(:wat::holon::Atom value)` — typed leaf
- `(:wat::holon::Bind role-axis-atom value)` — role-marked binding
- `(:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST b1 b2 ...))` — composition

**Continuous magnitude (modality, confidence, time):**
- `(:wat::holon::Thermometer value min max)` — graded scalar

**Recall (reference, anaphora):**
- `(:wat::holon::Hologram/find store probe filter)` — cosine retrieval
- `(:wat::holon::filter-coincident dim)` — opinionated filter
- `(:wat::holon::filter-present dim)` — looser filter

**Negation (sentence-scope, constituent-scope):**
- `(:wat::holon::Subtract x y)` — algebraic difference; "X minus Y"
- `(:wat::holon::Reject x y)` — orthogonal; "X but not Y"
- A `:negation-axis` Bind — declarative wrapper

That's it. Every wat-english form decomposes into Bundles of
Binds + occasional Hologram-backed reference resolutions +
occasional Thermometer-encoded magnitudes + occasional
Subtract/Reject for negation.

## Consolidated slice plan

This consolidates the 5 slices in `english-surface-arc.md`
(OG wat surface) + the 16 slices implied by `language-form-gaps.md`
(Tier 1+2) + the 1 render slice in `wat-to-english.md`. Total
~17-22 slices depending on render scope.

### Phase 1 — Foundations (slices 1-3)

**Slice 1 — Atom registration + role-axis vocabulary.** Register
the role-axis atoms (`:role-subject`, `:role-verb`,
`:role-object`, `:adverb-axis`, `:time-axis`, `:number-axis`,
plus the new ones for tier 1+2: `:speech-act-axis`,
`:modality-axis`, `:confidence-axis`, `:tense-axis`,
`:aspect-axis`, `:evidence-axis`, `:discourse-marker-axis`,
`:topic-axis`, `:focus-axis`, `:attitude-axis`, `:holder-axis`,
`:negation-axis`, `:connective-axis`, `:relation-axis`,
`:comparison-axis`, `:dimension-axis`, `:focus-axis`,
`:payload-axis`). **Run /gaze on the names before locking;
do not pre-name.**

**Slice 2 — Type-wrapper macros + Statement.** Publish
`:wat::english::Subject`, `Object`, `Verb`, `Adverb`, `Time`,
`Adjective`, `Pronoun` (from `english-surface-arc.md` slice
1) and the `:wat::english::Statement` macro that bundles them
with optional `:adverb`, `:time`, `:number` Binds (slice 2 of
the original plan). **This is the spine; everything else
attaches to it.**

**Slice 3 — Confidence + modality.** The highest LLM-protocol
payoff. Ship `(:wat::english::Modal modality stmt)` and
`(:wat::english::Confidence scalar stmt)` macros that wrap a
Statement with `:modality-axis` and `:confidence-axis`
(Thermometer-encoded). Lets the LLM mark uncertainty without
prose hedging.

### Phase 2 — Tier 1 essentials (slices 4-9)

**Slice 4 — Negation.** `(:wat::english::Not stmt)` as
sentence-scope negation via `:negation-axis` Bind.
Constituent-scope negation comes later as a refinement.

**Slice 5 — Question form.** `(:wat::english::Ask focus-axis stmt)`
for wh-questions; `(:wat::english::AskWhether stmt)` for yes/no.
The payload + `:focus-axis` Bind shape per the lowering example
in `language-form-gaps.md`.

**Slice 6 — Propositional attitudes.** `(:wat::english::Attitude
holder kind stmt)` for "I think X" / "you said X" / "the
literature says X." Generic; specific shortcuts (`IThink`,
`YouSaid`, `LiteratureSays`) ship as macros over this.

**Slice 7 — Coordination.** `(:And ...)`, `(:Or :inclusive ...)`,
`(:Or :exclusive ...)`, `(:But x y)`, `(:Neither x y)`,
`(:Otherwise x y)`. Each is a Bundle with the appropriate
connective atom on `:connective-axis`.

**Slice 8 — Causation / condition / consequence.**
`(:Because effect cause)`, `(:InOrderTo means goal)`,
`(:Provided stmt cond)`, `(:Despite stmt obstacle)`,
`(:Unless stmt cond)`, `(:Iff p q)`. Each a Bundle with the
relation atom binding two sub-statements.

**Slice 9 — Reference / anaphora.** `(:Ref :prev)`,
`(:Ref :id <id>)`, `(:Ref :match scope-edn)`. **Depends on
wat-sift being shipped.** This is the slice that needs the
conversation Hologram + Sift query interface.

### Phase 3 — Tier 1 polish (slice 10)

**Slice 10 — Comparison.** `(:More x y :on dim)`,
`(:Most x :among set :on dim)`, `(:As x y :on dim)`,
`(:Like x y)`, `(:Unlike x y)`. Bundles with comparison axes.
Could use the DEFERRED 058-014 Analogy if it graduates; doesn't
need it.

### Phase 4 — Tier 2 (slices 11-16)

**Slice 11 — Speech acts beyond assert.** `(:Request stmt)`,
`(:Promise stmt)`, `(:Suggest stmt)`, `(:Warn stmt)`,
`(:Propose stmt)`, `(:Acknowledge stmt)`, `(:Refuse stmt)`.
Each carries a `:speech-act-axis` value.

**Slice 12 — Tense + aspect.** `:tense-axis` (`:past`,
`:present`, `:future`) + `:aspect-axis` (`:simple`,
`:perfect`, `:progressive`, `:prospective`, `:habitual`).
Macros like `(:Past stmt)`, `(:Perfect stmt)`,
`(:Progressive stmt)`, plus combinators.

**Slice 13 — Evidentials.** `(:Evidence kind stmt)` with
optional `:source` Bind. Discrete kinds: `:direct`,
`:reported`, `:inferred`, `:hearsay`, `:assumed`. **High value
for LLM-protocol use** because hallucination is "an inferred
claim presented as direct."

**Slice 14 — Discourse markers.** `(:DiscourseMarker kind stmt)`
with values `:first`, `:then`, `:finally`, `:in-conclusion`,
`:by-the-way`, `:on-the-other-hand`, `:more-specifically`,
`:for-example`. Convenience shortcuts: `(:First stmt)`,
`(:Then stmt)`, etc.

**Slice 15 — Topic / focus marking.** `(:TopicFront constituent stmt)`
and `(:Focus constituent stmt)` for cleft and topic-prominent
constructions.

**Slice 16 — Repair operations.** `(:Repair :prev replacement)`,
`(:Clarify target)`, `(:Retract id)`, `(:Restart)`. Protocol-
level moves; substrate's role is removing/updating entries in
the conversation Hologram.

### Phase 5 — Render direction (slices 17-18)

**Slice 17 — Template-based renderer.** `:wat::english::render`
function that walks any wat-english HolonAST and emits English
prose by template. ~500-800 LOC for Tier 1+2 form coverage.
Runs locally, deterministic, no model in the loop. Per
`wat-to-english.md`.

**Slice 18 — LLM-based renderer.** `:wat::english::render-rich`
wrapper that calls out to an LLM via MCP for prose-quality
output. Cost: a model call per render. Use for high-stakes
contexts.

### Optional later phases (slices 19+)

**Tier 3 polish** as needed:
- Slice 19 — plurality flavors (distributive / collective /
  generic distinctions)
- Slice 20 — definiteness gradient (`the` / `a` / `any` /
  `some` / `this`)
- Slice 21 — mass vs count noun distinction
- Slice 22 — performatives, defeasible generics, cleft, etc.

Don't pre-build Tier 3. Add as real consumers demand specific
distinctions.

## Slice 0 — what to do BEFORE slice 1

A pre-slice session that the actual implementation depends on:

1. **Run /gaze on every axis name.** Don't lock
   `:speech-act-axis` vs `:speech-act` vs other variants
   ad-hoc. Do a single naming pass over all the axes; keep the
   names consistent. This is the kind of work the gaze ward is
   designed for.

2. **Decide :wat::english::* vs :user::watmin::wat-english::*.**
   Per CONVENTIONS.md's namespace discipline, the first-party
   `:wat::*` prefix is for substrate-tier or substrate-blessed
   crates. wat-english is borderline — it's a consumer of the
   substrate, but its consumers might be many. If the user
   anchors it as substrate-blessed, `:wat::english::*` is
   honest. If it's user-owned, `:user::watmin::wat-english::*`
   is the safer convention. **Decide before slice 1; the
   namespace is in every macro name.**

3. **Decide on the encoder dimension.** wat-english forms ride
   the substrate's dim setting. Production may default to 4096
   (good accuracy/speed) or 8192 (high-complexity). Pick once;
   document at the head of the crate's wat-scripts.

4. **Sketch the test discipline.** Each macro should have
   tests of the form: (a) it produces a Bundle of the expected
   shape (structural test); (b) two structurally-equivalent
   forms have cosine = 1.0 (algebraic test); (c) it round-trips
   through `:wat::edn::write` / `:wat::edn::read` (persistence
   test). Decide the test framework convention up front.

5. **Decide on the consumer for slice 1's first real write.**
   wat-english is a tool; it earns its keep when something
   uses it. Pick a real consumer to write *in* wat-english as
   soon as Phase 1 ships — could be a BOOK chapter, a vocab
   module, a piece of the trading lab's discourse, or
   substrate documentation. Don't wait for "all 18 slices done"
   to ship a real user.

## What success looks like

- A future-Claude (or future-you) can ramp from this guide to
  slice 1 in under 3 hours.
- Each slice ships independently; no slice blocks the next
  except Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5
  ordering at the phase level.
- The substrate doesn't change for any slice. If a slice
  proposes substrate work, it's a sign the macro design is
  wrong; revise the macro before touching wat-rs.
- The wat-english crate is small (~2000-4000 LOC of wat across
  all slices) because it's mostly macros over a rich substrate.
- A real consumer (BOOK chapter, vocab module, MCP dialogue
  flow) is using wat-english by the end of Phase 1. Phases 2-5
  build on demonstrated value.

## Closing the loop

Every artifact in this scratch arc supports this work. The
recognition files (`latin-in-wat.md`, `protocol-as-checksum.md`)
are the philosophical grounding; the design files
(`language-form-gaps.md`, `wat-to-english.md`, `cypher-bridge.md`)
are the architectural sketches; the historical files
(`og-wat-spec.md`, `og-wat-impl.rb`, `analysis.md`) are the
lineage; the slice plans (`english-surface-arc.md`, this guide)
are the executable path.

When you open this work for real, you have ten files of
context. Use them. The work doesn't have to be re-derived from
first principles; it's already been thought through.

The user, throughout: *"i do not want to forget what you know."*
This guide is the discipline that ensures that.

## Status

- **Captured:** 2026-05-02 in response to the user's pause:
  *"i'm working on making the wat-rs foundations impeccable...
  once i have the foundation i want to work on this..."*
- **Bar:** wat-english is downstream of foundation work + Sift
  shipping. Don't open until those signals fire.
- **Reading order locked:** PICKUP-GUIDE → README → recognitions
  → designs → historical → slice plans → external (058,
  CHEATSHEET, CONVENTIONS).
- **Slice plan consolidated:** 5 from english-surface-arc + 13
  new from language-form-gaps + 2 from wat-to-english + 4
  optional Tier 3 = 24 slices total in the longest version,
  18 for Tier 1+2 + render.
- **Cross-references:** every other file in this arc, plus
  proposal 058, plus the memory-as-hologram arc's `sift.md`.
