# Cascade architecture — context-accumulating tuple, treasury-fires labels

This file extends the README's inverse-hologram architecture with the cascade structure that connects the trader's tiers. The architecture is the pipeline of programs that carry data forward and labels backward; the magnitude addition appended to label structure is included.

---

## The pipeline

```
Forward (data accumulates context as it flows):
  parquet ─► ohlcv stream ─► candle ─► market ─► exit ─► broker ─► treasury

Backward (labels fire from treasury to all tiers when a paper trade closes):
  treasury ─► broker
            ─► exit
            ─► market
```

**Forward flow is a context-accumulating tuple.** Each tier APPENDS its own thought-AST and bias to the data structure flowing through. Nothing gates at the data level — every tier sees all prior context.

```
candle 
  → market adds:    market-thought-ast, market-bias       (one of 4 Bigrams or "abstain")
  → exit adds:      exit-thought-ast,   exit-bias          (close-current vs hold)
  → broker adds:    broker-thought-ast, broker-action      (the actual decision after vetoes)
  → treasury settles broker's action; observes outcome
```

By the time treasury sees the tuple, it carries the entire decision provenance: what each tier thought, what each tier biased toward, what broker decided to do, what actually happened. That makes per-tier label cascade trivial — treasury knows the provenance and translates the outcome into per-tier label spaces.

**Backward flow fans out from treasury, not chained tier-to-tier.** Treasury knows each tier's label space (it's the polyglot) and fires the right label to each. Per-tier specific labels, not a single label that gets translated as it cascades.

---

## When treasury fires labels — paper trade close

The label trigger is a **paper trade closing**. Two close mechanisms × two outcome signs gives four firing conditions:

| Close mechanism | P&L sign | Outcome |
|---|---|---|
| Expiration hit + lossy | negative | **violence** |
| Exit chose early-close + lossy | negative | **violence** |
| Expiration hit + residue-generating | positive | **grace** |
| Exit chose early-close + residue-generating | positive | **grace** |

A paper trader is bound to a broker who is composed of (market, exit). When the paper trade closes, treasury fires labels to each of the three tiers:
- **Market's label** — about its entry signal that opened this trade
- **Exit's label** — about its close-vs-hold call that ended this trade
- **Broker's label** — about its action choice (trust / veto market; trust / veto exit)

Same outcome, different label-space per tier.

---

## Per-tier label spaces

Each tier has its own four-Bigram structure:

### Market — entry-direction × outcome

```
Bigram(grace,    up)          ; entry-long got grace → confident long signal pays off
Bigram(grace,    down)        ; entry-short got grace → confident short signal pays off
Bigram(violence, up)          ; entry-long got violence → false long signal
Bigram(violence, down)        ; entry-short got violence → false short signal
```

Market's action gate: signal only if leaning toward grace; abstain if leaning toward violence.

### Exit — close-decision × outcome

```
Bigram(grace,    close)       ; closed early and got grace → good early-close
Bigram(grace,    hold)        ; held to expiration and got grace → good hold
Bigram(violence, close)       ; closed early and got violence → close didn't avoid loss
Bigram(violence, hold)        ; held to expiration and got violence → should have closed
```

Exit's action: close vs hold the open position.

### Broker — action × outcome

```
Bigram(grace,    take)        ; took market's signal and got grace → trust was right
Bigram(grace,    veto)        ; vetoed and counterfactual was violence → veto was right
Bigram(violence, take)        ; took the signal and got violence → trust was wrong
Bigram(violence, veto)        ; vetoed and counterfactual was grace → veto was wrong
```

Broker's two veto powers:
1. Refuse to ENTER a trade market is biased toward
2. Refuse to EXIT a trade exit is biased to close

Broker's labels need treasury to also compute counterfactuals (what would have happened if the veto hadn't fired) — see "Treasury responsibilities" below.

---

## Magnitude — labels carry "how much"

A 0.1% Grace and a 5% Grace are NOT the same lesson. The label structure binds a Thermometer-encoded magnitude to the categorical Bigram:

```scheme
(:wat::holon::Bind
  (:wat::holon::Bigram (:wat::core::vec :wat::holon::HolonAST
                         (:wat::holon::Atom action)         ; "up"/"down" or "close"/"hold" or ...
                         (:wat::holon::Atom severity)))     ; "grace"/"violence"
  (:wat::holon::Thermometer amount 0.0 1.0))               ; magnitude in [0,1] fraction-space
```

The `Bigram(action, severity)` is the categorical claim. The bound `Thermometer(amount, 0, 1)` carries magnitude with locality — a 0.6 Grace is coincident with 0.65 Grace; not coincident with 0.05 Grace.

### Cosine readout asymmetry

This shape gets you two probe styles:

- **Categorical-only probe**: `Bigram(grace, up)` with no magnitude bound. Cosine returns "leans toward grace-up regardless of magnitude." Used for navigation during the walk.
- **Categorical + magnitude probe**: full `Bind(Bigram(grace, up), Thermometer(0.7, 0, 1))`. Cosine returns "leans toward grace-up at this specific magnitude." Used for terminal readout to predict expected magnitude.

### The 0-1 fraction is universal; the (min, max) is the knob

The substrate sees `Thermometer(value, min, max)` and computes fraction = `(value − min) / (max − min)`, clamping to [0, 1]. So:
- **Fraction is always 0-1** at the substrate level — the algebra grid resolves √d cells uniformly across that range.
- **The (min, max) range is the user's knob** — domain-specific. For BTC 5-min trades, `(0.0, 0.05)` is reasonable (5% max single-trade move; anything above clamps).

Two strategies for picking (min, max):
- **Hard-coded asymptote** — pick `(0.0, 0.05)` first cut; deterministic; predictable.
- **ScaleTracker-learned** — observe the empirical range; adapt. Like the pre-wat-native code had. Defer until calibration becomes the bottleneck.

### Aggregation in the population

A node may be visited by many trades. Each deposits its own full label:

```scheme
;; label-cache.get(coordinate) → Bundle of full-Bind labels
(:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST
  (:wat::holon::Bind (Bigram grace up)    (Thermometer 0.5 0.0 1.0))
  (:wat::holon::Bind (Bigram grace up)    (Thermometer 0.8 0.0 1.0))
  (:wat::holon::Bind (Bigram violence dn) (Thermometer 0.2 0.0 1.0))
  ;; ...
))
```

Cosine readout against `Bind(Bigram(grace, up), Thermometer(0.7, ...))` answers "how much like 0.7-Grace-up is this state's history?" The substrate gives the answer geometrically — the bundle's accumulated direction encodes BOTH category-frequency AND magnitude-distribution. No separate "average magnitude per category" computation; the substrate's bundle does it.

---

## Treasury's responsibilities

Treasury's job grows beyond just settling executed trades:

1. **Settle paper trades.** Determine close (expiration or exit-fired). Compute P&L. Determine grace vs violence + magnitude.
2. **Translate outcome to per-tier labels.** Knows market's label space, exit's label space, broker's label space. Builds the right `Bind(Bigram, Thermometer)` for each tier.
3. **Fire labels backward to each tier.** Three messages, one per tier. Each tier deposits in its own label-cache.
4. **Counterfactual simulation for broker veto-related labels.** When broker vetoes a market signal and the trade-not-taken WOULD have produced grace, broker needs to learn "veto was wrong here." Treasury simulates the path-not-taken (trivial: replay the ohlcv from the would-be-entry to the would-be-close) and computes the outcome that didn't happen. This is broker-specific — market and exit only label trades that actually executed.

Counterfactual simulation only applies when broker vetoed. Without veto: regular outcome cascades. With veto: regular outcome cascades + counterfactual simulation for broker.

---

## The relay pattern for incremental build

Each tier (market, exit, broker) is a service program with a labeled-hologram-pair (prev-cache + label-cache) when fleshed out — and a relay otherwise.

### Relay-tier shape

```
Forward:  recv tuple, append (None, "abstain"), emit
Backward: recv label, no-op (no label-cache to deposit into yet)
```

Identity in both directions. No state.

### Fleshed-tier shape

```
Forward:  recv tuple → encode-thought → label-cache.lookup → predict bias → emit tuple+thought+bias
Backward: recv label → walk lineage in prev-cache → deposit Bind(Bigram, Thermometer) at each coord
```

Same channels, same protocol — stateful instead of pass-through.

### Phase 1 first cut

- **Market** — fleshed (the inverse-hologram thinker per the README; first real test of the substrate)
- **Exit** — relay (always emits `"hold"`; never closes early; trades close on expiration only)
- **Broker** — relay (always trusts market signal; no vetoes)
- **Treasury** — fleshed (settles paper trades, fires labels, runs counterfactuals when broker vetoes — but Phase 1 broker doesn't veto so counterfactuals never run)

Architecture lands with one real thinker. Three relays. Treasury observes outcomes and fires market's label. As phases accumulate, broker and exit grow real thinking — their label-caches activate, their veto powers come online, treasury's counterfactual machinery starts running.

### Phase 2

- **Exit** — fleshed. Has its own labeled-hologram. Predicts close vs hold per-candle. Emits exit-bias.
- **Broker** — fleshed. Has its own labeled-hologram per veto type. Reads market+exit biases; decides own action. Vetoes when its own learned hologram disagrees.
- **Treasury** — runs counterfactuals when broker veto fires.

Same wat program graph. Interior of each program changes.

---

## Locked architectural decisions (extends the README's table)

| Question | Answer |
|---|---|
| Forward flow | Context-accumulating tuple. Each tier APPENDS thought-ast + bias. Nothing gates at data level. |
| Backward flow | Treasury fans out per-tier specific labels (treasury is polyglot). Not chained tier-to-tier. |
| Label trigger | Paper trade close. Two mechanisms × two signs = 4 firing conditions. |
| Per-tier label spaces | Each tier has 4 Bigrams. Different action axes (entry-direction / close-decision / action-take-or-veto). |
| Magnitude | `Bind(Bigram(action, severity), Thermometer(amount, 0, 1))`. Fraction is always 0-1; (min, max) is domain knob. |
| Aggregation | `Bundle` of full-Bind labels at each coordinate. Substrate's geometry encodes category-frequency AND magnitude-distribution. |
| Counterfactual | Treasury runs the path-not-taken when broker vetoes; broker's label-cache learns from those too. |
| Relay pattern | Each tier is forward/backward identity until fleshed. Phase 1 = market only. |

---

## Still open

See `OPEN-QUESTIONS.md` for Q-cascade-6 (does magnitude affect walk navigation), Q-cascade-7 (population aggregation specifics), and the original Q1–Q6 from the inverse-hologram architecture.
