;; data_readers.clj — manifest for Clojure's standard tag readers.
;;
;; SCRATCH SKETCH — arc 003-edn-typed-wire.
;;
;; This file matters for code-as-data paths (regular `clojure.core/read`
;; from a Clojure source file). For application data files,
;; consumers use `wat-edn-clj.core/read-stream` which uses the
;; multi-arg default-fn instead — that path doesn't consult this
;; file.
;;
;; Most wat tags are handled by the default-fn in core; this manifest
;; just provides safe fallbacks for the standard reader path.

{wat.holon/Atom         wat-edn-clj.core/passthrough
 wat.holon/Symbol       wat-edn-clj.core/passthrough
 wat.holon/Bind         wat-edn-clj.core/passthrough
 wat.holon/Bundle       wat-edn-clj.core/passthrough
 wat.holon/Permute      wat-edn-clj.core/passthrough
 wat.holon/Thermometer  wat-edn-clj.core/passthrough
 wat.holon/Keyword      wat-edn-clj.core/passthrough
 wat.holon/I64          wat-edn-clj.core/passthrough
 wat.holon/F64          wat-edn-clj.core/passthrough
 wat.holon/Bool         wat-edn-clj.core/passthrough
 wat.holon/String       wat-edn-clj.core/passthrough}
