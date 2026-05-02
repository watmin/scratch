# wat-to-English — the inverse direction

The user, 2026-05-02:

> "i was equally convinced we can translate wat-english forms to
> english... there's a to-string impl here... i was working on
> this like over a year ago..."
>
> [moments later, correction]
>
> "i don't think i actually wrote it... conceptually... thought..."

Then, after I drafted a layered template-based renderer that
demoted the actual answer to "Layer 3 / fallback":

> "you had the realization mid stream.... the to-string is an
> llm call.. 'render this edn-form as english'... they have the
> familiar engrams already.."

The user was right. The renderer is one MCP call. Everything
else was over-engineering. This file captures that recognition
and the implications for the wat-english crate.

---

## The realization

**The to-string is an LLM call.** "Render this EDN as English."

Frontier LLMs already have the engrams. Every common structured
form (JSON, YAML, EDN, XML, S-expressions, RDF, schemas, ASTs)
and its English rendering pattern is in their training data.
Asking the LLM to render wat-english as English is asking it
to do something it does effortlessly. The "engrams" — internal
patterns that map structure to prose — are already there. We
don't need to build them.

That collapses the entire renderer design into one MCP call.

## What collapses

- **No template walker.** Not needed.
- **No `:render-template-axis` on Bundles.** Was solving a problem
  that doesn't exist when rendering isn't template-based.
- **No render-Engram library / Hologram of templates.** Not needed.
- **No `:wat::english::render` function in the wat-english crate.**
  The crate ships zero rendering machinery.
- **No tense/aspect conjugation tables.** The LLM handles it.
- **No determiner-selection rules.** The LLM handles it.
- **No pronoun-resolution heuristics.** The LLM handles it.
- **No per-language template sets** (for non-English rendering).
  The LLM handles it.

## What's left

```
prompt: "Render this wat-english AST as natural English prose.
        Keep the meaning exact; do not add information."
input:  <wat-english-AST as EDN>
output: <English prose>
```

That's the entire design. Routed through MCP per the
protocol-as-checksum architecture; the LLM is already in the
loop, so this isn't introducing a new component.

## The direction asymmetry — still real, but for a different reason

The two-direction framing the user had originally still holds:

- **Lift** — English → wat-english. *Lossy.* English is
  positionally ambiguous; the lift has to make judgment calls.
  This is `protocol-as-checksum.md`'s "Hard problem 1." Even
  though the LLM does this too (asked to lift, it produces a
  candidate wat-english AST), the lift requires user judgment
  to commit — the user has to verify the LLM's disambiguation
  matches their intent.

- **Render** — wat-english → English. *Easy.* The AST is
  unambiguous; the LLM has the engrams; the projection is
  trivial. Ask the LLM to render, get prose back. No user
  judgment needed because the canonical form is the AST and
  the prose is the derived projection.

The asymmetry isn't "deterministic-vs-research" anymore; it's
"requires-judgment vs doesn't." The LLM handles both directions
mechanically, but only the lift requires human ratification.

## Why render matters

Three audiences need to read wat-english that aren't going to
learn the form:

1. **Humans without wat priors.** Stakeholders, future
   collaborators, the DEFCON board, anyone reading the
   artifacts later. They should see "The dog chases the toy at
   t-0," not the EDN form.

2. **Other LLMs that don't speak wat.** A weaker model in the
   loop, a model fine-tuned on a different protocol — they can
   consume rendered English even if they can't emit valid
   wat-english. Asymmetric partnership becomes possible.

3. **Future-self.** When you re-read your own arcs in 6 months
   without wat in working memory, the rendered prose is the
   skim layer. The wat-english AST stays the durable record;
   the rendered English is the human-readable index.

## Round-trip honesty — still applies

The collapse to "LLM call" doesn't change the round-trip
property. English → wat-english → English produces *different*
English than the original because the lift made disambiguating
choices.

- The wat-english AST is the canonical form; both Englishes are
  views of it.
- This is fine if the user understands the AST is the source of
  truth and English is a derived projection.

Example:
- Input: "The dog might be barking."
- Lift (LLM): `(:Probably 0.5 (:Statement dog (:ProgressiveAspect bark)))`
- Render (LLM): "The dog is probably barking."

Same proposition; different surface. Both are correct
realizations of the same AST. Information that "was lost"
("might" vs "probably") was the lift's disambiguation. If
precision matters, the user reviews the lifted AST before
committing.

This is the same property TCP has: the wire format normalizes
the input; you don't get back the exact bytes you sent, you get
the canonical representation. For a thinking-substrate, that's
the right behavior — the AST is the durable record; the prose
is incidental.

## The protocol architecture simplifies — four shapes, not five

An earlier draft of this file proposed a five-shape protocol
(Express / Reflect / Verify / Recall / Render). With the
collapse to "LLM call," **Render isn't a separate shape —
it's a special case of Reflect.**

The LLM reflects in two surfaces:
- **Reflect-as-wat-english** — when speaking to the substrate
  for verification + recall (its emissions get type-checked
  and stored on the hologram)
- **Reflect-as-English** — when speaking to humans (or to
  systems that don't speak wat); the same LLM, the same
  underlying intent, a different surface

Same shape; the choice of surface is a parameter on the
reflect-call. The protocol has four shapes:

- **Express** — user emits a thought (wat-english, or English
  via LLM-assisted lift)
- **Reflect** — LLM emits a thought (wat-english to substrate;
  English to humans; either surface, same shape)
- **Verify** — substrate type-checks (`:wat::core::eval`)
- **Recall** — Sift / Hologram retrieves prior thoughts

The substrate's role in rendering = **nothing**. The substrate
verifies and stores; the LLM reflects in either surface; the
user expresses with optional LLM assist; recall is a substrate
operation.

This is cleaner than the five-shape framing. The boundaries
between roles match the actual responsibility split.

## Why I overcomplicated it (and what to remember)

When I drafted the layered template-based renderer, I was
hedging on three concerns:

1. **Cost.** A model call per render. *Doesn't matter.*
   Rendering is on-demand for human consumption, not on every
   write. Pennies per session.

2. **Determinism.** LLM output varies; templates produce stable
   strings. *Doesn't matter.* Phrase variation is irrelevant
   for the use case ("humans can read what was said"). If
   stability is needed for diffs, cache by AST hash.

3. **Self-containment.** The substrate "should" render without
   external help. *Wrong frame.* The LLM is already in the
   loop per the protocol architecture. Asking it to render is
   no more "external" than asking it to lift. The LLM is a
   first-class participant, not an outside dependency.

**The lesson:** when the LLM is already a system component,
collapse to it. Don't write the deterministic version "for
self-containment" if there's no real self-containment
requirement. The substrate's discipline (proposal 058) is to
ship only what's earned by cited use; that discipline applies
here too. A wat-english template renderer has no cited use
because the LLM-call answer covers the use case at lower cost
and zero implementation overhead.

## What the wat-english crate actually ships

After the collapse, the crate's render footprint is:

- **Nothing.** The crate ships only the lift macros (Statement,
  Modal, Negation, Question, etc.) that lower English-shaped
  forms to Bundle/Bind. Rendering is delegated to whatever LLM
  is in the MCP-mediated loop.
- Optionally, a tiny convenience: `(:wat::english::render-via-mcp
  ast-as-edn)` that bundles the standard prompt and routes
  through the configured MCP endpoint. This is purely a
  prompt-management helper, not a renderer.

That convenience ships in the same slice as wat-mcp's
substrate-tier integration, not as a separate render slice.

## Connection to OG wat's Ruby impl

The `og-wat-impl.rb` preserved alongside this file is the
original Ruby reference implementation of OG wat — 490 lines
of parser + evaluator. It does NOT contain a render / to-string
/ describe function. The user's recollection was correct —
the renderer was conceptual, never coded.

What the user couldn't have built 14 months ago: an MCP-mediated
LLM call as the renderer. The substrate (wat-rs) didn't exist;
the LLMs (frontier-class with reliable structured rendering)
weren't ready; the wire protocol (MCP) wasn't standardized.
All three exist now. The "to-string they couldn't build" turns
out to not need to be built — the LLM does it.

This is the same shape as the protocol-as-checksum recognition:
*you were 14 months early on tooling.* Now the tooling exists;
the work that seemed substantial collapses to a config item.

## Status

- **Captured:** 2026-05-02. The collapse recognition arrived
  mid-conversation when the user pointed at it; the file was
  rewritten to reflect it.
- **The renderer is an LLM call.** "Render this EDN as English."
  Frontier LLMs have the engrams already.
- **Direction asymmetry preserved:** lift requires user
  judgment (disambiguation); render is mechanical (the LLM
  does it both directions, but only lift needs ratification).
- **Protocol shapes corrected to four:** Express / Reflect /
  Verify / Recall. Render is a surface choice on Reflect, not
  a fifth shape.
- **The wat-english crate ships zero rendering code.** Optional
  thin convenience wrapper around the MCP call.
- **Cross-references:**
  - `og-wat-impl.rb` — the historical Ruby reference (parser +
    evaluator, no renderer; the renderer is unnecessary now)
  - `protocol-as-checksum.md` — the four-shape protocol
    (Express / Reflect / Verify / Recall); render is a Reflect
    surface
  - `language-form-gaps.md` — the gaps wat-english fills via
    macros; rendering of those macros is the LLM's job
  - `english-surface-arc.md` — the consumer crate; this file's
    update means render slices are unnecessary
  - `PICKUP-GUIDE.md` — slice plan; Phase 5 (render slices)
    collapses per this update
