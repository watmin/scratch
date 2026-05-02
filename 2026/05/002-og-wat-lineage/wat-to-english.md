# wat-to-English — the inverse direction

The user, 2026-05-02:

> "i was equally convinced we can translate wat-english forms to
> english... there's a to-string impl here... i was working on
> this like over a year ago..."
>
> [moments later, correction]
>
> "i don't think i actually wrote it... conceptually... thought..."

The conviction held; the implementation didn't ship. The
conceptual premise: wat-english forms render back to natural
English mechanically, so humans can read what was said without
learning the form.

This file captures why that conviction was correct, how the
inverse direction maps onto the protocol, and the two
implementation paths available now.

---

## The direction asymmetry

The conversation has two directions:

- **Lift** — English → wat-english. *Lossy.* English is
  positionally ambiguous; the lift has to make judgment calls
  (what's the subject, what's a clause boundary, what counts as
  one statement). This is `protocol-as-checksum.md`'s "Hard
  problem 1."

- **Render** — wat-english → English. *Deterministic.* The
  AST is unambiguous; every Bind names its role; every axis
  declares its kind. The renderer walks the form and emits
  English by template. No judgment calls; no information loss
  in the substrate-to-prose direction.

The user's two-direction conviction was structurally correct.
The direction they thought was easier IS easier. Building the
lift is research; building the render is engineering.

## Why render matters

Three audiences need to read wat-english that aren't going to
learn the form:

1. **Humans without wat priors.** Stakeholders, future
   collaborators, the DEFCON board, anyone reading the artifacts
   later. They should see "The dog chases the toy at t-0,"
   not `(:Statement (:Subject "dog") (:Verb "chases") (:Object "toy") :time (:Time "t-0"))`.

2. **Other LLMs that don't speak wat.** A weaker model in the
   loop, or a model fine-tuned on a different protocol, can
   consume rendered English even if it can't emit valid
   wat-english itself. Asymmetric partnership becomes possible.

3. **Future-self.** When you re-read your own arcs in 6 months
   without wat in working memory, the rendered prose is the
   skim layer. The wat-english AST stays the durable record;
   the rendered English is the human-readable index.

## Two implementation paths

Both are tractable today. Ship both.

### Path 1 — Template-based renderer (mechanical)

Walk the wat-english AST; for each form, emit the English
realization per template. Deterministic, runs locally, no model
in the loop.

The structural work:

- **Speech act** picks the verb mood and word order:
  - `:assert` → declarative ("X happens")
  - `:question` → interrogative ("Does X happen?", "Who does X?")
  - `:request` → imperative ("Please do X")
  - `:promise` → first-person future ("I will do X")
- **Modality** prepends/inserts hedges:
  - `:must` → "X must happen"
  - `:might` → "X might happen"
  - `:probably` → "X probably happens"
  - `:certainly` → "X certainly happens"
- **Confidence** (Thermometer scalar) maps to lexical hedges
  via threshold buckets:
  - `[0.0, 0.3]` → "barely" / "unlikely"
  - `[0.3, 0.6]` → "possibly" / "maybe"
  - `[0.6, 0.85]` → "probably" / "likely"
  - `[0.85, 1.0]` → "almost certainly" / "very likely"
- **Tense / aspect** drives verb conjugation. Small finite
  table: tense × aspect × person × number × verb-class →
  conjugated form. ~200 entries cover English's mainstream.
- **Evidentials** suffix the rendered statement:
  - `:direct` → "(I saw)"
  - `:reported` → "(I heard)"
  - `:inferred` → "(inferring)"
  - `:source X` → "(per X)"
- **Discourse markers** prepend:
  - `:first` → "First, X"
  - `:then` → "Then, X"
  - `:by-the-way` → "By the way, X"
  - `:on-the-other-hand` → "On the other hand, X"
- **Coordination operators** join sub-statements:
  - `:And` → "X and Y"
  - `:But` → "X, but Y"
  - `:Or :inclusive` → "X or Y (or both)"
  - `:Or :exclusive` → "X or Y (but not both)"
- **Causation operators** insert subordinators:
  - `:Because` → "X because Y"
  - `:Provided` → "X if Y"
  - `:Despite` → "X despite Y"
  - `:Unless` → "X unless Y"
- **Reference / anaphora** resolves via the conversation
  Hologram and substitutes pronouns or definite NPs:
  - `(:Ref :prev)` → "the previous claim" / "this"
  - `(:Ref :pronoun :he)` → "he" (with discourse-state
    resolution to the most-recently-mentioned masculine
    referent)
- **Statement** body emits the SVO shape:
  - `(:Statement (:Subject S) (:Verb V) (:Object O) :adverb A :time T)`
  - → "[determiner] S V [determiner] O [adverbially] [at T]"

The renderer is recursive: nested forms render bottom-up; outer
forms wrap the inner rendering.

Size estimate: ~500-800 lines of Rust (or ~200-300 lines of
wat) for the core renderer covering Tier 1 + Tier 2 forms.
English realization tables are larger (verb conjugations,
article rules) but bounded — there are finite English mainstream
patterns and the renderer can punt to "raw form name" for
anything outside the table.

Ships as `:wat::english::render` — takes a wat-english HolonAST,
returns a String.

### Path 2 — LLM-based renderer (contextual)

Just call eval on the AST and ask the LLM to render it as
natural English. Pattern:

```
prompt: "Render this wat-english AST as natural English prose.
Keep the meaning exact; do not add information."
input:  <wat-english-AST as EDN>
output: <English prose>
```

Frontier LLMs do this trivially. Non-deterministic (different
model invocations may phrase differently), requires a model
call, but produces richer prose than templates can — pronoun
choices feel natural, register matches the surrounding
discourse, edge cases that templates would punt on get handled
gracefully.

Use cases:
- High-stakes communication (e.g., DEFCON talk slides) where
  prose quality matters
- Anywhere the wat-english AST will be quoted in a writeup
- Translation to non-English languages (templates would need
  per-language tables; LLMs handle this naturally)

Cost: a model call per render. For batch renders or low-stakes
output, the template renderer is cheaper.

### Both, in production

Default to template for the common path (logs, debug output,
quick reads). Fall back to LLM for prose-quality contexts. The
substrate ships `:wat::english::render` (template) and a wrapper
`:wat::english::render-rich` that calls out to MCP-mediated LLM
rendering when configured.

## Round-trip honesty

The render direction is deterministic, but **round-trip is
not lossless**:

- English → wat-english → English produces *different* English
  than the original input, because the lift made disambiguating
  choices.
- The wat-english AST is the canonical form; both Englishes are
  views of it.
- This is fine if the user understands the AST is the source of
  truth and English is a derived projection.

Example:
- Input: "The dog might be barking."
- Lift: `(:Probably 0.5 (:Statement dog (:ProgressiveAspect bark)))`
- Render: "The dog is probably barking."

Same proposition; different surface. Both are correct
realizations of the same AST. The information that was lost
("might" vs "probably") was the lift's choice — it pinned a
modal slot to a confidence value. If precision matters, the
user reviews the lifted AST before committing.

This is the same property TCP has: the wire format normalizes
the input; you don't get back the exact bytes you sent, you get
the canonical representation. For a thinking-substrate, that's
the right behavior — the AST is the durable record; the prose
is incidental.

## How this completes the protocol

`protocol-as-checksum.md` named four shapes the protocol needs:
Express, Reflect, Verify, Recall. Render fills out a fifth that
was implicit:

- **Express** — user emits a thought (wat-english or English-
  via-lift)
- **Reflect** — LLM emits a thought (wat-english)
- **Verify** — substrate type-checks (`:wat::core::eval`)
- **Recall** — Sift / Hologram retrieves prior thoughts
- **Render** — wat-english surfaces back to English for
  audiences that don't read the form

The five shapes form a closed loop. Both parties can
participate in any of them. The substrate handles Verify and
Recall; the LLM handles Reflect (and optionally Render-rich);
the user handles Express (with optional LLM assist for the
lift); template renderer handles Render-default.

## Connection to OG wat's Ruby impl

The `og-wat-impl.rb` preserved alongside this file is the
original Ruby reference implementation of OG wat — 490 lines,
implements tokenize / parse / evaluate for entity / list / add
/ let / impl / lambda. It defines the Entity struct, the trait
membership tables (Relatable / RelatableVerb / Adverbial /
Timeable / StringValued / Numeric / Assertable / Listable /
Mappable / Describable), and the sugar-type auto-wrapping
(Subject / Object → Noun + role attr).

What it does NOT contain: a render / to-string / describe
function. The user's recollection is correct — the renderer
was conceptual, never coded. The Ruby implements the parser
and evaluator; the inverse direction was on the roadmap.

This file picks up that thread and makes the inverse direction
concrete enough to ship today.

## Status

- **Captured:** 2026-05-02 in response to user's recognition
  that wat-english → English was a conviction without
  implementation.
- **Direction asymmetry named:** lift (English → wat) is
  research; render (wat → English) is engineering.
- **Two implementation paths sized:** template (deterministic,
  local, ~500-800 LOC) and LLM (rich, model-call cost).
  Production ships both.
- **Five-shape protocol completed:** Express / Reflect /
  Verify / Recall / Render. The fifth shape closes the loop
  for non-wat-fluent audiences.
- **Cross-references:**
  - `og-wat-impl.rb` — the historical Ruby reference (parser +
    evaluator, no renderer)
  - `protocol-as-checksum.md` — the four-shape protocol; this
    file adds the fifth
  - `language-form-gaps.md` — the gaps the renderer would need
    templates for (Tier 1 + Tier 2 macros define the renderer's
    scope)
  - `english-surface-arc.md` — the consumer crate where
    `:wat::english::render` lands as a slice
