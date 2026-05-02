(ns wat-edn-clj.core
  "wat-edn-clj — Clojure side of the wat ↔ Clojure EDN bridge.

  SCRATCH SKETCH from arc 003-edn-typed-wire (beat 9).

  The wat side emits FQDN-tagged EDN (Strategy A from beat 4):

    #wat.core/Vec<i64>            [1 2 3]
    #enterprise.observer/Engram   {...}
    #wat.core/Some<f64>           42.5

  This namespace ships:
    - default reader fn that handles all wat.* tags
    - print-method extensions for writing wat-readable EDN back
    - helper for one-line application-type registration
    - variant wrappers + predicates for sums (Some/None/Ok/Err)

  Consumers register only application types. Everything in wat.*
  is handled by the default reader."
  (:refer-clojure :exclude [read])
  (:require [clojure.edn :as edn]
            [clojure.java.io :as io]))

;; ─── Reader: default fn for the wat namespace ────────────────────

(defn- starts-with? [^String s ^String prefix]
  (.startsWith s prefix))

(defn wat-default-reader
  "Default reader for tags in the wat.* namespace.

  Strategy: collection wrapper tags (Vec<T>, HashMap<K,V>,
  HashSet<T>, HolonAST variants) carry type information that
  Clojure pattern-matchers don't usually need — the body IS
  already idiomatic Clojure data. Strip the type tag, keep
  the data.

  Sum variants (Some, None, Ok, Err) wrap to preserve identity:

    [::some 42]      — Some 42
    [::none]         — None
    [::ok 42]        — Ok 42
    [::err \"boom\"] — Err \"boom\"

  Unknown tags (anything outside wat.*) preserve as
  tagged-literal so consumers can pass them through."
  [tag body]
  (let [s (str tag)]
    (cond
      ;; Collections — body is already the right shape
      (or (starts-with? s "wat.core/Vec")
          (starts-with? s "wat.core/HashMap")
          (starts-with? s "wat.core/HashSet")
          (starts-with? s "wat.holon/")
          (starts-with? s "wat.scalar/"))
      body

      ;; Sums (Style A from arc beat 7) — :type meta lets print-method
      ;; dispatch back to wat tag form on round-trip.
      (starts-with? s "wat.core/Some") (with-meta [::some body] {:type ::variant})
      (starts-with? s "wat.core/None") (with-meta [::none]      {:type ::variant})
      (starts-with? s "wat.core/Ok")   (with-meta [::ok body]   {:type ::variant})
      (starts-with? s "wat.core/Err")  (with-meta [::err body]  {:type ::variant})

      ;; Unknown — preserve gracefully
      :else (tagged-literal tag body))))

;; ─── Application-type registry ──────────────────────────────────

(def application-types
  "Atom of registered application types. Each entry maps a tag
  symbol to a reader fn. Defaults to identity (passthrough)."
  (atom {}))

(declare tag-as)

(defn- preserving-reader
  "Default reader for a registered application type: wraps the body
  with `:type ::tagged-map` + `:wat/tag` metadata so it round-trips
  cleanly through write."
  [tag-symbol]
  (fn [body] (tag-as tag-symbol body)))

(defn register-types!
  "Register one or more application types.

  Vector form (default tag-preserving reader):
    (register-types! ['enterprise.observer/TradeSignal
                      'enterprise.config/SizeAdjust])

    Reads `#enterprise.observer/TradeSignal {...}` as a map with
    metadata `^{:type ::tagged-map :wat/tag <symbol>}` so writing
    it back emits the same tag.

  Map form (custom decoders override the default):
    (register-types! {'enterprise.observer/TradeSignal map->TradeSignal
                      'enterprise.config/SizeAdjust    map->SizeAdjust})"
  [types]
  (cond
    (sequential? types)
    (swap! application-types into
           (zipmap types (map preserving-reader types)))

    (map? types)
    (swap! application-types merge types)

    :else
    (throw (ex-info "register-types! expects a vector or map"
                    {:got types})))
  nil)

(defn passthrough
  "Single-arg reader fn for data_readers.clj manifest entries.
  Returns body unchanged — the type tag is stripped at the
  read level."
  [body]
  body)

;; ─── Read API ───────────────────────────────────────────────────

(defn read
  "Read one EDN form from the reader. Uses the wat default-fn for
  any wat.* tag and the application-types registry for app tags.

  `eof` defaults to ::eof; pass a custom sentinel if you need to
  distinguish nil-as-data from end-of-stream."
  ([rdr] (read rdr ::eof))
  ([rdr eof]
   (edn/read {:readers @application-types
              :default wat-default-reader
              :eof     eof}
             rdr)))

(defn read-str
  "Parse a single EDN form from a string."
  [s]
  (edn/read-string {:readers @application-types
                    :default wat-default-reader}
                   s))

(defn read-stream
  "Read all top-level EDN forms from a reader. Returns a vector."
  [rdr]
  (let [pbr (if (instance? java.io.PushbackReader rdr)
              rdr
              (java.io.PushbackReader. rdr))]
    (loop [out []]
      (let [v (read pbr ::done)]
        (if (= v ::done)
          out
          (recur (conj out v)))))))

(defn read-file
  "Read all EDN forms from a file path."
  [path]
  (with-open [r (io/reader path)]
    (read-stream r)))

;; ─── Write API ──────────────────────────────────────────────────

;; A tag-as'd map carries metadata {:wat/tag <symbol>} plus a
;; type marker so print-method dispatches here.

(defmethod print-method ::tagged-map
  [v ^java.io.Writer w]
  (when-let [tag (-> v meta :wat/tag)]
    (.write w "#")
    (.write w (str tag))
    (.write w " "))
  (let [bare (with-meta (into {} v) nil)]
    (print-method bare w)))

(defn tag-as
  "Annotate a Clojure map so it round-trips as a wat-side struct.

  Usage:
    (tag-as 'enterprise.config/SizeAdjust
            {:asset :BTC :factor 1.5 :until [::some t]})"
  [tag-symbol m]
  (with-meta m
             {:type    ::tagged-map
              :wat/tag tag-symbol}))

(defn write-str
  "Serialize a value to an EDN string."
  [v]
  (binding [*print-dup* false]
    (pr-str v)))

(defn print-line!
  "Write a value to a writer as one EDN line."
  [^java.io.Writer w v]
  (.write w (write-str v))
  (.write w "\n")
  (.flush w))

(defn append-file!
  "Append a value to an EDN file as one line."
  [path v]
  (with-open [w (io/writer path :append true)]
    (print-line! w v)))

;; ─── Variant helpers ────────────────────────────────────────────
;;
;; Variants are represented as [::kind body] tuples with `:type`
;; metadata so print-method can dispatch them back to wat tag form.
;; Without the metadata, pr-str would emit
;;   [:wat-edn-clj.core/some 42]
;; which is not wat-readable. With it, print-method writes
;;   #wat.core/Some 42

(defn some-of  [x] (with-meta [::some x] {:type ::variant}))
(defn none-of  []  (with-meta [::none]   {:type ::variant}))
(defn ok-of    [x] (with-meta [::ok x]   {:type ::variant}))
(defn err-of   [e] (with-meta [::err e]  {:type ::variant}))

(defn some-variant? [x] (and (vector? x) (= ::some (first x))))
(defn none-variant? [x] (and (vector? x) (= ::none (first x))))
(defn ok-variant?   [x] (and (vector? x) (= ::ok   (first x))))
(defn err-variant?  [x] (and (vector? x) (= ::err  (first x))))

(defmethod print-method ::variant
  [v ^java.io.Writer w]
  (let [kind (first v)]
    (case kind
      ::some (do (.write w "#wat.core/Some ")
                 (print-method (second v) w))
      ::none (.write w "#wat.core/None nil")
      ::ok   (do (.write w "#wat.core/Ok ")
                 (print-method (second v) w))
      ::err  (do (.write w "#wat.core/Err ")
                 (print-method (second v) w)))))

(defn unwrap-some [x] (when (some-variant? x) (second x)))
(defn unwrap-ok   [x] (when (ok-variant?   x) (second x)))
(defn unwrap-err  [x] (when (err-variant?  x) (second x)))

(defn fold-option
  "Pattern-match an Option-style variant.
    (fold-option v
      :some (fn [x] ...)
      :none (fn []  ...))"
  [v & {:keys [some none]}]
  (cond
    (some-variant? v) (some (second v))
    (none-variant? v) (none)
    :else (throw (ex-info "Not an Option variant" {:got v}))))

(defn fold-result
  "Pattern-match a Result-style variant.
    (fold-result v
      :ok  (fn [x] ...)
      :err (fn [e] ...))"
  [v & {:keys [ok err]}]
  (cond
    (ok-variant?  v) (ok  (second v))
    (err-variant? v) (err (second v))
    :else (throw (ex-info "Not a Result variant" {:got v}))))
