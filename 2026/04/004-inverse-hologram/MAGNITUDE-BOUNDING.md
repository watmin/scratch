# Magnitude bounding — symmetric weight formula + ScaleTracker

The label structure carries `Bind(Bigram(action, severity), Thermometer(weight, 0.0, scale))`. This file pins down two things the architecture needs from the magnitude axis:

1. **How to compute the weight from raw trade prices** — naturally bounded to [0, 1).
2. **What ScaleTracker does** in this architecture — resolution, not bounding.

---

## The bounding formula

```
weight = abs(close_price - open_price) / max(close_price, open_price)
```

Equivalently:

```
ratio   = close_price / open_price
weight  = 1 - min(ratio, 1/ratio)
```

Both forms give the same answer. Choose by readability.

### Why this form

- **Naturally bounded in [0, 1).** No clamping needed; the math caps it.
- **Symmetric for gain vs loss.** A $100→$200 doubling and a $200→$100 halving both produce weight = 0.5. The categorical Bigram carries direction; the magnitude axis carries "how much."
- **Asymptotes at 1.0.** Total wipeout ($X → $0) gives weight = 1.0. Infinite gain ($X → ∞) approaches but never reaches 1.0.
- **No learned hyperparameter for bounding.** Pure math, deterministic, reproducible. ScaleTracker still earns its keep but at a different layer (see below).

### Worked examples

| Open | Close | Computation | weight |
|---|---|---|---|
| $100 | $250 | abs(250-100)/250 | **0.6** |
| $100 | $1,000 | abs(1000-100)/1000 | **0.9** |
| $100 | $103 | abs(103-100)/103 | **0.029** |
| $100 | $97 | abs(97-100)/100 | **0.03** |
| $100 | $50 | abs(50-100)/100 | **0.5** |
| $100 | $200 | abs(200-100)/200 | **0.5** |
| $100 | $0 | abs(0-100)/100 | **1.0** |
| $100 | $∞ | asymptote | **→ 1.0** |

Slight asymmetry in the small-percentage case ($100→$103 vs $100→$97 differ at the third decimal). For our use this is acceptable — the substrate's algebra grid resolves √d cells; sub-cell precision differences are below the discrimination boundary.

### Why NOT the naive formulas

```
;; Naive percentage gain — NOT bounded:
weight = abs(close - open) / open
;; $100→$50:  weight = 0.5
;; $100→$200: weight = 1.0   (linear "+100% gain")
;; $100→$1000: weight = 9.0  (UNBOUNDED — breaks the [0,1] requirement)

;; Smoother: log-and-saturate (works but adds a tuned hyperparameter):
weight = 1 - exp(-alpha * abs(log(close/open)))
;; α controls saturation rate; needs picking
```

The naive form is fine if you cap manually, but the symmetric form needs no cap and reads cleaner. The log-and-saturate form is mathematically smoother but introduces α as a knob — no clear principled value, just empirical tuning. For the inverse-hologram first cut, the symmetric form is honest and parameter-free.

---

## ScaleTracker's role — resolution, not bounding

The weight is in [0, 1) by construction. ScaleTracker is NOT for bounding. It earns its keep at a different layer: **resolution-tightening on the typical-magnitude range.**

### The problem ScaleTracker solves

Most 5-min BTC trades produce small weights — typically 0.005–0.05. Outliers reach 0.1+; rare moves above 0.3 happen. If we use `Thermometer(weight, 0.0, 1.0)` with a fixed 1.0 max:

```
Thermometer(0.005, 0.0, 1.0)  ; fraction = 0.005 → threshold = 0.5% of d
Thermometer(0.030, 0.0, 1.0)  ; fraction = 0.030 → threshold = 3% of d
Thermometer(0.050, 0.0, 1.0)  ; fraction = 0.050 → threshold = 5% of d
```

At d=10000, all of these land in the bottom 5% of cell-space. The √d=100 cells available aren't being used — typical magnitudes clump into ~5 cells. Discrimination between common magnitudes is lost.

### What ScaleTracker does

Tracks the EMA of |weight| over observed trades. Produces `scale = round_to(2.0 * ema_abs, 2)`.

After warmup, if typical weight ≈ 0.03, ScaleTracker's scale ≈ 0.06. The Thermometer's max becomes that:

```
Thermometer(0.005, 0.0, 0.06)  ; fraction = 0.083 → 8.3% of d  (cell ~8)
Thermometer(0.030, 0.0, 0.06)  ; fraction = 0.500 → 50% of d   (cell ~50)
Thermometer(0.050, 0.0, 0.06)  ; fraction = 0.833 → 83% of d   (cell ~83)
Thermometer(0.300, 0.0, 0.06)  ; fraction = 5.000 → CLAMPED at 1.0 (cell ~99)
```

Now the typical magnitude range spans the full algebra grid; outliers saturate. The substrate's resolution lives where the data lives.

### The clamp is honest about outliers

When a weight exceeds the learned scale (a 30% move when typical is 3%), the Thermometer clamps fraction at 1.0. The substrate sees "this is bigger than anything we typically see" — without distinguishing 30% from 50% from 100%. That's accurate to the substrate's discrimination capacity: above the learned range, all outliers look the same.

If outlier discrimination matters, ScaleTracker's slow drift (alpha caps at 1/100 after 100 observations) eventually lifts the scale toward the new regime. Recent outliers nudge `ema_abs` upward; scale adapts; future readings get more resolution at the now-larger range.

---

## The two-stage flow, restated

```
Treasury observes trade close:
  open_price, close_price → raw inputs
  ↓
Stage 1 — bounded math:
  raw_pnl_pct = (close - open) / open                  ; signed; → severity
  weight      = abs(close - open) / max(close, open)    ; → [0, 1) by construction
  severity    = "grace" if raw_pnl_pct > 0 else "violence"
  ↓
Stage 2 — substrate-native encoding:
  scale_tracker.update(weight)                         ; learns typical-magnitude range
  scale = scale_tracker.scale()                        ; round_to(2 * ema, 2)
  label = (Bind
            (Bigram (vec (Atom action) (Atom severity)))
            (Thermometer weight 0.0 scale))            ; substrate clamps fraction at 1.0
```

Two stages, two responsibilities:
- **Stage 1** does the math (bounded, symmetric, parameter-free).
- **Stage 2** does the substrate encoding (resolution-tightened to where data lives, with clamp-on-outliers).

Neither stage knows about the other's concerns. Composition is clean.

---

## Edge cases

**Trade closes at exactly open_price (0% P&L).** Both numerator and weight = 0. Severity = violence (per "if pnl > 0 then grace else violence"). Label deposited at zero-magnitude. The substrate sees a Thermometer at fraction 0.0; cosine readout against any positive-fraction probe returns minimal presence. These deposits don't pull the population's direction much; their main effect is that violence's cell-0 region accumulates "things that did nothing happened to be filed under violence."

If 0% trades happen often enough to skew the violence label-population, treat them as a separate class (a third Bigram: `Bigram(neutral, _)`) — but this is Phase 2 if it surfaces; Phase 1 lumps with violence.

**Open price is zero or negative.** Pathological, shouldn't happen in price data. Treasury rejects with a runtime error; no label deposited.

**Close price is zero (full loss).** Weight = 1.0; severity = violence; Thermometer clamped at fraction 1.0. The label IS "the worst possible outcome." This is the architecture's max-violence sentinel.

---

## What's locked from this discussion

| Question | Answer |
|---|---|
| How to compute weight | `abs(close - open) / max(close, open)`. Symmetric, naturally bounded to [0, 1). |
| Severity rule | `pnl > 0` → grace; `pnl ≤ 0` → violence. 0% lumps with violence (Phase 1). |
| ScaleTracker's role | Resolution-tightening on typical-magnitude range, NOT bounding (math handles bounding). |
| Outlier handling | Substrate's Thermometer clamps fraction at 1.0; ScaleTracker drifts toward new regime over time. |
| Tracker count | One tracker for absolute trade magnitude. Severity carried by the Bigram, not by separate trackers. |
