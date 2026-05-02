# Proposal — Inverse Hologram Experiment

**Status:** drafted in scratch 2026-04-28. To migrate to `holon-lab-trading/docs/proposals/2026/04/060-inverse-hologram-experiment/PROPOSAL.md` once infra clears the lab dir.

**Date frame:** ~36 hours to first data; tuning iterates from there.

**Architecture references (sibling files in this scratch):**
- `README.md` — top-level structural shape
- `CASCADE-ARCHITECTURE.md` — pipeline + per-tier label spaces + treasury responsibilities
- `NOISE-FILTERING-AND-ENGRAMS.md` — Phase 2 stripe-decomposition concept
- `MAGNITUDE-BOUNDING.md` — symmetric weight formula + ScaleTracker role
- `OPEN-QUESTIONS.md` — deferred decisions called out below

**Substrate predecessors (already shipped):**
- Arc 070 — `:wat::eval::walk` + `WalkStep<A>` + `AlreadyTerminal`
- Arc 071 — lab harness parity for parametric variant constructors
- Arc 074 — `Hologram` + `HologramCache` (substrate cache primitive used here, twice per fleshed tier)
- Arc 080–082, 087, 096 — telemetry stack (`LogEntry::Telemetry`, `MetricsCadence`, `RunDbService`)
- Arc 083–085 — `wat-sqlite` for L3 / runs-DB persistence
- Arc 095 — service paired channels for the cascade pipeline
- Arc 057 — typed HolonAST leaves; algebra closed under itself
- Arc 023 / 024 — `coincident?` / `presence?` primitives + sigma config
- EngramLibrary + StripedSubspace (existing substrate; consumed in Phase 2 only)

**Lab predecessors:**
- Proposal 056 — thought architecture; the indicator-rhythm pyramid this experiment ports faithfully
- Proposal 059 — the trader on substrate (the umbrella); 059-001 (L1/L2 caches) lands the cache surface; this proposal is a sibling experiment that uses the same caches differently
- `archived/pre-wat-native/src/encoding/{rhythm,scale_tracker,thought_encoder}.rs` — reference implementations to port
- `archived/pre-wat-native/src/domain/{market_observer,broker,treasury}.rs` — reference behavior

---

## What this experiment proves

The inverse architecture works on real BTC data: predictions emerge from substrate-native population reading without learned subspace + reckoner. Per the scratch's structural sketch, the trader's three thinkers (market, exit, broker) each become labeled-hologram-pairs; treasury fires labels at paper-trade close; the prediction walker reads label populations leftward through the prev-cache; the substrate's geometry IS the learner.

The pre-wat-native system was disappointing — raw rhythm cosine of 0.72–0.80 between regimes, only separable after a learned `OnlineSubspace.residual()` strip. Three serial fitting loops; cold start has no edge. This experiment tests whether the substrate's coincident? + Hologram cosine readout do the discrimination work without the three learned layers.

## Performance contract

**Primary gate: throughput.** Beat the pre-wat-native baseline (~7 candles/sec) decisively. Target: ≥272 candles/sec sustained on a 10k-candle representative run, matching proposal 059-001's contract.

**Telemetry observed during runs (NOT gates):**
- Hit rate when market signals grace+dir → fraction that closed grace+dir
- Signal vs abstain ratio over N candles
- Cache fill rate (distinct coords with labels)
- Walk depth distribution (mean / median / p99)
- P&L distribution per closed trade

The intelligence claim is observed in telemetry; tuning happens between runs. No a-priori target number for hit rate — let the data show what's plausible.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::holon::Hologram` (unbounded coordinate-cell store, cosine readout) | shipped (arc 074 slice 1) |
| `:wat::holon::lru::HologramCache` (bounded sibling, pure-wat over wat-lru) | shipped (arc 074 slice 2) |
| `:wat::holon::Bigram` / `Ngram` (idiomatic position-aware composition) | shipped |
| `:wat::holon::Thermometer` (locality-preserving scalar encoder) | shipped |
| `:wat::holon::Bind` / `Bundle` / `Permute` (the algebra core) | shipped |
| `:wat::eval::walk` + `WalkStep<A>` (the walker fold) | shipped (arc 070) |
| Service program shape + paired channels | shipped (arc 095) |
| Telemetry pipeline + `RunDbService` + `MetricsCadence` | shipped (arc 080–082, 087, 096) |
| `wat-sqlite` for runs DB | shipped (arc 083–085) |
| `:wat::holon::filter-coincident` / `filter-present` / `filter-accept-any` factories | shipped (with arc 074 follow-up) |
| Parquet → ohlcv stream → candle adapter | exists in archived; needs wat port (slice 1 work) |

## What's missing (this proposal)

| Op / change | What it does |
|----|----|
| `:trading::scale-tracker::ScaleTracker` (port) | EMA of |weight|; alpha caps at 1/100 after warmup; `scale = round_to(2.0 * ema_abs, 2)`. Per-tier instance. |
| `:trading::market::indicator-rhythm` (port from proposal 056) | The full pyramid: per-candle facts → trigrams → bigram-pairs → bundle, but encoded right-to-left (newest candle first; predecessors derived as we walk back). |
| `:trading::market::thinker` | Forward writer (encode + prev-cache deposit) + outcome writer (lineage walk + label-cache deposit) + prediction reader (leftward walk + grace-bias navigation + terminal label). |
| `:trading::exit::relay` | Forward identity + backward identity. Phase 1 only. |
| `:trading::broker::relay` | Forward identity + backward identity. Phase 1 only. |
| `:trading::treasury::Service` | Settles paper trades; observes outcomes; computes severity + weight; fires label backward to market (Phase 1 only). Counterfactual machinery stubbed for Phase 2. |
| `:trading::cascade::tuple` | Forward data structure: `(candle, market-thought, market-bias, exit-thought, exit-bias, broker-thought, broker-action)`. Each tier appends; nothing gates. |
| Telemetry hooks across all four programs | Walk depth, cache fill, signal ratio, hit rate per the diagnostics list. Per-program counters batched through `RunDbService`. |
| `:trading::config::set-*!` knobs | ScaleTracker alpha cap; walk presence floor; violence-terminate threshold; grace-bias tie-breaking. All surface as configurable; no recompilation between tuning sessions. |

Eight pieces. Most of the cost is the indicator-rhythm port (the pyramid is rich); the rest is wiring + telemetry.

---

## Slices

Six slices, ordered to land throughput-running-end-to-end as fast as possible. Each slice's "ships" describes the user-facing surface; "tests" describes what passes for slice-acceptance.

### Slice 1 — ScaleTracker port + Treasury skeleton

**Ships:**
- `wat/scale-tracker/ScaleTracker.wat` — wat port of `archived/pre-wat-native/src/encoding/scale_tracker.rs`. Single struct with `update`, `scale`, `count` methods. Alpha caps at 1/100 after warmup (configurable knob).
- `wat/treasury/Service.wat` — service program. Receives close-trade signals from broker (Phase 1: simulated by the test driver). Computes:
  - `raw_pnl_pct = (close - open) / open`
  - `weight = abs(close - open) / max(close, open)`
  - `severity = "grace" if raw_pnl_pct > 0 else "violence"`
  - Updates `ScaleTracker(weight)` → current `scale`
  - Builds label: `Bind(Bigram(action, severity), Thermometer(weight, 0.0, scale))`
  - Fires the label backward (Phase 1: only to market via channel)

**Tests (slice 1 probe):**
- T1-1: ScaleTracker converges per the rust port's existing tests (carry over verbatim).
- T1-2: Treasury settles a synthetic trade `(open=100, close=103)`; emits `Bind(Bigram("up","grace"), Thermometer(0.029, 0, scale))`.
- T1-3: Treasury settles a synthetic trade `(open=100, close=97)`; emits `Bind(Bigram("up","violence"), Thermometer(0.030, 0, scale))`.
- T1-4: Telemetry rows land in rundb for each settled trade.

### Slice 2 — Market thinker structure (cache scaffolding only)

**Ships:**
- `wat/market/Thinker.wat` — service program with two `HologramCache<HolonAST,HolonAST>` instances:
  - `prev-cache` keyed on encoded state, values are predecessor coords (per Q1 first cut: single-edge chain; predecessor = previous candle's encoded state).
  - `label-cache` keyed on encoded state, values are bundled label populations (initially empty; populated by outcome writer in slice 3).
- A stub encoder that produces a HolonAST for each candle. Stub for slice 2 — the full pyramid lands in slice 4. Slice 2 uses a single `Bind(Atom("rsi-thought"), Thermometer(rsi-value, 0, 100))` per candle so the cache machinery can be exercised.
- Forward writer: each candle → encode → put into prev-cache with prev pointer.

**Tests (slice 2 probe):**
- T2-1: Forward writer fills prev-cache as candles arrive.
- T2-2: Querying prev-cache by current state returns the previous coord.
- T2-3: Two distinct candles produce distinct encoded coords (sanity).
- T2-4: Cache holds `len > 0` after N candles via `HologramCache/len`.

### Slice 3 — Outcome writer + label-cache population

**Ships:**
- Connects treasury → market via paired channel for backward labels.
- When a label arrives at market, walk the lineage in prev-cache (held trade's open → close range), depositing the label at each coord:
  - `label-cache.put(coord-pos, coord-h, current-label-bundle ⊕ new-label-bind)`
  - The "current-label-bundle" is the existing value; new label gets bundled in.
- The "lineage" is the sequence of coords visited between trade open and trade close — the walker uses prev-cache to reconstruct it.

**Tests (slice 3 probe):**
- T3-1: A simulated trade opens at coord-A, closes at coord-D. After treasury fires the label, label-cache has labels at A, B, C, D.
- T3-2: Cosine readout at coord-D against `Bigram("up","grace")` returns positive presence after a grace deposit.
- T3-3: Repeated deposits at the same coord accumulate (bundle grows; cosine against the dominant Bigram strengthens).

### Slice 4 — Prediction reader + walk navigation

**Ships:**
- `wat/market/predict.wat` — given a current state, walk leftward through prev-cache, gated by:
  - presence? on state-found in prev-cache
  - max grace presence ≥ floor (the categorical-only navigation per Q-cascade-6 (a))
- Per-step rule:
  - Cosine probe against all 4 Bigrams in label-cache
  - `strongest = max(cosines)`
  - If strongest ∈ {grace_*} → continue to prev-cache.get(state)
  - If strongest ∈ {violence_*} → terminate; emit "abstain" + the violence Bigram + magnitude
  - If no presence clears floor → terminate; return strongest grace seen (or abstain if never)
- Emit market-bias forward in the cascade tuple.

**Tests (slice 4 probe):**
- T4-1: A populated label-cache (10 trades' worth of grace-up deposits) with a current state coincident with one of them returns `Bigram("up","grace")` at expected magnitude.
- T4-2: A current state with violence-strongest at the surface immediately abstains.
- T4-3: A current state with no presence in prev-cache (cold lookup) returns abstain.
- T4-4: Walk depth telemetry surfaces (mean steps before terminate).

### Slice 5 — Cascade pipeline + relays + end-to-end

**Ships:**
- `wat/exit/Relay.wat` — forward identity, backward identity. Always emits `"hold"` for exit-bias.
- `wat/broker/Relay.wat` — forward identity, backward identity. Always trusts market signal; opens position when market signals grace; closes on expiration only.
- The cascade tuple wired through paired channels: parquet → ohlcv → candle → market → exit → broker → treasury, with backward labels treasury → broker → exit → market.
- A driver binary that reads parquet, runs the pipeline, lets it execute paper trades, observes outcomes.

**Tests (slice 5 acceptance):**
- T5-1: Pipeline runs against 1000 candles from `data/btc_5m_raw.parquet`. No deadlocks. No data loss.
- T5-2: Trades open and close per broker's logic; treasury settles them; labels fire back; market's caches grow.
- T5-3: Throughput ≥ 272 candles/sec sustained on the 10k-candle benchmark.
- T5-4: Telemetry rows for all key counters land in rundb queryable via SQL.

### Slice 6 — Telemetry instrumentation (woven through slices 1–5; called out separately)

**Ships:**
- Counters per program: `lookups`, `hits`, `misses`, `evictions`, `cache-size`, `walk-depth`, `signals-emitted`, `abstains-emitted` (market). `trades-opened`, `trades-closed`, `expirations`, `early-exits`, `pnl-distribution-p50`, `pnl-distribution-p90` (treasury/broker). All written through `RunDbService` per arc 080–082's pattern.
- A standard SQL query bundle in `holon-lab-trading/runs/queries/` for tuning sessions:
  - `cache-fill-by-tier.sql` — number of distinct coords with labels per tier
  - `signal-vs-abstain.sql` — ratio over time
  - `hit-rate-when-grace.sql` — of trades signaled grace, what fraction closed grace
  - `walk-depth-distribution.sql` — histogram of walk lengths
  - `throughput.sql` — candles processed per minute over time

**Tests:**
- T6-1: All counter rows land in rundb during a 100-candle smoke test.
- T6-2: Each query in `runs/queries/` runs without error against the fresh rundb.

---

## Knobs (configurable; surface in `wat/config/`)

| Knob | Default | Purpose |
|---|---|---|
| `:trading::config::scale-alpha-cap-inverse` | 100 (alpha = 1/100) | ScaleTracker's slow-EMA floor. Tune higher for slower drift, lower for faster adaptation. |
| `:trading::config::walk-presence-floor` | substrate's `coincident-floor(d)` | The "leans enough" threshold per walk step. |
| `:trading::config::violence-terminate-floor` | substrate's `coincident-floor(d)` | Below this, violence isn't strong enough to force abstain. |
| `:trading::config::grace-bias-tie-break` | "up" | When grace-up = grace-down at a step, which wins. |
| `:trading::config::trade-expiration-candles` | 12 (≈ 1 hour at 5-min) | Max held trade duration before forced close. |

All readable from wat code at startup. No recompilation between tuning sessions; user edits a config file (or sets via CLI flags), restarts.

---

## Acceptance criteria (for declaring Phase 1 done)

- All slice probe tests pass.
- T5-3 throughput ≥ 272 candles/sec sustained on 10k.
- 1000-trade end-to-end run completes without deadlock; outcomes settle; labels propagate; market's caches populate.
- All telemetry SQL queries from slice 6 return non-empty rows on the run's rundb.

What's NOT in acceptance:
- A specific hit rate (defer to telemetry observation; tune from there).
- Engram promotion or StripedSubspace (Phase 2).
- Exit and broker fleshed (Phase 2).
- Counterfactual simulation when broker vetoes (Phase 2).

---

## Deferred decisions (called out in OPEN-QUESTIONS.md, addressed during slice work)

| Decision | When | Default for first build |
|---|---|---|
| Q1 — single-edge vs multi-edge prev-cache | Slice 2 | single-edge (immediate predecessor); revisit if multi-lineage smear shows in telemetry |
| Q2 — terminal label readout shape | Slice 4 | label at deepest-confident terminal (the user's literal framing); strongest-along-walk surfaces in telemetry as alternative |
| Q-cascade-6 — magnitude in walk navigation | Slice 4 | categorical-only probe (option a); magnitude read at terminal only |
| Engram promotion threshold | Phase 2 | not in Phase 1 |

---

## Phase 2 follow-ups (not this proposal)

- Engram-cache: saturation detector → StripedSubspace decomposition → multi-stripe readouts. Solves multi-lineage coord smear.
- Exit fleshed: own labeled-hologram-pair; own thinker; own bias.
- Broker fleshed: own labeled-hologram-pair; own veto power; counterfactual labeling.
- Treasury counterfactual machinery: when broker vetoes, simulate the path-not-taken.
- L2/L3 cache promotion: lab umbrella 059-001's HologramCache pattern at the per-tier level — share market's caches across multiple market-observer instances; persist across runs to L3 SQLite.

---

## What this DOESN'T do

- **No reckoner.** The substrate's cosine readout replaces it.
- **No `OnlineSubspace.residual()`.** Phase 2 engrams handle multi-lineage decomposition; Phase 1 lives with the bundle.
- **No multi-lens market observer.** One lens (the full proposal-056 pyramid). Multi-lens is a future concern.
- **No multi-asset.** BTC only.
- **No live trading.** Paper only.
- **No Phase 2 substrate work.** Engrams + counterfactuals + exit/broker thinking land later.

PERSEVERARE.
