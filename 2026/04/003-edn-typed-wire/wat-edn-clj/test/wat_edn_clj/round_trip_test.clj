(ns wat-edn-clj.round-trip-test
  "Smoke test: feed sample wat-emitted EDN through the library, see
  if the read path produces sensible Clojure data structures."
  (:require [wat-edn-clj.core :as wat-edn]))

(defn header [s]
  (println)
  (println (str "─── " s " " (apply str (repeat (- 60 (count s)) "─")))))

(defn case! [label edn-str]
  (println)
  (println (str "  Input:  " edn-str))
  (try
    (let [parsed (wat-edn/read-str edn-str)]
      (println (str "  Parsed: " (pr-str parsed)))
      (println (str "  Class:  " (class parsed))))
    (catch Throwable t
      (println (str "  ERROR:  " (.getMessage t)))
      (println (str "          (" (class t) ")")))))

;; ─── Primitives & collections ─────────────────────────────────

(header "Primitives & collections")

(case! "int"      "42")
(case! "float"    "3.14")
(case! "string"   "\"hello\"")
(case! "keyword"  ":rsi")
(case! "kw-path"  ":enterprise.observer.foo")

(case! "Vec<i64>"
       "#wat.core/Vec<i64> [1 2 3]")
(case! "HashSet<i64>"
       "#wat.core/HashSet<i64> #{1 2 3}")
(case! "HashMap<String_f64>"
       "#wat.core/HashMap<String_f64> {\"win-rate\" 0.594 \"sharpe\" 1.41}")

;; ─── Sums ─────────────────────────────────────────────────────

(header "Sums (Style A)")

(case! "Some<f64>"  "#wat.core/Some<f64> 64500.0")
(case! "None<f64>"  "#wat.core/None<f64> nil")
(case! "Ok<i64_String>"  "#wat.core/Ok<i64_String> 42")
(case! "Err<i64_String>" "#wat.core/Err<i64_String> \"boom\"")

;; Test variant predicates
(println)
(let [some-val (wat-edn/read-str "#wat.core/Some<f64> 64500.0")
      none-val (wat-edn/read-str "#wat.core/None<f64> nil")
      ok-val   (wat-edn/read-str "#wat.core/Ok<i64_String> 42")]
  (println "  some-variant? on Some:" (wat-edn/some-variant? some-val))
  (println "  unwrap-some on Some:  " (wat-edn/unwrap-some some-val))
  (println "  none-variant? on None:" (wat-edn/none-variant? none-val))
  (println "  ok-variant? on Ok:    " (wat-edn/ok-variant? ok-val))
  (println "  unwrap-ok on Ok:      " (wat-edn/unwrap-ok ok-val)))

;; ─── HolonAST variants ─────────────────────────────────────────

(header "HolonAST variants")

(case! "Atom"
       "#wat.holon/Atom :rsi-rising")
(case! "Bind"
       "#wat.holon/Bind [:role :filler]")
(case! "nested Atom+I64"
       "#wat.holon/Atom #wat.holon/I64 42")

;; ─── Built-in EDN tags ─────────────────────────────────────────

(header "Built-in EDN tags")

(case! "#inst" "#inst \"2026-04-26T14:30:00Z\"")
(case! "#uuid" "#uuid \"550e8400-e29b-41d4-a716-446655440000\"")

;; ─── Application types (registered) ────────────────────────────

(header "Application types")

(wat-edn/register-types! ['enterprise.observer/TradeSignal
                          'enterprise.config/SizeAdjust])

(def realistic-blob
  (str "#enterprise.observer/TradeSignal\n"
       "{:asset       :BTC\n"
       " :side        :Buy\n"
       " :size        0.025\n"
       " :confidence  0.73\n"
       " :reasoning   #wat.core/Vec<wat.holon.HolonAST>\n"
       "                [#wat.holon/Atom :rsi-rising\n"
       "                 #wat.holon/Atom :flow-positive]\n"
       " :proposed-at #inst \"2026-04-26T14:30:00Z\"}"))

(case! "TradeSignal" realistic-blob)

(println)
(println "  Field access on the parsed signal:")
(let [sig (wat-edn/read-str realistic-blob)]
  (println (format "    asset:       %s" (:asset sig)))
  (println (format "    side:        %s" (:side sig)))
  (println (format "    size:        %s" (:size sig)))
  (println (format "    confidence:  %s" (:confidence sig)))
  (println (format "    proposed-at: %s" (:proposed-at sig)))
  (println (format "    reasoning:   %d elements" (count (:reasoning sig)))))

;; ─── Write side ─────────────────────────────────────────────────

(header "Writing wat-readable EDN (struct path)")

(let [adjust (wat-edn/tag-as 'enterprise.config/SizeAdjust
                             {:asset  :BTC
                              :factor 1.5
                              :until  (wat-edn/some-of
                                       #inst "2026-04-26T16:00:00Z")})]
  (println)
  (println "  Value (tagged-map):")
  (println (str "    " (pr-str adjust)))
  (println)
  (println "  EDN form (via wat-edn/write-str):")
  (println (str "    " (wat-edn/write-str adjust))))

;; ─── Round-trip ─────────────────────────────────────────────────

(header "Round-trip a struct")

(wat-edn/register-types! ['my.app/Thing])

(let [original {:value 42 :name "test"}
      tagged   (wat-edn/tag-as 'my.app/Thing original)
      written  (wat-edn/write-str tagged)
      back     (wat-edn/read-str written)]
  (println)
  (println (str "  Original:  " (pr-str original)))
  (println (str "  Tagged:    " (pr-str tagged)))
  (println (str "  Written:   " written))
  (println (str "  Read back: " (pr-str back)))
  (println (str "  Match:     " (= original back))))

(header "Round-trip a struct with a sum field")

(let [adjust  (wat-edn/tag-as
                'enterprise.config/SizeAdjust
                {:asset  :BTC
                 :factor 1.5
                 :until  (wat-edn/some-of
                          #inst "2026-04-26T16:00:00Z")})
      written (wat-edn/write-str adjust)
      back    (wat-edn/read-str written)
      back2   (wat-edn/write-str back)]
  (println)
  (println (str "  Written:    " written))
  (println (str "  Read back:  " (pr-str back)))
  (println (str "  Re-written: " back2))
  (println (str "  Match:      " (= written back2))))

(println)
(println "─── Done ───")
