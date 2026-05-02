;;
;; consumer.wat — example wat program using wat_edn for Clojure interop.
;;
;; SCRATCH SKETCH from arc 003-edn-typed-wire (beat 9, consumer-bridge.md).
;; Demonstrates the bidirectional bridge: wat enterprise emits trade
;; signals, reads config adjustments back from a Clojure dashboard.
;;
;; Wire-up (in the Rust binary that runs this):
;;
;;   wat::main! { deps: [shim, wat_edn] }
;;

;; ─── Shared protocol types ──────────────────────────────────────

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

;; ─── Emit a signal (wat -> Clojure direction) ──────────────────

(:wat::core::define (:enterprise::observer::emit-signal
                     (sig :enterprise::observer::TradeSignal)
                     (out :wat::io::IOWriter)
                     -> :Result<:(), :wat::edn::EdnError>)
  (:wat::core::let* (((edn :String) (:wat::edn::write-str sig)))
    (:wat::io::write-line! out edn)))

;; ─── Read a config update (Clojure -> wat direction) ───────────

(:wat::core::define (:enterprise::config::read-adjust
                     (in :wat::io::IOReader)
                     -> :Result<:enterprise::config::SizeAdjust,
                                :wat::edn::EdnError>)
  (:wat::core::let* (((line :String)
                      (:wat::core::try (:wat::io::read-line! in))))
    (:wat::edn::read-str-as<:enterprise::config::SizeAdjust> line)))

;; ─── Apply an adjustment (consumer-side action) ─────────────────

(:wat::core::define (:enterprise::apply-adjust!
                     (adj :enterprise::config::SizeAdjust)
                     -> :())
  ;; SCRATCH: real version would update a treasury parameter.
  ;; This stub just demonstrates the destructure.
  (:wat::core::match (:enterprise::config::SizeAdjust/until adj) -> :()
    ((Some t) ())   ; sized increase until t
    (:None    ()))) ; permanent adjustment

;; ─── Heartbeat: emit signal, check for config update ────────────

(:wat::core::define (:enterprise::on-tick
                     (sig         :enterprise::observer::TradeSignal)
                     (signals-out :wat::io::IOWriter)
                     (config-in   :wat::io::IOReader)
                     -> :())
  (:wat::core::try (:enterprise::observer::emit-signal sig signals-out))
  (:wat::core::match (:enterprise::config::read-adjust config-in) -> :()
    ((Ok adj) (:enterprise::apply-adjust! adj))
    ((Err _)  ())))   ; no adjustment available, carry on

;; ─── Main entry point ──────────────────────────────────────────

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  ;; SCRATCH: full lifecycle would open signals.edn and
  ;; config-updates.edn as IOWriter/IOReader, route the heartbeat
  ;; through them, and loop on candle ticks. This stub shows the
  ;; wiring for one tick.
  (:wat::core::let* (((sig :enterprise::observer::TradeSignal)
                      (:enterprise::observer::TradeSignal/new
                        :BTC :Buy 0.025 0.73
                        (:wat::core::vec)
                        (:wat::time::now))))
    (:enterprise::on-tick sig stdout stdin)))
