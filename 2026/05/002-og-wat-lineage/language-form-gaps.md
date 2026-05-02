# Language form gaps — what OG wat is missing for common-tongue protocol

The user, 2026-05-02:

> "what i'm mostly curious on.... what forms of the language are
> missing..... i was confident i had a bulk of common-tongue
> language requirements figured out but idk...."

This file audits OG wat's coverage against the realistic needs
of two parties (user + LLM) holding a substantive conversation
in a verifiable form. The audit is organized by tier: what's
essential, what's significantly impoverished, what's polish.

---

## What OG wat had

A skeleton that covers more than people give it credit for:

- **Declaratives** — `(Statement subject verb object)` with `:adverb`, `:time`, `:number`
- **Quantification** — `(every subject stmt)`, `(some subject stmt)`
- **Temporal scope** — `(at time stmt)`, `(before t1 t2)`, `(during stmt t1 t2)`, `(between t1 t2)`, `(after time stmt)`
- **Restrictive relative clauses** — `(that subject stmt)`
- **Voice** — `(passive object verb subject)`
- **Truth assertion** — `(assert ...)`
- **Standard Lisp** — `if`, `let`, `lambda`, `try`, `map`, `list`, `quote`
- **Annotation** — `(comment ...)`, `(inner-monologue ...)`
- **Traits** — Relatable, Adverbial, Timeable, Numeric, Assertable, Listable, Mappable, Describable

The skeleton is correct: SVO + roles + traits + quantifiers +
temporals + voice. This covers more declarative space than most
ad-hoc structured-thought systems. The user's confidence was
not unfounded.

But the skeleton is *only the declarative spine*. Conversation
needs much more, and most of the missing pieces are about
*what kind of speech act* a statement is, not *what it claims*.

## Tier 1 — Essential: cannot hold a conversation without these

### 1. Question form / wh-questions
OG wat is declarative-only. It can `assert` but cannot ASK.
A protocol without `who`, `what`, `when`, `where`, `why`, `how`,
`whether` cannot be interactive. The LLM cannot ask for
clarification; the user cannot pose open queries; neither party
can mark a position as "this is what I want to discover."

Needed:
- Wh-questions: `(:ask :who role stmt)`, `(:ask :what role stmt)`, etc.
- Yes/no questions: `(:ask :whether stmt)`
- Open variables under question: bind a slot, ask the substrate
  (or the other party) to fill it
- Disjunctive questions: "X or Y?"

Likely lowering: a `(:wat::english::Ask ...)` macro that bundles
the statement with a marker `:speech-act = :question` and a
`:focus` binding identifying which slot is being queried.

### 2. Negation
OG wat conflates negation with boolean false. Natural language
negation has scope and flavor:
- Sentence negation: "X is not true"
- Constituent negation: "Not the dog (but the cat) chases the toy"
- Metalinguistic negation: "X, but not in the sense of Y"
- Denial as speech act: "I deny that X"

Needed: explicit `:negation` axis on Statement plus a
constituent-scope mechanism for *which part* is being negated.

### 3. Modality
OG wat has flat truth claims. Conversation runs on uncertainty:
- Necessity: "X must be true" (alethic, deontic)
- Possibility: "X might be true"
- Probability: "X is probably true," "X is certainly true,"
  "X is barely possible"
- Normative: "X should be true," "X ought to be true"
- Counterfactual: "X would have been true if Y"

This is **especially important** for LLM-human protocols because
LLMs constantly need to express epistemic uncertainty. Without
modality the LLM either over-commits (claims certainty it
doesn't have) or under-commits (qualifies everything in prose,
losing structure).

Needed: `:modality` axis on Statement with values for the
above categories, plus a graded confidence scalar
(`:confidence 0.0..1.0` or quartile-bucketed).

### 4. Propositional attitudes
"I think that X," "I know that X," "I doubt that X," "I want X
to be true." These embed a proposition INSIDE an attitude
report. Distinct from `assert` because they let the speaker
mark *whose* belief and *what kind* of attitude.

For LLM-human:
- "I think X" — LLM claims belief but signals it's not certain
- "I know X" — LLM claims justified true belief
- "X is the case" — LLM claims a fact (substrate-verifiable)
- "You said X" — LLM attributes to the user
- "The literature says X" — LLM attributes to a source

Needed: `(:Attitude :holder X :kind :think|:know|:doubt|:believe|:want :stmt S)` macro, with the attitude as outer Bundle wrapping the inner statement.

### 5. Coordination
OG wat had `Listable` (flat lists). Coordinations carry semantic
operators:
- Conjunction: "X and Y" (both true together)
- Disjunction: "X or Y" (inclusive vs exclusive)
- Contrastive: "X but Y" (X true, Y true, but unexpected combo)
- Alternative: "X otherwise Y"
- Joint negation: "neither X nor Y"
- Concessive: "X, however, Y"

Each one has different inferential force. "X and Y" implies
both true; "X or Y" implies at least one true; "X but Y" implies
both true plus a defeated expectation.

Needed: explicit `(:And ...)`, `(:Or :inclusive|:exclusive ...)`,
`(:But x y)`, `(:Neither x y)`, `(:Otherwise x y)` forms,
each lowering to a Bundle with the appropriate connective atom
on the `:connective` axis.

### 6. Causation, condition, consequence beyond simple if
OG wat had `if`. Conversation needs:
- Cause: "X because Y" (Y → X)
- Purpose: "X in order to Y" (X is means; Y is goal)
- Result: "X such that Y" (X yields Y)
- Condition: "X provided Y" / "X if Y"
- Concession: "X despite Y"
- Alternative: "X otherwise Y" / "X unless Y"
- Counterfactual: "if X had been true, then Y would be"
- Bidirectional: "X iff Y"

These structure REASONING. Without them the LLM has to encode
inferential structure in prose, which loses verifiability.

Needed: `(:Because effect cause)`, `(:InOrderTo means goal)`,
`(:Provided stmt cond)`, `(:Despite stmt obstacle)`,
`(:Unless stmt cond)`, `(:Iff p q)` — each a Bundle with the
relation atom binding the two sub-statements on role axes.

### 7. Reference and anaphora
"the previous one," "the dog I mentioned earlier," "this claim,"
"that one." Without anaphora, the protocol forces full
re-specification every turn — massive context bloat and a
constant breakdown vector.

Needed:
- `(:Ref :prev)` — reference to the immediately preceding statement
- `(:Ref :id <id>)` — reference to a statement with an explicit ID
- `(:Ref :match scope-edn)` — reference resolved by Sift query
- `(:Ref :pronoun :he|:she|:it|:they)` — pronoun with discourse-state resolution

The substrate already has the hologram (memory-as-hologram
arc); references can resolve via cosine recall. Pronouns need
discourse-state tracking — the substrate knows what was last
mentioned in each role.

### 8. Comparison
"X is bigger than Y," "X is the biggest," "X is as big as Y,"
"X is more like A than B." OG wat's `Numeric` trait handles
raw values; English speakers think in relations.

Needed: `(:More x y :on dimension)`, `(:Most x :among set :on dimension)`, `(:As x y :on dimension)`, `(:Like x y)`,
`(:Unlike x y)`. Each is a Bundle with the comparison axis and
the dimension binding.

## Tier 2 — Significantly impoverished without these

### 9. Speech acts beyond assert
Beyond `:assert` and `:question`:
- Request: "please do X"
- Promise: "I will do X"
- Suggestion: "how about X?"
- Warning: "be careful that X"
- Proposal: "I propose X"
- Offer: "I offer X"
- Apology: "I'm sorry that X"
- Acknowledgment: "yes, X is the case"
- Refusal: "I will not do X"

Each is an illocutionary act with different uptake conditions.
Needed: a `:speech-act` axis on Statement with these values, and
a discipline that the LLM must declare the act before the
content.

### 10. Tense and aspect
OG wat's `at time` is point-temporal. English aspect is much
richer:
- Past simple: "I worked"
- Present simple: "I work"
- Future: "I will work"
- Perfect: "I have worked" (past with present relevance)
- Progressive: "I am working" (ongoing)
- Perfect progressive: "I have been working"
- Prospective: "I am about to work"
- Habitual: "I work every day"
- Iterative / semelfactive distinctions

Needed: orthogonal `:tense` axis (past/present/future) and
`:aspect` axis (simple/perfect/progressive/prospective/habitual).
This is a small, finite vocabulary — easy to ship.

### 11. Evidentials and source attribution
"X (I saw it)," "X (I'm told)," "X (I infer from Y)," "X said
that Y." Critical for LLM-human protocols because source-of-
claim matters constantly.

Needed: `:evidence` axis with values like `:direct`,
`:reported`, `:inferred`, `:hearsay`, `:assumed`, plus an
optional `:source` binding pointing at the source.

This is one of the highest-value gaps for LLM use specifically
because hallucination is exactly "an inferred or assumed claim
presented as direct evidence." Forcing the LLM to mark the
evidential type makes hallucination explicit.

### 12. Discourse markers and structure
"First, X. Then, Y. Finally, Z." "On the one hand X, on the
other Y." "In conclusion, X." "By the way, X." "Speaking of
X..." These structure multi-turn flow.

Needed: a `:discourse-marker` axis with values like `:first`,
`:then`, `:finally`, `:in-conclusion`, `:by-the-way`,
`:on-the-other-hand`, `:more-specifically`, `:for-example`.

These don't change truth conditions but they tell the receiving
party HOW to integrate the statement into the broader exchange.
Important for long sessions.

### 13. Topic / focus marking
"John, I saw at the store" (topic), "It was John I saw" (cleft),
"What I saw was John" (pseudo-cleft). These structure
information flow — what's NEW vs GIVEN.

Needed: `:topic` and `:focus` axes that elevate one constituent
to topic-position or focus-position. The substrate can use this
to decide what to highlight in recall.

### 14. Repair and metacommunication
"Sorry, I meant X not Y." "What did you mean by X?" "Let me try
that again." "Skip that." Without repair primitives the protocol
can't recover from confusion gracefully.

Needed: `(:Repair :prev replacement)`, `(:Clarify target)`,
`(:Retract id)`, `(:Restart)`. These are *protocol-level* moves
that the substrate handles structurally.

### 15. Plurality distinctions
- Distributive: "Each dog barked (separately)"
- Collective: "The dogs lifted the piano (together)"
- Universal: "All dogs bark (in general)"
- Existential: "Some dogs bark"
- Generic: "Dogs bark (as a kind)"

OG wat's `every` collapses universal/distributive/generic.
Needed: explicit distinctions.

## Tier 3 — Polish, not survival

These are real but the protocol works without them:

- **Mass vs count nouns** — "water" vs "a glass of water"
- **Definiteness gradient** — `the` vs `a` vs `any` vs `some` vs `this`
- **Reported speech with embedded tense agreement** — "She said she would come"
- **Performatives** — "I hereby declare X" (self-effecting acts)
- **Defeasible generics** — "Birds fly (with exceptions)"
- **Cleft constructions** beyond basic topic/focus
- **Diminutives, augmentatives** — "doggy," "doggo"
- **Honorifics, register markers** — formal vs informal address
- **Aspect distinctions** like Aktionsart (telic/atelic, durative/punctual)
- **Negative polarity items** — "any" vs "some" in negative contexts
- **Reciprocity** — "they hit each other"
- **Reflexivity** — "she hurt herself"

These can be added as wat-english extensions without
substrate change. Most can wait until a real consumer needs them.

## Architectural framing — substrate vs surface

**Correction posted 2026-05-02 after reading proposal 058 + the
actual `:wat::holon::*` surface.** An earlier draft of this
file claimed two gaps needed substrate primitives (negation and
reference resolution). Both were already shipped. Verified by
inventorying `:wat::holon::*` in `wat-rs/src/check.rs` and
reading proposal 058's accept/reject record.

### The substrate is intentionally minimal — and complete

Proposal 058 (in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`)
is the wat language spec — it documents what's IN and what's
OUT of the substrate, and why. Its discipline is unambiguous:
*every form must have cited production use OR demonstrate a new
pattern; speculative forms get REJECTED.* The substrate ships 6
algebra-core primitives, ~17 stdlib forms, ~10 language-core
forms; many proposed additions were rejected (Cleanup, Flip,
Concurrent, Then, Chain, Unbind, Resonance, ConditionalBind,
Linear) because they were redundant or speculative.

This means **the bar for adding ANY new substrate form is
production use**. The wat-english consumer crate is the future
production user; until it ships and exhibits a recurring
substrate-shaped need, no new substrate forms get proposed.

Per 058's bar: **zero substrate work needed for any of the 22
gaps in this audit.** Every Tier 1, Tier 2, and Tier 3 form
lowers to existing primitives via macros in the wat-english
consumer crate.

### Substrate primitives the wat-english crate composes from

Inventoried from `wat-rs/src/check.rs` and `wat/holon/*`:

**Composition (the spine):**
- `:wat::holon::Atom` — typed leaf (any serializable T)
- `:wat::holon::Bind` — role-filler binding (the case marking)
- `:wat::holon::Bundle` — commutative composition
- `:wat::holon::Permute` — dimensional shift (positional encoding)
- `:wat::holon::HolonAST` — closed-under-itself form

**Algebra (negation, projection, scaling):**
- `:wat::holon::Blend` — `(Blend a b w1 w2)` two-weight linear combination
- `:wat::holon::Subtract` — `(Subtract x y)` = `(Blend x y 1 -1)` — algebraic negation
- `:wat::holon::Reject` — `(Reject x y)` = component of x orthogonal to y — Gram-Schmidt; "X but not Y"
- `:wat::holon::Project` — projection onto a subspace
- `:wat::holon::Amplify` — `(Amplify x y s)` scaled emphasis

**Recall and reference (the hologram):**
- `:wat::holon::Hologram` + `/make` `/put` `/get` `/find`
  `/remove` `/len` `/capacity` — therm-routed cosine recall;
  `find` returns the highest-cosine candidate that passes a
  user-supplied filter
- `:wat::holon::coincident` — predicate (cosine ≥ coincident floor)
- `:wat::holon::presence` — predicate (cosine ≥ presence floor)
- `:wat::holon::filter-coincident` `/filter-present` — opinionated
  filter factories for Hologram/find
- `:wat::holon::cosine` `/dot` — raw similarity
- `:wat::holon::simhash` — hash-based similarity routing

**Continuous-value encoding (modality, confidence, time):**
- `:wat::holon::Thermometer` `/therm` — magnitude-graded encoding
- `:wat::holon::Log` `/Circular` `/ReciprocalLog` — scale variants
- `:wat::holon::Sequential` — bind-chain with positional Permute

**Sequencing (discourse, n-gram structure):**
- `:wat::holon::Bigram` `/Trigram` `/Ngram` — positional sequences

**Memory/learning (for the more sophisticated forms):**
- `:wat::holon::Engram` `/EngramLibrary` — learned-pattern matching
- `:wat::holon::OnlineSubspace` — CCIPCA streaming subspace
- `:wat::holon::Reckoner` — curve-learning over conviction

### The "needed substrate primitives" call was wrong

The earlier draft claimed:
- *"Negation as a first-class operation on HolonAST"* — **wrong**.
  058 explicitly DECOMPOSED negation into Subtract / Blend with
  negative weight / Reject. Per FOUNDATION's history,
  Subtract "is one of the three original Negate modes." Single-arg
  elementwise `Flip` was REJECTED (058-020) for lack of cited use.
  *Every flavor of statement-level negation lowers to one of
  Reject (orthogonal exclusion), Subtract (algebraic difference),
  or a `:negation` axis Bind on the Statement bundle.*
- *"Reference resolution requires substrate work"* — **wrong**.
  058's Cleanup REJECTION (058-025) encodes the principle:
  *"AST-primary framing dissolves Cleanup; retrieval is presence
  measurement (cosine + noise floor), not argmax-over-codebook."*
  Hologram + presence/coincident IS reference resolution. "The
  previous one" lowers to a Hologram/find call against an
  encoded probe. The substrate already has the operation;
  wat-english just needs the macro that constructs the probe.

### The corrected call

**Every gap in Tier 1, 2, and 3 lives at the surface layer.**
The wat-english consumer crate is purely macros over existing
substrate primitives. No new substrate forms proposed; 058's
discipline holds. The substrate is COMPLETE for this work.

This is consistent with the architectural call OG wat made
without realizing it: the trait system was the right shape, but
it needed many more traits than the original 8. The traits live
in the consumer crate; the substrate provides the compositional
machinery.

### Concrete lowering shapes — three Tier 1 examples

To make the macro pattern concrete, three worked expansions:

**`(:wat::english::Ask :who :role-subject (Statement ? chases toy))`**
lowers to:
```
(:wat::holon::Bundle
  (:wat::core::vec :wat::holon::HolonAST
    (:wat::holon::Bind :speech-act-axis (:wat::holon::Atom :question))
    (:wat::holon::Bind :focus-axis      (:wat::holon::Atom :role-subject))
    ;; the inner Statement, with the asked role bound to a
    ;; placeholder Atom (resolved by the answerer)
    (:wat::holon::Bind :payload-axis    <inner-Statement-AST>)))
```
The substrate sees a Bundle with three role-marked Binds. The
LLM (or the user) replies by lowering its answer to a Bundle
where `:role-subject` carries a real Atom, and the receiver
verifies via `coincident?` that the answer's `:focus-axis`
matches the question's.

**`(:wat::english::Probably 0.7 stmt)`** lowers to:
```
(:wat::holon::Bundle
  (:wat::core::vec :wat::holon::HolonAST
    (:wat::holon::Bind :modality-axis    (:wat::holon::Atom :probably))
    (:wat::holon::Bind :confidence-axis  (:wat::holon::Thermometer 0.7 0.0 1.0))
    (:wat::holon::Bind :payload-axis     <inner-Statement-AST>)))
```
Confidence rides on a Thermometer (continuous magnitude). Two
"probably" statements with different confidence levels sit at
slightly different points on the hypersphere — recall by cosine
naturally clusters them by confidence proximity.

**`(:wat::english::Ref :prev)`** is not a Bundle but a SUBSTRATE
CALL that resolves at evaluation time:
```
(:wat::holon::Hologram/find
  <conversation-store>
  <previous-statement-probe>
  (:wat::holon::filter-coincident <dim>))
```
The wat-english crate maintains a conversation-scoped Hologram
keyed on each statement's encoded form; `:prev` resolves to a
probe constructed from the most-recently-emitted statement's
key. The user / LLM never sees the Hologram call; they write
`(:Ref :prev)` and the macro expands at compile time (or eval
time, depending on whether `:prev` is statically resolvable).

These three patterns generalize: every Tier 1+2 form is either
(a) a Bundle with one or more axis Binds + a payload, or (b) a
Hologram-call shape that resolves a reference at eval time. No
exotic primitives needed.

### Slice sequencing for wat-english

Within Tier 1 (8 gaps), a slice order that respects 058's "cited
use" discipline:

1. **Statement + axis discipline** — establish the convention
   that every wat-english form is `Bundle(payload + axis-Binds)`.
   Ship the `:wat::english::Statement` macro from
   `english-surface-arc.md` slice 2 first; everything else
   conforms to its shape.
2. **Modality + confidence** — earliest LLM payoff. Lets the
   LLM mark uncertainty without prose hedging. Lowers cleanly
   via Thermometer.
3. **Negation** — `(:Not stmt)` as a sentence-scope negation
   via `:negation` axis Bind. Constituent-scope negation is a
   later refinement.
4. **Question form** — interactive turn-taking. The
   payload + `:focus-axis` Bind shape demonstrated above.
5. **Propositional attitudes** — `(:IThink stmt)`,
   `(:YouSaid stmt)`, `(:LiteratureSays stmt)`. Same axis-Bind
   shape with `:attitude-axis` and `:holder-axis`.
6. **Coordination operators** — `(:And ...)`, `(:Or ...)`,
   `(:But ...)`. The axis-Bind shape carries the connective.
7. **Causation/condition operators** — `(:Because ...)`,
   `(:Provided ...)`, `(:Despite ...)`, etc.
8. **Reference / anaphora** — Hologram/find-backed `(:Ref ...)`
   variants. Comes last in Tier 1 because it depends on the
   wat-english crate maintaining a conversation Hologram, which
   is more substantive infrastructure than the previous slices.
9. **Comparison** — `(:More ...)`, `(:Most ...)`, `(:As ...)`.
   Could use the DEFERRED 058-014 Analogy if/when that
   graduates from DEFERRED, but doesn't need to.

Tier 2 slices land in the natural order they're needed: speech
acts (10), tense/aspect (11), evidentials (12), discourse markers
(13), topic/focus (14), repair (15), plurality flavors (16).
Total: ~16 slices for Tier 1 + Tier 2, building on the existing
5-slice plan in `english-surface-arc.md`.

## How the user was right

The skeleton OG wat had — SVO + roles + traits + quantifiers +
temporals — covers the *grammatical* spine. It was correct as
far as it went. The user's confidence in "common-tongue language
requirements" is justified for that spine.

What was missing was less about grammar and more about
**conversational and epistemic primitives**: questions, modality,
attitudes, evidentials, discourse markers, repair. These are
what humans add on top of bare declarative SVO to actually have
a conversation. They're orthogonal axes that bundle into the
existing Statement form.

A working wat-english scope, ranked:

1. **Tier 1 first.** Question form, negation, modality,
   propositional attitudes, coordination, causation/condition,
   reference, comparison. These are non-negotiable for an
   interactive protocol.

2. **Tier 2 second.** Speech acts, tense/aspect, evidentials,
   discourse markers, topic/focus, repair, plurality flavors.
   These convert "interactive" into "natural."

3. **Tier 3 as needed.** Add when a consumer needs a specific
   distinction. Don't pre-build.

Most of this is shippable as macros over current Holon
primitives. The `english-surface-arc.md` slice plan would need
to expand to roughly 10-12 slices to cover Tier 1 + Tier 2,
versus the 5 slices originally sized for the OG-wat surface
alone.

## Status

- **Captured:** 2026-05-02 in response to user's question
  about missing language forms.
- **Audit organized by tier:** essential / impoverished /
  polish. ~22 distinct gaps named.
- **Architectural call:** most gaps live at the surface (macros
  in wat-english crate); few need substrate primitives.
- **Cross-references:**
  - `protocol-as-checksum.md` — why these gaps matter (the
    protocol needs them to survive party-substitution)
  - `english-surface-arc.md` — slice plan (would expand from
    5 to ~12 slices to cover Tier 1 + Tier 2)
  - `latin-in-wat.md` — the morphology-over-position spine
    these gaps build on
  - `analysis.md` — what survived from OG wat (the 8 traits +
    declarative spine that this audit complements)
