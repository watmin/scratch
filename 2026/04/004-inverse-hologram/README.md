# Scratch 004 — Inverse Hologram

**Started:** 2026-04-28.
**Status:** structural shape locked; next conversation cycle continuing.
**Sibling materials:**
- `holon-lab-trading/archived/pre-wat-native/src/` — the system this is the inverse OF
- `holon-lab-trading/docs/proposals/2026/04/056-thought-architecture/PROPOSAL.md` — the "what the screen is encoding" proposal
- `holon-lab-trading/BOOK.md` Chapters 59 / 65 / 66 / 67 / 71 — the substrate framings this experiment cashes in
- `wat-rs/docs/arc/2026/04/074-holon-store/` — the `Hologram` + `HologramCache` primitives this experiment composes over

---

## What this scratch captures

A new architectural direction surfaced 2026-04-28 in conversation. The proofs lane was working through "how should the trader's caches consume the substrate" and the user redirected: the FIRST real experiment shouldn't be a port of the pre-wat-native architecture — it should be its **inverse**.

This scratch holds the structural sketch, the open questions, and the connection to what's already shipped, so the experiment can be drafted cleanly when the user is ready.

---

## The pre-wat-native architecture (what we're NOT doing)

The trader-as-shipped (now in `archived/pre-wat-native/`) encoded **left to right** — candles arrive in temporal order, oldest to newest. For each indicator the lens selects, the system built a 4-level pyramid:

```
Level 0: per-candle facts            (thermometer + delta-bind, per candle)
Level 1: trigrams                    (Bind + Permute over 3 consecutive)
Level 2: bigram-pairs                (Bind over 2 consecutive trigrams)
Level 3: rhythm                      (Bundle all pairs, trim to √d, Bind to atom)
```

Three thinkers (market / regime / broker), each with its own `OnlineSubspace` that learned what "normal" looks like. Each thinker's prediction came from the **anomaly** = `subspace.residual(thought)`. A `Reckoner` discriminant on the anomaly produced the trade signal.

### The disappointment

`prove_rhythm_real_data.rs` measured uptrend-vs-downtrend cosine on real BTC:

```
raw rhythm:   uptrend vs downtrend = 0.72–0.80   (looks the same)
after strip:  uptrend vs downtrend = -0.09–-0.10  (orthogonal)
```

The discrimination only emerges AFTER `OnlineSubspace.residual()` strips learned-normal background. The raw rhythm geometry doesn't separate regimes — the regime signal lives in **deviation from learned normal**, not in the rhythm itself.

This means three serial fitting loops before a trade fires (encoder, subspace, reckoner). Each is fragile. Cold start has no edge.

The encoding pyramid IS right (locality-preserving, order-honest, reversible). What was off was the three-layer stack of LEARNED operations on top of it.

---

## The inverse (what this experiment IS)

User's framing, verbatim:

> "its the mirror of what the right side shows... we treat the right boundary as a hologram to the left... we use the mirror image of the timeline's behavior to learn with prior right→left patterns were predictive of...."
>
> "when we get a grace hit we label every node right to left to the end as bearing grace.... same for a violence hit... we label every node right to left to the end as bearing violence..."
>
> "(and they'll be the 2x2 grid... as a label ... remember HolonAST is a label)"
>
> "so.... when we go to make a prediction... we walk our current right-as-a-surface to the left and follow the path who has the most presence at each step.. we take the highest label we encountered along the chain as the prediction"

User's correction (later in conversation): the four labels are not flat atoms but `Bigram`s — the n=2 specialization of `Ngram` over two ordered atoms. See "The label shape" below.

### The structural shape

Time still flows left-to-right (candles arrive in order). Encoding still happens forward in time — each candle's encoded state becomes a HolonAST stored in a HologramCache. The inversion is in **how outcomes propagate** and **how predictions read**:

```
Time:           past ←─────────────── now (right boundary)
                                       │
Forward chain:  node₀ → node₁ → ... → node_t      (encoded states; one per coordinate)

Outcome fires at the right boundary (Grace at +k, Violence at +k):
  ↓
Backward label propagation:
  node₀ [+g] ← node₁ [+g] ← ... ← node_t [+g] ← outcome
                                                  every node on the lineage gains the label
                                                  same for {g-up, g-down, v-up, v-down}

Many such lineages over time → each node accumulates a population of labels.

Prediction (also walks right-to-left):
  start: now (right boundary), encode current state
  walk leftward through chain:
    presence? on state-found AND lean's strongest-label
    if both: step to chain.prev
  terminate when either presence fails
  return: the strongest label at the terminal node
```

### The label shape — Bigrams, not flat atoms

The four labels are the 2×2 grid of `(severity ∈ {grace, violence}) × (direction ∈ {up, down})`, expressed as the substrate's `Bigram` idiom:

```scheme
(:wat::holon::Bigram (:wat::core::vec :wat::holon::HolonAST
                       (:wat::holon::Atom "grace")
                       (:wat::holon::Atom "up")))      ; g-up

(:wat::holon::Bigram (:wat::core::vec :wat::holon::HolonAST
                       (:wat::holon::Atom "grace")
                       (:wat::holon::Atom "down")))    ; g-down

(:wat::holon::Bigram (:wat::core::vec :wat::holon::HolonAST
                       (:wat::holon::Atom "violence")
                       (:wat::holon::Atom "up")))      ; v-up

(:wat::holon::Bigram (:wat::core::vec :wat::holon::HolonAST
                       (:wat::holon::Atom "violence")
                       (:wat::holon::Atom "down")))    ; v-down
```

`Bigram` is the n=2 specialization of `Ngram` — internally it's `Bind(Atom_a, Permute(Atom_b, 1))`. Order matters: `Bigram(grace, up) ≠ Bigram(up, grace)`. Four distinct HolonAST coordinates emerge from two binary axes.

**Why Bigram beats flat atoms:**
- Order is preserved (severity comes before direction).
- The two axes can be queried compositionally — a probe of shape `Bigram(grace, _)` partially unbinds against the population to ask "is this leaning toward grace regardless of direction?" without enumerating both g-up and g-down separately.
- The substrate's algebra reads ordered structure cleanly; this is what `Ngram` was built for.

### The dual-cache shape (BOOK Chapter 59 reused)

The architecture lands on TWO `HologramCache<HolonAST, HolonAST>` instances — the same dual-LRU coordinate cache pattern from Chapter 59, repurposed:

```
prev-cache  :  HologramCache<HolonAST, HolonAST>
                ; key = state coordinate
                ; val = prev-state coordinate (the chain edge, walked leftward at prediction)

label-cache :  HologramCache<HolonAST, HolonAST>
                ; key = state coordinate
                ; val = bundled label-population AST
                ;       (a Bundle of bind-weighted Bigram labels accumulated over outcomes)
```

**Two HologramCache instances. ZERO new cache types.** The substrate's HologramCache (arc 074 slice 2) is already `HolonAST → HolonAST`; the label-cache stores a population AST as its value, the prev-cache stores a coordinate AST. Both fit cleanly.

### The walk per step

```
loop:
  1. presence? state in prev-cache                  ; "am I found"
  2. label-pop = label-cache.get(state)             ; AST: bundled labels
  3. for each of 4 Bigram labels:
       cos = cosine(label-pop, bigram-label)
     pick max → strongest-label, strongest-cos
  4. presence? strongest-cos against the population ; "do I lean enough"
  5. if both presence checks hold:
       state ← prev-cache.get(state)
       continue
     else:
       break
return strongest-label at the terminal state
```

Two presence checks per step (hologram-found AND label-leans). Same substrate predicate, different surfaces. The terminal label IS the prediction AST.

### Why this is "the inverse"

The pre-wat-native system's data flow:
```
candles ──pyramid encoder──> vector ──learned subspace──> anomaly ──learned reckoner──> direction
                                       (THREE serial learning loops)
```

The inverse system's data flow:
```
candles ──encode each step──> coordinate stored in HologramCache    (no learning yet)
outcomes ──propagate labels right-to-left along the lineage──> populated label-cache
prediction ──walk leftward through prev-cache──> read terminal label-pop ──cosine readout──> direction
                                                        (substrate cosine; no model fitting)
```

Three learning loops collapse to **writes into the substrate's algebra grid**. Every Grace/Violence event is a write. Every prediction is a walk-and-read. The substrate's `coincident?` + `presence?` + Hologram cosine readout do what the OnlineSubspace + Reckoner combination was approximating — and do it from the first bar without training.

The encoding pyramid (Thermometer + delta + trigram + bigram-pair + bundle) stays. What changes is what happens to the produced vector: instead of "feed it through learned classifiers," it becomes "key into a substrate-native cache of past labeled lineages."

### Connection to BOOK framings already shipped

- **Chapter 59** — *42 IS an AST.* The dual-LRU coordinate cache; this experiment uses exactly that pattern (two HologramCaches keyed by state, one for the chain edge, one for the labels).
- **Chapter 65** — *The Hologram of a Form.* The right boundary IS the surface; the encoded state IS the hologram of all the past it represents. Walking inward (leftward in time) reads the encoded interior.
- **Chapter 67** — *The Spell.* Coordinates publishable to a network. Once labels live as Bigram coordinates, the substrate is the protocol — multiple traders can share a labeled algebra grid without coordinating beyond seed agreement.
- **Chapter 71** — *Vicarious.* Every cache hit is a walker eating a walker. The prediction walker reads labels deposited by past trajectories that bore Grace or Violence. Past walkers' deaths feed the present walker's decisions. Predation is honest.
- **Arc 070** — `:wat::eval::walk`. The forward walker's machinery; the inverse experiment uses the SAME shape, traversed leftward, reading instead of computing.
- **Arc 074** — `Hologram` + `HologramCache`. The substrate primitives this experiment writes into and reads from. ZERO new cache types needed.

---

## What's locked

| Question | Answer |
|---|---|
| Labels (per-tier base shape) | Four `Bigram(action, severity)` HolonASTs per tier; not flat atoms. |
| Magnitude addition | `Bind(Bigram, Thermometer(amount, 0, 1))`. Fraction always 0-1; (min, max) is domain knob. |
| Storage | Two `HologramCache<HolonAST, HolonAST>` instances PER TIER: `prev-cache` (chain) + `label-cache` (population). No new cache types. |
| The "labeled object" | The label-cache (a HologramCache whose value side is a population AST). NOT an OnlineSubspace. |
| Walk shape | Right-to-left, gated by two presence checks per step (state-found AND lean-strong-enough). |
| Prediction | The strongest Bigram (with bound magnitude) at the terminal state — where presence runs out. |
| Cascade — forward | Context-accumulating tuple. Each tier appends thought-ast + bias. |
| Cascade — backward | Treasury fans out per-tier specific labels at paper-trade-close. |
| Per-tier label spaces | Market: entry × outcome. Exit: close-decision × outcome. Broker: action × outcome (with counterfactual labels for vetoes). |
| Phase 1 build | Market fleshed; exit + broker as relays; treasury settles + fires market's label. |
| Magnitude bounding | `weight = abs(close-open) / max(close, open)`. Symmetric, naturally bounded to [0, 1). No learned hyperparameter. |
| ScaleTracker's role | **Resolution-tightening** on typical-magnitude range, NOT bounding. Substrate clamps fraction at 1.0 for outliers. |
| Walk navigation | Grace-bias on equality. **Violence-strongest at any step → terminate, abstain.** Grace-strongest → continue. |
| Multi-lineage coords | Engrams via StripedSubspace decomposition. Required, not optional. Phase 1 ships without; Phase 2 adds engram promotion. |

## Open

See:
- `OPEN-QUESTIONS.md` — original architecture questions (chain-edge multiplicity, terminal readout, propagation rule, etc.) plus magnitude follow-ups.
- `CASCADE-ARCHITECTURE.md` — the per-tier pipeline architecture, treasury responsibilities, magnitude addition, relay pattern for incremental build.
- `NOISE-FILTERING-AND-ENGRAMS.md` — why noise filtering came back as a requirement; how engrams via StripedSubspace solve multi-lineage coord smear; phase progression.
- `MAGNITUDE-BOUNDING.md` — how to compute the weight bounded to [0, 1); ScaleTracker's role refined to resolution-tightening; severity rule and edge cases.
- `PROPOSAL.md` — six-slice build plan with throughput gate; the actionable plan for the 36-hour window. Migrates to lab proposal 060 once infra clears.
- `ENGRAM-HOLOGRAM-DESIGN.md` — in-progress design for the substrate primitive that replaces `Hologram` (unbounded) and `HologramCache` (LRU) for label-cache use. Compaction-by-merge eviction; count + age out-of-band. Resume after infra wraps.
