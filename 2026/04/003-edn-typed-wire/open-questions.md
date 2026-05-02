# Open questions — Q2 and Q3 deferred to first concrete consumer

Q1 was the load-bearing question. Resolved 2026-04-26 via /gaze on sketched handler trees — see `sum-style-resolution.md`. Style A wins (zero L1, zero L2 against Style B's 1 L1 and 4 L2s); applied uniformly to `Option`, `Result`, `HolonAST`, and future user enums.

Q2 and Q3 below remain open. Neither blocks implementation; each can be settled when a concrete consumer forces a choice.

## 2. Primitive tags — never, sometimes, or always?

In a homogeneous container, the inner primitives need no tags — they're known-typed from the container's type param:

```edn
#wat.core/Vec<i64> [1 2 3]   ;; the 1 2 3 are i64s by container constraint
```

But in heterogeneous Value contexts (untagged data, or HashMap with a Value-typed value), is there a case for explicit primitive tags?

```edn
{:foo #wat.core/i64 42  ;; explicit (defensive, redundant)
 :bar 42}               ;; implicit (parses as :i64 anyway)
```

### What favors never tagging primitives

- EDN's native primitives map 1:1 to wat's: `integer`/`float`/`string`/`boolean`/`keyword`. No wrapping required.
- Tagging primitives is redundant and clutters the wire format.
- Less to register, less to maintain.

### What favors sometimes tagging primitives

- In a Value-typed context, the consumer might want to distinguish "a literal i64" from "a thing that happened to be an i64." Tagging is honest about which.
- For programs-as-holons, an `Atom<i64>` is conceptually different from a bare `i64` — the atomization adds semantic content. So `#wat.holon/Atom #wat.holon/I64 42` is genuinely different from bare `42`.

### Tentative lean

Never tag primitives in their own right. But DO tag wrappers around primitives that have semantic content (`Atom<T>`, `Linear<f64>`, `Log<f64>` — the wat scalar forms that add meaning). The line is: tag when the type adds information beyond "raw value of this kind."

## 3. Reader aliases — short forms vs strict FQDN-only

Full FQDNs are correct but verbose. `#wat.core/HashMap<String_i64>` is 28 characters before the body. For hand-written wat code reading EDN literals (test fixtures, examples, debugging), ergonomics could matter.

Option: a config-level alias map on the reader:

```scheme
(:wat::edn::set-aliases!
  {:v       :wat::core::Vec
   :map     :wat::core::HashMap
   :set     :wat::core::HashSet
   :h       :wat::holon::HolonAST})

;; then both forms resolve to the same handler:
#v/Vec<i64> [1 2 3]
#wat.core/Vec<i64> [1 2 3]
```

### What favors aliases

- Hand-written EDN fixtures stay readable.
- Test data is less noisy.
- The `wat.core` prefix is the same on every line of dense EDN — strong invariant, low information.

### What favors strict FQDN-only

- No registry-time configuration drift. Two programs reading the same EDN must agree on what every tag means. FQDNs are content-addressed; aliases require the alias table to be in sync across all readers.
- Clojure interop is more honest: a Clojure consumer reading `wat.core/Vec<i64>` can act on the FQDN without consulting a wat-side config.
- User said "we need fqdn names" — explicit commitment.

### Tentative compromise

**Asymmetric: aliases on read, FQDN-only on write.**

The reader supports an optional alias map (resolves alias → FQDN → handler). The writer ALWAYS emits FQDN form. The wire format has one canonical shape; ergonomic short forms are a reader-side convenience that doesn't pollute the wire.

This preserves the user's "FQDN names" commitment for the wire format while giving hand-writers an opt-in ergonomic surface. Test fixtures can use `#v/Vec<i64>`; round-trip through a writer always emits `#wat.core/Vec<i64>`. Two programs reading the same canonical EDN see the same tags; programs reading hand-written EDN with aliases configured see the resolved FQDN.

But this is a UX call worth chewing on. The user may reject the asymmetry.

## What unblocks each question

- Question 2 (primitive tags): a concrete consumer that uses Value-typed contexts (e.g. a generic JSON-like config reader). Until then, the principle "never tag primitives in their own right" is fine.
- Question 3 (aliases): hand-written-EDN volume. If first consumers are all programmatic (writer emits, reader consumes), strict FQDN works. If users start writing EDN fixtures by hand, aliases pay for themselves.

Neither blocks the first slice. Implementation can proceed with: Style A everywhere (resolved), no primitive tags, FQDN-only — and revisit if a consumer's needs argue otherwise.
