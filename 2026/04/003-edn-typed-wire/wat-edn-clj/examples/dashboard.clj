(ns dashboard
  "Example wat ↔ Clojure dashboard.

  SCRATCH SKETCH — arc 003-edn-typed-wire (beat 9).

  Reads trade signals from a wat-emitted EDN file, displays them,
  and writes config adjustments back as EDN that the wat side
  consumes.

  Run from this directory:
    clj -M:run-dashboard signals.edn config-updates.edn"
  (:require [wat-edn-clj.core :as wat-edn]
            [clojure.java.io :as io]))

;; Register application types. Defaults to passthrough (the body
;; IS the Clojure data we want). Pass a map instead if you want
;; defrecord wrapping or validation.
(wat-edn/register-types!
  ['enterprise.observer/TradeSignal
   'enterprise.config/SizeAdjust])

(defn show-signal [sig]
  (println
    (format "[%s] %s %s size=%.4f conf=%.2f reasoning=%d-elem"
            (:proposed-at sig)
            (name (:asset sig))
            (name (:side sig))
            (:size sig)
            (:confidence sig)
            (count (:reasoning sig)))))

(defn read-signals [path]
  (wat-edn/read-file path))

(defn emit-adjust [path asset factor until]
  (wat-edn/append-file!
    path
    (wat-edn/tag-as 'enterprise.config/SizeAdjust
                    {:asset  asset
                     :factor factor
                     :until  (if until
                               (wat-edn/some-of until)
                               (wat-edn/none-of))})))

(defn -main [& args]
  (let [signals-path (or (first args)  "signals.edn")
        config-path  (or (second args) "config-updates.edn")]
    (println "Reading signals from" signals-path)
    (doseq [sig (read-signals signals-path)]
      (show-signal sig))
    (println)
    (println "Emitting size-up for BTC...")
    (emit-adjust config-path
                 :BTC
                 1.5
                 #inst "2026-04-26T16:00:00Z")
    (println "Done. Wrote to" config-path)))
