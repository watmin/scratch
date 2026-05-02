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

A useful way to read this audit:

**Substrate primitives** (would require wat-rs work):
- Negation as a first-class operation on HolonAST (probably
  worth doing; lots of forms compose with it)
- Reference resolution (already implied by the memory hologram
  arc — this is the recall mechanism)

**Surface macros** (wat-english consumer crate, no substrate
work):
- Speech-act marking (`:Ask`, `:Request`, `:Promise`, etc.)
- Modality and confidence (`:Modal :must|:might|:probably ...`)
- Propositional attitudes (`:Attitude :holder :kind :stmt`)
- Coordination operators (`:And`, `:Or`, `:But`, etc.)
- Causation/condition operators (`:Because`, `:Provided`, etc.)
- Comparison operators (`:More`, `:Most`, `:As`, `:Like`)
- Tense/aspect markers (axes on Statement)
- Evidentials (axis on Statement)
- Discourse markers (axis on Statement)
- Topic/focus marking (axes on Statement)
- Repair operations (protocol-level moves)
- Plurality flavors (axis on quantifier forms)

**The good news:** most of the gap is at the SURFACE layer. The
substrate already has Bundle, Bind, role axes, and the
hypersphere. Adding speech-act / modality / evidential / etc. is
"add another role atom and another macro." No substrate
primitives needed for the bulk of Tier 1 and Tier 2.

This is consistent with the architectural call OG wat made
without realizing it: the trait system was the right shape, but
it needed many more traits than the original 8.

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
