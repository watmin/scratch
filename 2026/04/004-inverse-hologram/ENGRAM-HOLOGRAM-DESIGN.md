# EngramHologram — design notes (in progress)

**Started:** 2026-04-28.
**Status:** mid-design. Not yet locked. Resuming after infra wraps the lab side.

---

## Why this is being designed

The PROPOSAL.md's Phase 1 had `Hologram` (unbounded; arc 074 slice 1) as the cache. That's wrong:

- **Unbounded** means memory grows forever; not viable for long runs.
- **HologramCache** (LRU) is also wrong — LRU evicts based on access recency. Labels are intuition; a coord that hasn't been queried recently but bears a hundred grace-up deposits is MORE valuable, not less. LRU's polarity is backwards for durable wisdom.

What's needed: a Hologram variant that **bounds memory through compaction, not eviction.** When per-slot pressure arrives, similar entries get merged (information preserved by superposition); only when nothing is mergeable does anything get dropped (and what's dropped is the lowest-supported intuition, not the least-recently-accessed).

EngramLibrary + StripedSubspace existed in the holon-rs library to handle multi-modal pattern matching, but the *holographic interface* (cosine readout against arbitrary probes, presence?, position-keyed slot lookup) wasn't part of that. This design names what fills the gap.

---

## What a "merge" actually does (the load-bearing risk to understand)

Two entries A and B at the same coord:

```
A = Bind(Bigram(grace, up), Thermometer(0.50, 0, 0.06))
B = Bind(Bigram(grace, up), Thermometer(0.55, 0, 0.06))

Bundle(A, B) = element-wise sum of encoded vectors,
               then bipolar-threshold (sign of each component)
               to stay in {-1, 0, +1}^d.
```

Three effects:

1. **Direction is preserved** for components where A and B agree (most of them, since A and B are coincident).
2. **Bipolar threshold is lossy at the dim level.** When A votes +1 and B votes +1, the sum 2 → threshold +1 (OK). When A votes +1 and B votes -1, the sum 0 → tiebroken (information collapses). The arithmetic loses signal in disagreement-dimensions.
3. **Count becomes invisible.** A bundle of 1 vector and a bundle of 100 vectors all pointing the same direction produce the SAME bipolar-thresholded result. The substrate cannot tell from the vector alone how many deposits contributed.

That third effect is the load-bearing risk. **A coord with 1 grace-up deposit and a coord with 1000 grace-up deposits have the same vector after bundling — they look identical to cosine readout.** The "I am confident" axis disappears.

This is why the user said "the merge is risky." It's not just smear from non-coincident merges — even SAFE merges erode the population's confidence axis.

---

## The fix — store count alongside the bundle

```
struct EngramEntry:
  bundle  : HolonAST       ; the geometric direction (substrate-bundled, bipolar)
  count   : i64             ; how many original deposits got merged in here
  age     : i64             ; first-deposit timestamp (or rule-TBD per Q-engram below)
```

**This is the missing substrate primitive.** The HologramCache's value side today is `HolonAST → HolonAST`. The EngramHologram's value side is `HolonAST → Vec<EngramEntry>` (or however many entries fit per slot, with compaction on overflow).

When merging:

```
merged.bundle = Bundle(A.bundle, B.bundle)     ; substrate's bipolar bundle
merged.count  = A.count + B.count               ; counts add (out-of-band)
merged.age    = min(A.age, B.age)               ; or weighted-mean — TBD
```

Count is preserved out-of-band of the geometric bundle. Cosine readout uses the bundle's direction; consumer reads count separately to know "this engram's confidence" beyond geometric direction alone.

Two coords with the same direction but counts {5, 5000} have the same cosine but different intuition-strength — the consumer reads both.

---

## The merge rule (strict, designed for stripes-emerging)

```
EngramHologram per-slot has up to N EngramEntry instances.

put(coord, new_label):
  ; new_label arrives as one EngramEntry with count=1
  
  best_match = argmax over slot.entries of cosine(e.bundle, new_label.bundle)
  
  if cosine(best_match.bundle, new_label.bundle) > merge_floor:
    ; STRICT — only merge if truly coincident
    best_match.bundle = Bundle(best_match.bundle, new_label.bundle)
    best_match.count += 1
    best_match.age = min(best_match.age, new_label.age)
  else:
    ; no existing entry is coincident enough to merge
    if slot.entries.len() < N:
      append new_label as a new entry (count=1)
    else:
      ; pressure: must compact existing entries to make room
      compact_pair_merge(slot)
      append new_label

compact_pair_merge(slot):
  ; find the two most-coincident existing entries
  pair = argmax over (i, j) of cosine(slot.entries[i].bundle, slot.entries[j].bundle)
  if pair_cosine > merge_floor:
    ; mergeable — bundle them
    merge_into(slot.entries[pair.i], slot.entries[pair.j])
    drop slot.entries[pair.j]
  else:
    ; nothing in the slot is mergeable — the bad case
    ; policy: drop the lowest-count entry (least-supported intuition)
    drop_lowest_count(slot)
```

### The strict floor is the load-bearing parameter

Set `merge_floor` to the substrate's `coincident-floor(d)` (the σ/√d at the encoded dim). Things that pass the substrate's coincident? predicate CAN merge; below that, they're treated as different stripes.

- Set too loose → smear (grace-up entries merge with violence-down entries; bundle becomes mush)
- Set too tight → frequent fall-through to drop_lowest_count → information loss
- At substrate-floor — STRICT: things merge only if the substrate already considers them the same point on the algebra grid

### Why this is "stripes-aware automatically"

The strict floor rule means a grace-up deposit and a violence-down deposit will NEVER merge — their bundles are quasi-orthogonal; cosine far below floor. They live as separate entries in the slot. The slot naturally holds stripes:

```
slot at coord X (after many trades):
  entry[0]: bundle of grace-up deposits        (count = 247)
  entry[1]: bundle of violence-down deposits   (count = 38)
  entry[2]: bundle of grace-down deposits      (count = 14)
  entry[3]: ...
```

No explicit "stripe" type. Stripes emerge from the merge rule. Each entry IS a stripe — a bundle of mutually-coincident deposits.

---

## Risks and mitigations

| Risk | Mechanism | Mitigation |
|---|---|---|
| **Smear from too-loose merge_floor** | Things that "look similar" but represent different lineages get merged; bundle becomes ambiguous | Set `merge_floor = coincident-floor(d)` exactly; tighter than floor = strictly coincident only |
| **Direction drift from cumulative merges** | Merge A+B, then merge result with C; result averages A,B,C with A and B already weighted twice in the bundle | Count tracks original deposit-count; the bundle's direction reflects the population's CURRENT direction; drift IS the population's drift — acceptable |
| **Count overflow** | Trades pile up; count grows unbounded | i64 holds 9.2e18; not a concern at trade frequency |
| **Age semantics after merge** | Multiple deposits, multiple timestamps; only one EngramEntry | Use `min(age)` for "earliest" semantics; or `weighted_mean(age, weights=counts)` for "centroid" semantics — see Q-engram-5 |
| **Eviction when nothing is mergeable** | Slot full; no two entries are coincident; something must go | Drop lowest `count` — the "least-supported intuition" loses. This IS the only honest information loss; everything else preserves geometrically + via count |
| **Strict floor → frequent eviction** | If the slot regularly has N truly-orthogonal stripes (multi-modal lineage space), eviction fires often; we lose rare-deposit stripes | Increase per-slot N (the working memory). Substrate's √d cells bound the total slot count; per-slot N is the engram-richness knob |

---

## Open design questions

### Q-engram-1 — Where does the count live in the AST representation?

Two readings:

- **(a) EngramEntry is a substrate struct distinct from HolonAST.** Cache's value-side is `Vec<EngramEntry>`, not `Vec<HolonAST>`. Honest about count being metadata. Substrate-side change required.
- **(b) Encode count INTO the HolonAST.** Bind a `Thermometer(log(count), 0, log_max)` or similar into the entry. Then the cache value is just HolonAST. No new substrate type but awkward — count isn't really a "geometric" axis; encoding it as one risks confusing readouts.

**My instinct: (a).** EngramEntry as a substrate struct. The cache becomes `EngramHologram<HolonAST>` with EngramEntry as the per-slot entry shape.

### Q-engram-2 — Original deposits durable, or only the merged bundle?

If we ever want "show me the 5 grace-up deposits that contributed to this engram" — we'd need to keep originals.

For Phase 1: only the engram (merged bundle + count). Original deposits flow through ScaleTracker (which sees them) but aren't separately stored. If forensic traceback becomes a need, a durable SQLite log of every deposit ships as an L3 layer in a future arc.

### Q-engram-3 — Readout structure when probe arrives at a slot

The slot has N EngramEntries (stripes). Two readout shapes:

- **(a) Probe-vs-each:** cosine probe against each entry; pick max; return that entry (with its count as confidence).
- **(b) Probe-vs-known-Bigrams:** cosine probe against EACH of the 4 known label Bigrams (g-up, g-down, v-up, v-down) directly (skipping the slot entries entirely); aggregate.

(a) is what the walker per-step query needs for navigation. (b) is more diagnostic.

For the inverse-hologram experiment specifically: (a) — the walker probes the slot's stripes and picks the strongest grace one to follow.

### Q-engram-4 — Compaction on read vs on write?

- **(a) On write (synchronous):** every put that overflows N triggers compact_pair_merge before append. Simple, predictable; pays cost on writes.
- **(b) On read (lazy):** writes always append (slot grows over N temporarily); when read happens, notice overflow, compact then return result. Spreads cost across reads; introduces async race if multi-thread.

**For Phase 1: (a).** Synchronous on write. Simple. Predictable. If perf hurts, switch to lazy.

### Q-engram-5 — Age semantics after merge

When A and B merge, what's `merged.age`?

- **(a) `min(A.age, B.age)`** — earliest contribution. "When did this engram first start forming."
- **(b) `weighted_mean(A.age, B.age, weights=A.count, B.count)`** — count-weighted mean timestamp. "Centroid" of the engram's deposit history.
- **(c) `max(A.age, B.age)`** — latest contribution. "When was this engram last reinforced."

Each captures different semantics. (a) for "intuition lineage start." (b) for "average era." (c) for "recent reinforcement."

Probably (a) for a first cut — it answers "when did we first start trusting this state's lean?"

### Q-engram-6 — Should the EngramHologram support eviction-by-age too?

Even with strict-merge + drop-lowest-count, an engram with high count from years ago might dominate forever even if the market regime has shifted. Should we eventually retire old engrams?

Possible mechanisms:
- **(a) No age-based eviction.** Count rules.
- **(b) Decay count by age** — every engram's effective count = `count * decay^(now - age)`. Old engrams gradually lose voting weight; eventually drop_lowest_count catches them.
- **(c) Hard age cap** — engrams older than X get discarded.

This is the regime-shift question. Markets change. Old wisdom may no longer apply. But premature decay throws away durable patterns.

For Phase 1: (a). No age decay. Phase 2 reconsidered if telemetry shows old-engram dominance becoming a problem.

### Q-engram-7 — Can the EngramHologram be SQLite-backed for cross-run durability?

If we want intuition to survive process restarts:

- **(a) In-memory only.** Engrams reset every run.
- **(b) Snapshot-at-shutdown.** Save engrams to SQLite; reload on startup.
- **(c) Source-of-truth on disk.** Every put writes to SQLite (append-only); in-memory is a compacted projection of the durable record. Restart rebuilds the projection from disk.

(b) is simpler; (c) is honest about durability but heavier.

For the experiment first cut: (a). 36-hour window doesn't need cross-run durability — the EXPERIMENT itself is one run. (b) is an obvious next step once the architecture is validated.

---

## How EngramHologram fits the existing trait

```
trait HolonStore<K, V>:           ; same trait that's been the architecture lockstep
  put(k, v) → store
  get(k) → Option<V>

implementations:
  Hologram<K, V>            ; arc 074 slice 1 — unbounded, no eviction
  HologramCache<K, V>       ; arc 074 slice 2 — LRU eviction
  EngramHologram<K>         ; this design — compaction-by-merge eviction
                              ; (note: V is fixed at EngramEntry, not generic)
  HologramDB<K>             ; future — SQLite-backed
```

The trait stays — same put/get signature. Only the eviction policy and value-shape differ. The architecture-is-interface rule applies.

EngramHologram's value-side is fixed at `Vec<EngramEntry>` (since the merge semantics need the EngramEntry struct's count + age fields). It's not parametric over V the way HologramCache is. Or — alternatively — it COULD be parametric if EngramEntry is only the "default merge wrapper" and consumers can plug in their own.

---

## What needs to ship for the experiment

**Substrate-side (a real arc — call it 097-engram-hologram or whatever the next number is):**

1. `EngramEntry` struct (bundle + count + age) — registered as a substrate type
2. `EngramHologram<K>` cache — coordinate-keyed; per-slot Vec<EngramEntry>; merge-on-pressure
3. `EngramHologram/put` (k, label) → store — does the strict-merge + compact-on-overflow + drop-lowest-count
4. `EngramHologram/get` (k, probe) → Option<(EngramEntry, cosine)> — readout per Q-engram-3 (a)
5. `EngramHologram/len` (k) → i64 — how many entries in this coord's slot
6. `EngramHologram/stripes-at` (k) → Vec<EngramEntry> — list all entries (diagnostic)
7. Knobs: `merge_floor` (default = coincident-floor(d)), `per_slot_N` (default = ?? — TBD; probably √d)

**Consumer-side (in the experiment proposal — replaces "Hologram" everywhere it's mentioned):**

- Market's `prev-cache` stays `Hologram` (chain edges; never compacted)
- Market's `label-cache` becomes `EngramHologram` (the durable intuition)
- Treasury fires labels into market's `label-cache` per the existing flow

The chain-edge cache doesn't need merge semantics — chain edges are pointers between coords, not population codes. They're best-stored unbounded (or with a different eviction rule like "drop predecessors of the oldest reachable coord" — but Phase 1 ignores).

The label-cache is where compaction matters. That's where the EngramHologram lands.

---

## Next steps when work resumes

1. Settle Q-engram-1 (struct vs encoded-into-AST) — likely (a)
2. Settle Q-engram-7's first-cut answer (in-memory only for Phase 1)
3. Pick `per_slot_N` default — proposal: `√d` (matches the substrate's natural cell budget but at the per-coord level)
4. Update PROPOSAL.md to replace `Hologram` with `EngramHologram` for the label-cache; note that chain prev-cache stays as `Hologram`
5. Decide whether to draft a wat-rs arc for `EngramHologram` first OR ship the experiment with a lab-side EngramHologram-equivalent (slower path; more lab code) and promote later

The user said "we tackle these things not defer them" — so option 5 leans toward drafting the substrate arc properly. Will do once we resume.
