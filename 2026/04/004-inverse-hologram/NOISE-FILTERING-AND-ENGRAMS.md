# Noise filtering and engrams — derived from cascade Q-cascade-7

The user's realization: *"we just arrived at the realization we need noise filtering again... strip noise and then do work... b+c just emerged as a requirement through deduction."*

Engrams are not optional. The architecture requires noise filtering at the label-cache layer. Where the pre-wat-native system used `OnlineSubspace.residual()` to strip background BEFORE classifying, this architecture uses StripedSubspace decomposition AT THE LABEL POPULATION to clean multi-lineage coords.

It is NOT the OnlineSubspace coming back as a separate stage. It is the label-cache itself becoming smarter about contested coordinates.

---

## The problem — multi-lineage coords smear

A coordinate that lies on the path of GENUINELY DIFFERENT lineages accumulates contradictory deposits. Example: state X was visited during 50 grace-up trades and 30 violence-down trades over 6 months. Its label-cache value is:

```
Bundle(
  Bind(Bigram(grace, up),    Therm 0.5),  ; deposit 1
  Bind(Bigram(grace, up),    Therm 0.7),  ; deposit 2
  ...50 grace-up deposits total...
  Bind(Bigram(violence, dn), Therm 0.4),  ; deposit 51
  ...30 violence-down deposits total...
)
```

Cosine readout against `Bigram(grace, up)` returns the average direction-magnitude of the bundle's grace-up component. Same for violence. But there's a problem: **the bundle's accumulated direction is the SUM of all 80 deposits**, which smears.

A naive readout sees "this coord has both grace-up history AND violence-down history." Cosine to either Bigram fires partially. The walker can't tell:

- "Strong grace-up history; mostly grace lineages; safe to follow"
- "Strong grace-up AND violence-down history; contested; dangerous to trust"

Both produce mid-strength cosines against grace-up. The walker can't distinguish them without decomposing the bundle.

---

## The fix — StripedSubspace decomposition (the engram concept)

Engrams in this architecture are **multi-stripe decompositions of saturated label-cache coords.** When a coord accumulates enough deposits that the bundle becomes interpretable as multi-modal, it gets PROMOTED to an engram with explicit stripes.

```
Engram at coord X:
  stripe_grace_up:    learned manifold of the 50 grace-up deposits
  stripe_violence_dn: learned manifold of the 30 violence-down deposits
  ; (other stripes for any classes that fired here)
```

Each stripe is a clean direction representing ONE mode of behavior at that coord. The walker queries each stripe separately:

```
walker at coord X:
  for each stripe in engram:
    cosine(probe, stripe.direction) → presence-strength for that stripe
  decision:
    if grace_up stripe is strong AND violence_down stripe is weak: continue toward grace_up
    if grace_up stripe is strong AND violence_down stripe is also strong: ABSTAIN — contested coord
    if violence_down dominates: ABSTAIN
    if no stripe has strong presence: terminate, return what we found so far
```

The "strip noise" the user named: noise IS contested-direction in the bundle. Stripping = decomposing the bundle into clean stripes so the walker reads each separately rather than averaging.

---

## When does an engram form

Three triggers (likely all needed):

1. **Saturation count.** Coord has more than threshold-N deposits. (E.g., > 50 deposits, or > √d as a Kanerva-aligned ceiling.)
2. **Multi-modal evidence.** Cosine readout against ALL 8 base Bigrams (4 categories × 2 magnitudes) shows that more than one fires above floor. The bundle is genuinely multi-modal, not just one class with magnitude variance.
3. **Direction stability stops drifting.** New deposits don't substantially change the bundle direction. The accumulated information is representative.

Once these conditions are met, the cache promotes the coord from `label-cache` to an `engram-cache` (the EngramLibrary per substrate convention). The engram-cache uses StripedSubspace internally; lookups are stripe-separated.

The walk's per-step query consults engram-cache FIRST (fast, decomposed); falls through to label-cache (slow, raw bundle) for non-promoted coords.

---

## Substrate support already shipped

Per memory `project_trading_lab.md`: *"EngramLibrary supports StripedSubspace encoding for multi-modal libraries."*

The substrate's EngramLibrary + StripedSubspace are already the right shape. This architecture consumes them; no new substrate work needed for the engram layer specifically.

(Whether HologramCache's value type is HolonAST or EngramSnapshot is an open question — see follow-up below.)

---

## Phase progression for the engram concept

**Phase 1** (first cut, no engrams):
- Single label-cache per tier holding raw bundles.
- Walker reads bundle directly; cosines against each of 4 Bigrams; picks max.
- Multi-lineage coords smear. Acceptable while we learn what the smear looks like.

**Phase 2** (engrams active):
- Saturation detector promotes coords from label-cache → engram-cache.
- Walker queries engram-cache first; falls through to label-cache for unpromoted coords.
- Stripe-separated readouts let the walker distinguish "clean grace-up history" from "contested grace-up + violence-down history."

**Phase 3** (engrams as primary):
- Most coords are engrams. Label-cache becomes the staging layer for new observations.
- Engram updates happen periodically (recompute stripes from label-cache state).
- Throughput stabilizes.

The first experiment ships Phase 1 — single-bundle label-cache per tier. Phase 2 ships when readout quality on multi-lineage coords becomes the bottleneck. Phase 3 is throughput-driven.

---

## Walk navigation under noise — Q-cascade-8 resolved

The user's interesting case: at a step, **grace-up presence is 0.6 AND violence-down presence is 0.7**. What does the walk do?

**Resolution: violence dominance triggers abstain at the surface or mid-walk.**

```
each step:
  cosines = {grace_up, grace_down, violence_up, violence_down}
  strongest = max(cosines)
  if strongest is in {grace_up, grace_down}:
    continue walking that direction
  if strongest is in {violence_up, violence_down}:
    terminate; ABSTAIN; the substrate's signal at this coord is "danger"
  if no cosine clears floor:
    terminate; return strongest grace seen along the walk (or abstain if never had grace)
```

Grace-bias on EQUALITY (where two cosines tie at the same value) — yes, grace wins (the user's "(a) wins it" answer for the equality case). But asymmetric — when violence is genuinely stronger, the walk respects that signal: violence dominance means contested or dangerous neighborhood, regardless of grace's presence.

The `(a)` answer for equality + the `terminate-on-violence-strongest` rule for asymmetric cases together form the navigation rule.

---

## Open

- **The engram-cache type.** The HologramCache's value side is HolonAST. If engrams are HolonAST-shaped (a Bundle or Bind structure encoding the stripes), they fit the existing cache. If engrams need a different value shape (e.g., a struct carrying multiple Vectors per stripe), this is new substrate work. Need to study the substrate's EngramLibrary to verify.
- **Saturation thresholds.** The "promote to engram when..." conditions (count > N, multi-modal evidence > floor, drift < epsilon). All knobs. Pick defaults; let observation tune.
- **ScaleTracker behavior.** See OPEN-QUESTIONS Q-cascade-9 — needs study of the archived implementation.
