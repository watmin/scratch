# Open questions for the inverse-hologram experiment

What's locked (not in this file): the four labels are `Bigram(severity, direction)` HolonASTs; the storage is two `HologramCache<HolonAST, HolonAST>` instances (`prev-cache` for the chain edge, `label-cache` for the bundled label population); the walk goes right-to-left gated by `presence?` on state-found AND lean-strong-enough; the prediction is the strongest Bigram at the terminal node where presence runs out.

What's still open: the questions below, with consulting-session recommendations.

---

## Q1 — Multi-edge chain by label?

When a state-coordinate is visited in genuinely different lineages — once during a Grace-up trade, once during a Violence-down trade — does the prev-cache hold:

- **(a) Single-edge chain.** The prev-cache is `state → state`; one prev pointer per node. Multiple lineages collapse to a single chain. Order of-visits decides which prev "wins." Simpler.
- **(b) Multi-edge by label.** Each entry is `state → (label, prev)` per label class. Following the strongest lean's edge means walking the lineage that BORE that label. The walk threads through history-segments-by-outcome.

(b) is more honest if the same coordinate appeared in genuinely-different lineages — following the right edge means seeing what *happened next* in lineages that bore the same label we're now leaning toward. It also matches the user's "follow that path" framing — *path* implies label-specific.

Cost: (b) needs a richer `prev-cache` value. Either four separate prev-caches (one per Bigram), or one prev-cache whose value is a bundled `(label-i, prev-i)` set the consumer destructures.

**Consulting recommendation: (b).** The whole experiment's claim is that the substrate's algebra grid carries the cognition; a single-edge chain that loses lineage-by-label is a substrate-side compression that throws away signal the user explicitly said matters. Carry the labels into the chain edge.

If the lab doesn't want to ship four caches, the pragmatic move is one prev-cache whose value is a bundle of (Bigram-label · prev-coord) bindings. The cosine readout works the same as on the label-cache; the consumer cosines against each Bigram and picks the prev that matches the strongest lean.

---

## Q2 — The terminal label

Three plausible reads:
- **(i)** The label at the FINAL node where presence still held (the deepest confident point).
- **(ii)** The label whose lean fired strongest somewhere along the walk (the maximum-confidence step, regardless of where in the walk).
- **(iii)** The lean at the terminal AFTER stepping through it — i.e., the label whose presence failed first, signaling "this is what the chain was leaning toward right before running out of evidence."

User's phrasing — *"label that is most closely associated with this terminal point (where no next hops have any presence... we went as far as we could)"* — strongly suggests **(i): the label AT the terminal node, where we stopped because nothing leans further.**

**Consulting recommendation: (i).** The terminal is the deepest reachable past state — the earliest predecessor that still bore a labelable outcome-signature. Its label is the deepest matched pattern, the strongest historical analogue. (ii) and (iii) are alternative readings if (i) underperforms; they're easy to add as alternative readout modes once the experiment has data.

---

## Q3 — Forward chain construction

When a new candle arrives, what defines the chain edge from the previous node?

- **(a) Always link to immediate predecessor** — the chain is the candle stream; each node's `prev` is the previous candle's encoded state.
- **(b) Link to most-cosine-similar predecessor in some window** — closer to a graph than a linked list; lets the topology emerge from similarity.
- **(c) Link to multiple predecessors** — `prev` is a Vec<form-handle>; the walker chooses by some rule.

Per Q1's recommendation, (b) and (c) become orthogonal to the lineage-by-label question. The chain edge can be (a) AND multi-edge-by-label simultaneously: each candle's prev IS the immediate predecessor; but the prev-cache entry for that predecessor carries a label-tagged set of prevs from MULTIPLE lineages.

**Consulting recommendation: (a) for chain construction, (b) is for prediction prev-edge.** Each candle's chain link is just the temporal predecessor; lineage diversity at any given node is the consequence of MANY trades passing through that coordinate over time.

---

## Q4 — Label propagation rule (writing on outcome)

When a Grace event fires for a held trade (open=t₀, close=t₁), how do labels deposit at each node along the lineage?

- **(a) Set membership (binary)** — every node on the path is tagged "bears grace-up"; equal weight. Subsequent reads see the label as a Boolean.
- **(b) Decay (distance matters)** — node_t-1 bears `0.9 · grace-up`, t-2 bears `0.81 · grace-up`, etc. Closer-to-outcome → stronger label.
- **(c) Bundled population code (every node IS a population of labels)** — each node stores a Bundle of `(Bigram_g_up · w₁) ⊕ (Bigram_g_down · w₂) ⊕ (Bigram_v_up · w₃) ⊕ (Bigram_v_down · w₄)` where the weights drift based on accumulated outcomes.

**Consulting recommendation: (c).** This is the most substrate-native shape — every node IS a population code; the relative magnitudes ARE the prediction weight; the cosine readout reflects the ratio of accumulated outcomes naturally. Connects directly to Chapter 70's tuning-curve / Prolog framing.

(b)'s decay might be ADDED on top of (c) as a write-time multiplier — when an outcome fires, deposit the label with weight = `decay_factor^(steps_back_from_outcome)`. That gives "recent history matters more than far history" without changing the population-code shape.

---

## Q5 — When does an outcome fire

The trader decides Grace vs Violence based on what — MFE/MAE labels per memory's `project_mfe_mae_labels.md`? Closed trade resolution? Some real-time threshold crossing?

Per the BOOK's framing, Grace = principal + residue returned (good outcome); Violence = bounded loss (bad outcome). These are TRADE-LEVEL events, not candle-level.

**Consulting view:** the outcome fires at trade close; the labeled lineage extends from open to close. Every coordinate encountered during the held position gets the label. Trades of different durations leave longer or shorter lineage trails.

Worth confirming with the user.

---

## Q6 — Up/Down dimension

The 2×2 grid is `Bigram(severity, direction)`. Direction has two natural readings:

- **(a) Trade direction** — long position closed in profit = Bigram(grace, up); long closed in loss = Bigram(violence, up); short closed in profit = Bigram(grace, down); short closed in loss = Bigram(violence, down). The `direction` is the position direction at trade open.
- **(b) Market direction during the held period** — separate from the position; reflects what the market did.

(a) is operationally cleaner — the label encodes "I went long here and got Grace" vs "I went long here and got Violence." Predictions return a direction recommendation directly.

**Consulting recommendation: (a).** Makes the prediction step also choose direction.

---

---

## RESOLVED — Q-cascade-7 (engrams + noise filtering)

User: *"i think we just arrived at the realization we need noise filtering again... strip noise and then do work... b+c just emerged as a requirement through deduction."*

**Resolution:** engrams are required, not optional. They are StripedSubspace decompositions of saturated/multi-lineage label-cache coords. Walker queries each stripe separately to distinguish clean grace-up history from contested grace-up + violence-down history.

Phase 1 ships without engrams (raw bundle in label-cache). Phase 2 adds saturation detector + engram-cache promotion + stripe-separated readout. See `NOISE-FILTERING-AND-ENGRAMS.md` for the full structural sketch.

## RESOLVED — Q-cascade-8 (mixed grace + violence at a step)

User: *"a — we want grace — the equality check wins it... oh.. hrm... the better question is what happens when violence is 0.7 and grace is 0.6..."*

**Resolution:**
- On EQUAL presence → grace bias wins; continue walking grace.
- On VIOLENCE STRONGER → terminate, abstain. Violence dominance means contested or dangerous neighborhood; the substrate's signal at that coord overrides the grace-bias.

Walk per-step rule:
```
strongest = max over {grace_up, grace_down, violence_up, violence_down}
if strongest in grace_*:    continue walking that direction
if strongest in violence_*: terminate; abstain
if no presence:             terminate; return strongest grace seen so far (or abstain if never had grace)
```

## DEFERRED — Q-cascade-9 (ScaleTracker observation timing)

User: *"is this another question masquerading?... the original use for this was to extract unknown magic values through observation.. they just float based on recent data — study the impl..."*

**Resolution:** the question was masquerading. ScaleTracker's design intent is *continuous observation drift* — it learns the empirical magnitude range from recent data; floats. Not a "warmup then freeze" hyperparameter.

**Action:** study the archived `pre-wat-native/src/encoding/scale_tracker.rs` implementation to confirm the exact observation/decay rule before committing to a behavior in the new wat code.

---

## Q-cascade-6 — Does magnitude affect walk navigation, or only terminal readout?

The label structure carries `Bind(Bigram, Thermometer(amount, 0, 1))`. The walk's per-step lean check could:

- **(a) Navigation ignores magnitude; only category matters.** Walker probes with just the Bigram (categorical-only). Magnitude is read at terminal as the expected-amount.
- **(b) Navigation weights by magnitude.** Higher-magnitude past outcomes pull the walk harder — presence-cosine includes magnitude proximity. Walk follows lineages whose past magnitudes match the current state's encoded magnitude.

(a) is simpler. (b) is more honest if "how big a Grace" tracks with "how much do I lean here."

**Consulting recommendation: (a) for first cut.** Phase 1 walks with categorical probes; terminal readout uses the bound magnitude. (b) is a future refinement once Phase 1 has data.

---

## Q-cascade-7 — Population aggregation specifics

A node may have been visited by many trades. Each deposits a full `Bind(Bigram, Thermometer(amount, ...))` label. The label-cache value at that coordinate is a `Bundle` of these binds.

Reading: cosine the bundle against `Bind(Bigram(grace, up), Thermometer(probe-amount, ...))` returns "how like that specific category+magnitude is this state's history?"

Two questions about aggregation:

1. **Is there a per-deposit weight?** If a category fires often, does its accumulated direction in the bundle naturally outweigh a category that fired rarely (frequency reflected in vector magnitude)? Or does each deposit carry equal weight regardless of frequency?
2. **Is there decay over time?** A Grace from yesterday and a Grace from a year ago both deposit at the same coord — do they weight equally, or does recency matter?

**Consulting recommendation:**
- **(1) Equal weight per deposit; let frequency emerge from how many bundle elements share the same direction.** The substrate's bundle naturally accumulates magnitude in proportion to how often a direction is reinforced. No separate weighting needed.
- **(2) No decay for first cut.** The architecture should support decay later (a per-deposit weight = `decay^age`), but Phase 1 lets all deposits weight equally. Decay is a hyperparameter; introduce it when needed, not preemptively.

---

## What follows the open questions

Once Q1–Q6 settle, the experiment shape clarifies:

1. **Forward writer** — every candle, encode current state, append to `prev-cache` with chain link to previous. No labels yet.
2. **Outcome writer** — every trade close, walk backward through the held lineage, deposit the appropriate `Bigram(severity, direction)` label at each node in `label-cache` (with optional decay if Q4 = c+b).
3. **Forward reader / predictor** — at decision time, encode current state, walk leftward through `prev-cache`, gated by both presence checks; at the terminal, read the strongest Bigram from `label-cache` and return.

Three programs. Two HologramCaches. No subspace. No reckoner. The substrate's geometry IS the learner.

This becomes proof 020 (or the next experiment number) — fewer than 200 lines of wat against arc 074's HologramCache + arc 070's eval-walk machinery.
