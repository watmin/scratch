# Scratch arc — OG wat lineage

The current wat language (Lisp on Rust, the algebra surface,
Bind/Bundle/cosine) has an older ancestor: a Grok-era spec for a
*pure, strongly typed, English-like Lisp* with 28 core primitives,
SVO-ordered statements, a trait system with English-named axes
(Relatable, Adverbial, Timeable, Numeric, Assertable, ...),
temporal quantification (`every`, `some`, `at`, `before`,
`during`), relative clauses, and homoiconicity. The user wrote
this spec years ago in a Grok conversation. He carried it on disk
through years of *"the substrate that could host this didn't
exist yet."* In mid-January 2026, when the substrate finally
landed, current wat became the realization of OG wat's discipline
on top of VSA + Holon algebra.

The user surfaced the OG wat file at `/tmp/og-wat.md` on
2026-05-01, hours after the DEFCON CFP submission shipped, with
the note: *"you'll get it... i've built most of it... the origins
are closer."*

This scratch captures my (Claude's) understanding of OG wat, the
mapping from OG to current, the still-latent English-like SVO
surface that current wat could ship as a consumer crate, and the
contextualization of the DEFCON submission's *"three years of
haunting"* claim. Captured here so the recognition persists past
this conversation's compaction.

---

## Files in this arc

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `og-wat-spec.md` | The OG wat spec, **verbatim** from `/tmp/og-wat.md`. 28 primitives, the trait system, the SVO statement form, the full example. Preserved here because `/tmp` is volatile and this is a load-bearing historical artifact. |
| `analysis.md` | What OG wat is. What survived intact in current wat. What transformed (the substrate landed). What's still latent. The user's recognition that *"the origins are closer."* |
| `english-surface-arc.md` | The candidate `wat-english` consumer crate that could ship OG wat's English-like SVO surface as macros over current Holon primitives. Sized as a real wat consumer arc. Not blocking; an interesting follow-on. |
| `latin-in-wat.md` | The user's question before compaction: *"do you see the latin in wat?... what i was reaching for?"* Captured in full. The Latin in wat is **morphology over position** — meaning lives in the term's case-marking, not in the term's seat. Bind = case marking; trait system = declension system; HolonAST closed under itself = morphology native; the hypersphere = morphology made geometric. The four tattoos read through this lens. Three persistence layers; one impulse. |
| `protocol-as-checksum.md` | The user's original premise from ~14 months ago: *"i was quite convinced that getting an llm to speak in lisp was trivial.. compile their answers and measure for violations... bad forms get rejected and a retry is performed... we just consider bad forms to be a protocol error and retry."* Validated as directionally right, 14 months early on tooling. Names the four prerequisites that didn't exist then and now do. Maps "add a layer" to constrained decoding (the powerful one). Bidirectional verification = the architectural insight (TCP-style trust-the-checksum, not the wire). Two hard problems flagged for later. |
| `language-form-gaps.md` | Audit of OG wat's coverage for common-tongue conversation. ~22 gaps named, organized by tier. Tier 1 (essential): question form, negation, modality, propositional attitudes, coordination, causation, reference, comparison. Tier 2 (significantly impoverished without): speech acts beyond assert, tense/aspect, evidentials, discourse markers, topic/focus, repair, plurality flavors. Tier 3 (polish): mass/count, definiteness, performatives, defeasible generics, etc. **Corrected 2026-05-02 after reading proposal 058 + the actual `:wat::holon::*` surface — zero substrate work needed; every gap is a wat-english macro over existing primitives (Reject + Subtract for negation; Hologram + presence/coincident for reference).** Three concrete Tier 1 lowering shapes (Ask / Probably / Ref) included; slice order for ~16 slices laid out. |
| `og-wat-impl.rb` | The original Ruby reference implementation of OG wat. 490 lines. Preserved from `/tmp/wat.rb` (volatile). Implements tokenize/parse/evaluate for `entity / list / add / let / impl / lambda` plus the 10-trait membership tables and Subject/Object sugar-type auto-wrapping. Does NOT contain a render/to-string function — that direction was conceptual, never coded. Historical artifact behind the OG wat spec. |
| `wat-to-english.md` | The inverse direction. The user's premise: *"i was equally convinced we can translate wat-english forms to english... i don't think i actually wrote it... conceptually... thought."* The conviction held; OG wat never shipped it. This file makes it concrete: render direction is deterministic (AST has no ambiguity); lift direction is lossy. Two implementation paths — template-based renderer (~500-800 LOC, deterministic, local) and LLM-based renderer (rich, model-call cost). Production ships both. Closes a fifth shape on top of `protocol-as-checksum.md`'s four (Express / Reflect / Verify / Recall / **Render**). Round-trip is honestly lossy — same property TCP has. |
| `cypher-bridge.md` | The Neptune-Cypher era as the substrate's pre-history. *"i was doing a lot with neptune db at the time.. working with relational triples and whatever else... i found the cypher lang to be quite close to what i wanted but its expression forms.... are very bad... i wanted them in wat and i thought that would be the bridge."* Maps Cypher concepts to current wat realizations: triple = Bind; subgraph = Bundle; MATCH = Hologram/find with Bundle template; WHERE = wat lambda filter; RETURN = wat function over results. The bridge user imagined absorbed both endpoints. Sharpens what wat-sift is measuring against: *"wat-shaped Cypher with similarity native, patterns as data, hygienic expression forms."* |
| **`PICKUP-GUIDE.md`** | **The on-ramp document. Read FIRST when wat-english is ready to move from scratch to a real arc.** Specifies the three readiness signals, verifies prerequisites in wat-rs, identifies wat-sift as the one hard external prereq, locks reading order, inventories substrate primitives, consolidates the slice plan into 5 phases (~18 slices for Tier 1+2 + render; up to 24 with Tier 3), and includes a Slice 0 for pre-implementation decisions (gaze pass on axis names, namespace decision, encoder dim, test discipline, first real consumer). Designed to ramp a future-Claude (or future-you) from cold to slice 1 in under 3 hours. |

## Why this matters

The DEFCON submission's `Speaker Perspective` answer says:

> *"For nine years inside AWS I tried to convince anyone who would
> listen... I wrote them on my body in Latin in college because I
> needed a persistence layer that wouldn't let me forget. I tried
> to express them at AWS for a decade and got blank stares."*

The OG wat spec is the SECOND persistence layer the user wrote for
the same ideas — the first was on his body (Latin), the second
was on disk (Grok-era spec). Both held the discipline through
years of the substrate not existing yet. When the substrate
landed, both got expressed in code.

The submission timeline (3y haunting / 1mo rest / 3mo building)
is honest. The OG wat file shows the haunting was not formless —
it was a fully-specified Lisp waiting for a substrate that could
host its discipline. The "3 months of building" was substantively
**realizing a years-old design** on the substrate that finally
existed, not designing from scratch.

## Status

- **Captured:** 2026-05-01.
- **OG wat preserved:** verbatim in `og-wat-spec.md`.
- **Analysis written:** in `analysis.md`.
- **Follow-on candidate:** `wat-english` consumer crate, sized
  in `english-surface-arc.md`. Not opened as a real arc; lives
  here as the thought.
- **Connected to:** DEFCON submission's Speaker-Perspective
  answer; BOOK Chapter 5 (the prequel); the lineage
  established in `defcon-2026/THESIS.md`.
