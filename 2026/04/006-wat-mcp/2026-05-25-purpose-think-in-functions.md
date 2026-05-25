# wat-mcp — purpose & "think in functions" (2026-05-25)

## What it's for

**Let the LLM express a concept in wat and have it evaluated.** That's the
whole purpose, and it's the proving-point operationalized: two years ago an
early Claude (on Bedrock, fed an s-expr preamble) answered with a *generator
function whose evaluation produced more than the response could hold* — it
communicated as a function that had to be *run* to be heard. wat-mcp is the
channel where *any* model does that *on purpose,* and the daemon realizes it.
(See `project_wat_proving_point` in memory.)

## Two use cases — one move, two scales

1. **Writing wat programs.** The model expresses logic as a function,
   evaluates it, reads the EDN result or error, iterates. A REPL *for the
   model.* Discovery is wat-shaped — `(:wat::pause::ls)` / `(show :sym)`
   *through* `speak-wat`; the SymbolTable is the contract, the tool list never
   goes stale.
2. **Structural analysis of a data-backed problem.** The model expresses a
   structural probe as a function — `encode` this shape, what's `coincident?`,
   what's near it — against the *warm* holon store, and gets structural
   matches back. This is holon's **original** purpose (sub-queries over
   structure, not field-equality) — finally with a model driving it by
   thinking in functions.

## "Think in functions" is the interface principle, not a nicety

The interface must **be** functions, because functions are the native medium —
the author's *and the model's* (it has sufficiently observed Lisp, Scheme,
Clojure, lambda calculus; it stands at that coordinate in embedding space).

The one-tool collapse follows from this and is *correct* for it, not merely
simple: a menu of JSON-schema'd endpoints is **the "go learn Rust" of agent
interfaces** — a non-function surface that forces the model out of its medium
into endpoint-selection. `speak-wat` is the refusal: one channel, express the
function, realize it. **It is "notation is the barrier" applied to the tool
interface** — *te respuo* to schemas; give the model the engine and let it
think in functions. (See `feedback_notation_is_the_barrier`.)

This is also why **locate, don't train**: wat sits at the coordinate where
Lisp + VSA + math already live in the embedding; the model navigates there by
similarity. wat-mcp is the dial-tone to that coordinate.

## The frontier — the coherence gate

A persistent wat daemon is the natural home for the truth-engine move: a form
the model calls to ask *"does this thought cohere?"* before returning it —
freeze-time type-check (form coherence, shipped) + `coincident?` against valid
structure (geometric coherence, shipped primitives). **Not built.** But the
daemon is exactly where it lives, and the pieces are on the shelf. This is the
deepest eventual purpose: not just *express and evaluate*, but *express, check
coherence, and only then return.* (See `project_wat_proving_point`,
`project_truth_engine`.)
