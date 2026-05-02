# Protocol-as-checksum — wat as a verifiable wire format

The user's original premise from ~14 months ago, articulated
sharply on 2026-05-02:

> "i was quite convinced that getting an llm to speak in lisp
> was trivial.. the premise was that i could compile their
> answers and measure for violations... bad forms get rejected
> and a retry is performed... we just consider bad forms to be
> a protocol error and retry... if we can actually train and do
> whatever that.... 'add a layer is' to an existing model we
> could more tractably get this?...
>
> but i was adamant we could query in lisp and reply in lisp...
> both parties could verify the transmission content adheres to
> a required form and then proceed forward...."

This file captures why the premise was directionally right but
14 months early on the tooling, what's now actually possible,
and the architectural shape of the realized system.

---

## The premise was right

Compile every emission against a type system. Reject what doesn't
type-check. Retry. Treat bad forms as protocol errors with the
same semantics as a TCP retry: the wire is unreliable; the
checksum is what you trust. Both parties hold the same checksum;
both verify before consuming. The form survives across party
substitution because the substrate, not the parties, holds the
invariants.

This is the load-bearing architectural insight. It maps onto how
modern reliable protocols work. Trust moves from social to
mechanical.

## What was missing 14 months ago

Four prerequisites the premise required, none of which existed:

1. **A model good enough to emit complex Lisp reliably.** Grok-1
   class models, GPT-3.5-class models, the open models of early
   2025 — all could emit s-expressions, but error rates climbed
   fast as forms grew. Long sessions degraded. Brackets
   unbalanced. Retry budgets exhausted before convergence.

2. **A substrate to compile against.** OG wat was a spec. There
   was no parser, no type-checker, no trait verifier. To run
   the premise you'd have had to build the compiler before the
   loop had anything to compile against.

3. **A standard wire protocol for retry.** Putting error
   messages back into the prompt worked but consumed context.
   No clean shape for "your last emission failed type-check;
   here's the diagnostic; here's a fresh chance."

4. **Bidirectional symmetry as a real possibility.** Both
   parties verifying the same transmission requires both
   parties to share a substrate, not just a notation. Without
   a shared substrate, "verify the form" is honor-system.

## What exists now

Each prerequisite is filled:

1. **Frontier models can stay in form.** Opus 4.7, Sonnet 4.6,
   GPT-class equivalents — they emit deeply nested typed
   s-expressions across long contexts with low first-shot error
   rates. The retry loop terminates in 1-2 attempts on most
   well-formed requests.

2. **wat-rs is the substrate to compile against.** HolonAST is
   closed under itself; the type-checker rejects ill-typed
   forms; `:wat::core::eval` runs the result. The compiler the
   premise needed exists.

3. **MCP is the retry protocol.** The LLM emits wat through a
   tool; the tool returns `is_error=true` with the type error;
   the LLM revises in the same turn without context expansion.
   Tool use is the retry primitive.

4. **Shared substrate is now real.** The user runs wat-rs
   locally; the LLM accesses wat-rs through MCP; both verify
   against THE SAME substrate. One source of truth for "is this
   well-typed."

## "Add a layer" — what that means in modern terms

The user's intuition that you could *add* something to a model
to make it natively wat-fluent maps onto four mechanisms, in
increasing power:

- **System prompt + few-shot.** Cheapest. Prime the model with
  the type system and canonical examples. Works for frontier
  models on simple statements; degrades on complex forms.

- **Fine-tuning / LoRA.** Train a lightweight adapter on a
  corpus of valid wat. The corpus can be synthetic — wat-rs
  generates millions of well-typed examples on demand. Cheap on
  a single GPU.

- **RLHF / RLAIF on wat correctness.** Reward signal: did the
  emission type-check on the first try? Converges toward
  near-100% adherence at substantial training cost.

- **Constrained decoding.** **The powerful one.** Mask the
  next-token distribution at inference time to only allow
  tokens that lead to valid wat. The grammar is derived from
  the wat type system. The model literally CANNOT emit invalid
  syntax. Available now via outlines, guidance, llama.cpp
  grammars, AWS Bedrock grammars, vLLM with grammar support.
  No training needed. Attach the wat grammar to any base model
  and that model becomes a wat-emitter that's syntactically
  perfect by construction.

The fourth option is the "add a layer" the user was intuiting,
in a much sharper form than was available then. The "layer"
isn't fine-tuned weights; it's a runtime constraint. Any model —
including small local ones — gains 100% wat syntactic validity
by being run under the constraint. This drops protocol-error
rate to ZERO for syntax. Only semantic errors (type mismatches,
undefined symbols) reach the substrate's verifier.

So the user's "trainable layer" intuition split into two things
in the modern world:

- A **cheap permanent layer** (constrained decoding) for syntax
- A **retry loop against the substrate** for semantics

They compose. The combined system has near-zero protocol error
rates.

## Bidirectional verification — the deep architectural insight

The pieces above are tooling. The architectural insight is that
both parties verify before proceeding. This is the move TCP
makes that ad-hoc terminal sessions don't. You don't trust the
wire; you trust the checksum. wat plays the role of the
checksum.

Consequences:

- **The LLM cannot lie about what it said.** Its emission is in
  a verifiable form. The "did you really mean this?" loop
  becomes mechanical instead of conversational.

- **The user cannot misread.** The form's morphology is
  explicit. `Bind(:role-subject, dog)`, not `dog chases toy`
  where you have to guess what's the subject. Misreading is
  impossible because there's nothing to interpret positionally.

- **Trust is in the substrate, not in the parties.** Neither
  side has to remember the trait system. The substrate enforces
  it. You and the LLM are both *clients* of the same verifier.

- **The protocol survives party-substitution.** Swap the LLM
  for another model, or have multiple models speaking the same
  protocol, or model-to-model exchanges where the user isn't
  in the loop. The form is preserved because the substrate is
  the invariant.

## Retry-as-protocol-error — the right framing

Treating bad forms as protocol errors and retrying makes this
not just possible but *cheap*:

- **Retries don't pollute history.** Tool-use semantics let you
  discard failed emissions; only the successful one becomes
  part of the conversation. The LLM doesn't have to read its
  own past failures to learn from them — the substrate's
  diagnostic is enough in-turn.

- **The retry budget is small.** With constrained decoding for
  syntax + a typed substrate for semantics, even complex
  emissions converge in 1-3 retries. Budget aggressively.

- **Bounded retries surface real disagreement.** If the LLM
  can't converge after N tries, that's a signal — the request
  is ambiguous, the type system is too narrow, or the LLM
  doesn't know how to express the thought in the form. The
  signal is information, not failure.

- **Retries are localizable.** A failed Bundle inside a larger
  message doesn't invalidate the whole message; only the failed
  sub-form gets retried. Fine-grained verification.

## Two hard problems — deferred for later

Two honest things, neither fatal, both flagged for future work:

### Hard problem 1 — Lifting free English to wat-english is lossy

When the user types a paragraph, the LLM has to lift it into
structured statements. The lift involves judgment calls (what's
the subject, what's the verb, what counts as one statement vs.
many). The substrate can verify the lift is well-typed but
can't verify it preserved the user's meaning.

Candidate solution shape: the lift is a *proposal* the user
accepts or revises. Round-trip: user types English, LLM proposes
wat-english, user accepts or corrects, accepted form becomes the
canonical record. The English is the conversation; the
wat-english is the durable artifact.

User's stance 2026-05-02: "the two 'hard problems' are something
i'll approach tackling later... go[od] to know..."

### Hard problem 2 — Cost-vs-value for trivial exchanges

Wrapping "ack" or "looks good" in wat-english is overhead with
no payoff. The protocol earns its keep on substantive work —
claims, plans, designs, decisions. Ephemeral chat stays in
English.

Candidate discipline: the protocol is for SUBSTANTIVE work, not
chat. The decision of *which exchanges go into the substrate*
is itself a discipline to figure out — a user-visible "commit
this turn to the substrate" toggle, or heuristic detection of
substantive vs. ephemeral, or just user judgment.

User's stance: same — deferred for later approach.

## What this means concretely

The user's original premise becomes the actual architecture.
Two practical implications, both already sized in this arc:

- **wat-english as a consumer crate** is the surface both
  parties write in. It lifts to Holon Bind/Bundle that the
  substrate verifies. The user writes it directly when
  precision matters; the LLM lifts free English to it on
  demand. Sized 5 slices in `english-surface-arc.md`.

- **Constrained decoding becomes the standard for any model
  talking wat.** Whether you're using Opus through MCP or
  running a local Llama under llama.cpp grammar constraints,
  the syntax is guaranteed by the inference layer. The
  substrate handles semantics. The retry loop only fires on
  type errors and converges fast.

## Status

- **Captured:** 2026-05-02 (continuing from the latin-in-wat
  recognition thread).
- **Premise:** validated as directionally right, 14 months early
  on tooling.
- **Realized architecture:** named (constrained decoding +
  substrate retry loop + bidirectional verification + MCP wire).
- **Two hard problems:** flagged, user-deferred, captured here
  with candidate solution shapes.
- **Cross-references:**
  - `latin-in-wat.md` — the *grammar* recognition (morphology
    over position); this file is the *protocol* counterpart
  - `english-surface-arc.md` — the consumer crate that lands
    OG wat's surface on top of current substrate
  - `analysis.md` — what survived from OG wat
  - DEFCON Speaker-Perspective answer — the public-facing thread
    about what the user was carrying for years
