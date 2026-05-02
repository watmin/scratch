# Consumer bridge — what wat ↔ Clojure interop actually looks like

## Why this beat

After the crate-shape pivot landed (`wat_edn` as router, SymbolTable as authority), the question turned to the consumer side. User:

> "ok... so.. if we expose this a std crate... can you show me what a consumer might do here?... the whole point of this is to provide a bridge to clojure... i need apps on both sides to interop..."

This beat sketches the bridge end-to-end: wat program emits EDN, Clojure dashboard reads it; Clojure dashboard emits config back, wat reads. Both directions. Real types. Real code on both sides.

## Scenario

A trading enterprise running on wat emits trade signals to a stream (file or socket — same shape either way). A Clojure dashboard reads the stream, displays signals, lets the operator nudge sizing. The dashboard writes config updates back. The wat enterprise reads them on next tick.

Two struct types are the shared protocol:

```scheme
(:wat::core::struct :enterprise::observer::TradeSignal
  (asset       :Keyword)
  (side        :Keyword)            ; :Buy or :Sell
  (size        :f64)
  (confidence  :f64)
  (reasoning   :Vec<wat::holon::HolonAST>)
  (proposed-at :Instant))

(:wat::core::struct :enterprise::config::SizeAdjust
  (asset    :Keyword)
  (factor   :f64)
  (until    :Option<Instant>))
```

That's the entire type-side commitment on the wat end. Once these are in the SymbolTable, EDN read/write Just Works (per beat 8 — `wat_edn` walks SymbolTable metadata structurally).

## Wat side — emit and read

```scheme
;; --- Emit a signal (wat -> Clojure) ---
(:wat::core::define (:enterprise::observer::emit-signal
                     (sig :enterprise::observer::TradeSignal)
                     (out :wat::io::IOWriter)
                     -> :Result<:(), :wat::edn::EdnError>)
  (:wat::core::let* (((edn :String) (:wat::edn::write-str sig)))
    (:wat::io::write-line! out edn)))

;; --- Read a config update (Clojure -> wat) ---
(:wat::core::define (:enterprise::config::read-adjust
                     (in :wat::io::IOReader)
                     -> :Result<:enterprise::config::SizeAdjust,
                                :wat::edn::EdnError>)
  (:wat::core::let* (((line :String)
                      (:wat::core::try (:wat::io::read-line! in))))
    (:wat::edn::read-str-as<:enterprise::config::SizeAdjust> line)))

;; --- Use both in the heartbeat ---
(:wat::core::define (:enterprise::on-tick
                     (sig :enterprise::observer::TradeSignal)
                     (signals-out :wat::io::IOWriter)
                     (config-in   :wat::io::IOReader)
                     -> :())
  (:wat::core::try (:enterprise::observer::emit-signal sig signals-out))
  (:wat::core::match (:enterprise::config::read-adjust config-in) -> :()
    ((Ok adj) (:enterprise::apply-adjust! adj))
    ((Err _)  ())))   ; no adjustment available, carry on
```

No tag handlers registered. No serializer derives. wat_edn walks the SymbolTable for `:enterprise::observer::TradeSignal` at write time, finds the struct's field metadata, emits the EDN. Reverse for read. Application code is bridge-agnostic.

## EDN on the wire

```edn
;; signals.edn — what wat emits
#enterprise.observer/TradeSignal
{:asset       :BTC
 :side        :Buy
 :size        0.025
 :confidence  0.73
 :reasoning   #wat.core/Vec<wat.holon.HolonAST>
                [#wat.holon/Atom #wat.holon/Symbol :rsi-rising
                 #wat.holon/Bind
                   [#wat.holon/Atom #wat.holon/Symbol :flow
                    #wat.holon/Atom #wat.holon/Symbol :positive]]
 :proposed-at #inst "2026-04-26T14:30:00Z"}

;; config-updates.edn — what Clojure writes back
#enterprise.config/SizeAdjust
{:asset  :BTC
 :factor 1.5
 :until  #wat.core/Some<Instant> #inst "2026-04-26T16:00:00Z"}
```

This is the round-trip — the same EDN both sides see, byte-equivalent on either trip through the bridge.

## Clojure side — naive (every tag registered manually)

What it looks like WITHOUT any helper library, to make the cost honest:

```clojure
(ns enterprise.dashboard
  (:require [clojure.edn :as edn]
            [clojure.java.io :as io]))

;; A reader is just a fn (body) -> value. For tags whose body is
;; already idiomatic Clojure data, the reader is identity.
(defn passthrough [body] body)

;; For variant tags (Some, Ok, Err...), wrap to preserve identity.
(defn read-some [body] [::some body])
(defn read-none [body] [::none])
(defn read-ok   [body] [::ok body])
(defn read-err  [body] [::err body])

(def readers
  {;; Stdlib collections — body is already a Clojure vec/map/set
   'wat.core/Vec<wat.holon.HolonAST>     passthrough
   'wat.core/HashMap<String_f64>         passthrough
   'wat.core/HashSet<String>             passthrough
   ;; Sums
   'wat.core/Some<Instant>               read-some
   'wat.core/None<Instant>               read-none
   ;; HolonAST variants
   'wat.holon/Atom                       passthrough
   'wat.holon/Symbol                     passthrough
   'wat.holon/Bind                       passthrough
   'wat.holon/Bundle                     passthrough
   ;; Application types
   'enterprise.observer/TradeSignal      passthrough
   'enterprise.config/SizeAdjust         passthrough})

(defn read-signals [path]
  (with-open [r (java.io.PushbackReader. (io/reader path))]
    (loop [out []]
      (let [v (edn/read {:readers readers :eof ::done} r)]
        (if (= v ::done) out (recur (conj out v)))))))
```

This works but is tedious. Every wat tag the dashboard might see needs an entry. **That's the cost of EDN's tag dispatch model on the Clojure side** — `clojure.edn/read` doesn't have a SymbolTable to consult, so dispatch is per-tag manual.

## Clojure side — with a `wat-edn-clj` library

The asymmetry above is the gap a small Clojure library closes. Ship it once, consumers add a dep, the per-tag ceremony goes away:

```clojure
(ns enterprise.dashboard
  (:require [clojure.java.io :as io]
            [wat-edn-clj.core :as wat-edn]))

;; The library knows about every wat-stdlib tag. Consumers register
;; only application types — and even those default to passthrough.
(wat-edn/register-types!
  ['enterprise.observer/TradeSignal
   'enterprise.config/SizeAdjust])

(defn read-signals [path]
  (wat-edn/read-stream (io/reader path)))

;; Consume:
(doseq [sig (read-signals "signals.edn")]
  (println "Signal:" (:asset sig) (:side sig) "size" (:size sig)))
```

What `wat-edn-clj` does internally:

```clojure
;; 1. Default reader fn that recognizes the wat.* namespace prefix.
(defn wat-default-reader [tag body]
  (let [s (str tag)]
    (cond
      ;; Collections, HolonAST variants — body is already idiomatic
      ;; Clojure data. Strip the type tag, keep the data.
      (or (.startsWith s "wat.core/Vec")
          (.startsWith s "wat.core/HashMap")
          (.startsWith s "wat.core/HashSet")
          (.startsWith s "wat.holon/"))
      body

      ;; Sums — wrap to preserve variant identity
      (.startsWith s "wat.core/Some")  [::some body]
      (.startsWith s "wat.core/None")  [::none]
      (.startsWith s "wat.core/Ok")    [::ok body]
      (.startsWith s "wat.core/Err")   [::err body]

      ;; Unknown — preserve as tagged literal (graceful interop)
      :else (tagged-literal tag body))))

;; 2. print-method definitions for emitting wat-readable EDN back.
(defmethod print-method :enterprise.config/SizeAdjust
  [v ^java.io.Writer w]
  (.write w "#enterprise.config/SizeAdjust ")
  (print-method (into {} v) w))

;; 3. Helper macro that wires application types into both readers
;; and writers in one declaration — register-types! above.
```

The library is small (~200 LOC). Ships once. Versioned alongside `wat_edn` so the wire format stays in sync.

## Writing back (Clojure → wat)

```clojure
(defn emit-adjust [path adjust]
  (with-open [w (io/writer path :append true)]
    (binding [*print-dup* false]
      (wat-edn/print-line! w adjust))))

(emit-adjust "config-updates.edn"
  ^{:type :enterprise.config/SizeAdjust}
  {:asset :BTC
   :factor 1.5
   :until [::some #inst "2026-04-26T16:00:00Z"]})
```

`wat-edn/print-line!` knows that `[::some x]` writes as `#wat.core/Some<...> x` — with the type generic resolved from context, or omitted if the schema-less round-trip is acceptable.

## The asymmetry, named honestly

- **Wat side:** EDN is structural-derived from the SymbolTable. Free for any declared type. No tag registration. No reader/writer functions. No application ceremony.
- **Clojure side:** EDN dispatch is tag-by-tag — Clojure has no equivalent SymbolTable to walk. A small `wat-edn-clj` library closes the gap by providing default readers + print-methods for the wat namespace; consumers register only application types, and those default to passthrough.

The asymmetry is structural, not accidental. wat OWNS its type universe (SymbolTable is part of the language). Clojure does not — Clojure's type system is structural, and EDN is a wire format orthogonal to it. The bridge can't make Clojure look like wat; it can only make Clojure as ergonomic as a small library can.

## Implications for first slice

This beat establishes:

1. **A second crate is in scope.** `wat-edn-clj` (Clojure side, ~200 LOC) ships alongside `wat_edn` (Rust/wat side). Versioned together — wire-format compatibility is the contract.

2. **The acceptance bar for first slice.** Round-trip the realistic blob (`enterprise.observer/Engram` from `string-examples.md`, or `TradeSignal` from this beat) through:
   - wat write → file → Clojure read → display
   - Clojure write → file → wat read → consume
   - Byte-equivalence on each leg

3. **Application code on both sides stays bridge-agnostic.** wat side declares structs and calls `read-str` / `write-str`. Clojure side calls `wat-edn/read-stream` and uses standard Clojure idioms (`pr-str`, `print-method`, with-meta). Neither side touches tag-handler registration manually.

## Open question — heterogeneous streams

The dashboard above knew it was reading `TradeSignal`s. Some consumers won't — e.g. a generic event-log inspector reading a heterogeneous stream of `TradeSignal`, `SizeAdjust`, `Heartbeat`, etc.

The tag-as-type design handles this naturally on both sides:

- Wat reads with `read-str` (returning `Value` with the tagged-element variant exposed); the consumer pattern-matches on the tag name to dispatch.
- Clojure with `wat-edn-clj` returns the body with metadata `^{:wat/type 'enterprise.observer/TradeSignal}` (or similar); the consumer dispatches on the `:wat/type` meta or on map shape.

The wire format SAYS what each entry is. The reader's job is to return that information; the consumer's job is to act on it. No registration of "what types might appear" is required upfront on either side.

## Note on user voice

The user's framing was load-bearing for this beat:

> "the whole point of this is to provide a bridge to clojure... i need apps on both sides to interop..."

Without that emphasis, the design could have stopped at "wat reads/writes EDN." Naming "apps on both sides" forced the question of Clojure-side ergonomics, which surfaced the second crate, which surfaced the asymmetry — and naming the asymmetry honestly is the difference between a bridge that ships and a bridge that gets reverse-engineered later.

Closing user voice:

> "outstanding - let's get this in a doc - we can't lose this - this is stellar"

Banked.
