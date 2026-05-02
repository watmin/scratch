# Goal and constraints

## The ask (user's voice, verbatim)

> "ok - i want to accept edn with vectors as [1 2 3] expr, hashmaps
> as {:key1 :val1 :key2 :val2} and hashsets as #{1 2 3}"
>
> "in wat lisp there's only parens... and its strongly typed... i
> want to figure out how to write and read edn for clojure
> integration"

## What's actually in collision

Three things wanted at once:

1. **Read EDN** — accept text containing `[]`, `{}`, `#{}` (and the rest of EDN's repertoire) and parse it into wat values.
2. **Write EDN** — emit wat values as EDN text Clojure can read back.
3. **Preserve wat's discipline** — paren-only reader, strong typing, no surprise inference.

EDN is dynamically typed; wat is statically typed. EDN has three native collection literals; wat has zero literals for `:Vec<T>`, `:HashMap<K,V>`, `:HashSet<T>` (they're constructed via paren forms). Clojure programs ARE EDN data; wat programs are paren-only s-expressions. The two languages share Lisp heritage but diverge at almost every concrete point.

## Two distinct surfaces (conflated in the initial ask)

The user's framing didn't yet separate these:

- **EDN-as-data** — read/write functions take/return strings. EDN syntax exists ONLY inside string literals. The wat reader never sees `[]` `{}` `#{}` as source forms.
- **EDN-as-source-syntax** — extend the wat reader to recognize `[]` `{}` `#{}` as legal data literals in `.wat` files.

These are very different commitments. The first is a serialization library. The second is a reader extension that crosses wat's "verbose is honest" posture (what does `(:wat::core::vec ...)` ELIMINATE? Mostly text, not load-bearing pain — adding three reader forms requires real justification).

## Assistant's first-pass framing (rejected at beat 2)

The framing offered before the user's pivot:

- EDN syntax stays inside string literals (`(read-str "[1 2 3]")`)
- Dynamic surface: `:wat::edn::Value` enum mirroring `serde_json::Value` / the EDN crate's `Edn`
- Typed sugar: `read-str-as<:Vec<i64>>` for the homogeneous common case
- Caller annotates the expected type at every read site

Tradeoff named at the time: Clojure programmers can't paste literals directly into wat source (always wrap in string + read call), but the reader stays pure-parens and types stay strong. Mentioned in passing: EDN tags exist in the spec; could be used.

The user's next question — "can we not use edn's tags for types?" — picked up that throwaway and made it the load-bearing answer. (See `tag-as-type.md`.)

## What was already in the project's ground truth

Worth flagging: wat already has canonical-EDN serialization for ASTs in holon-rs (`canonical_edn_holon`). It's used for content-addressing, signed-load digests, and the AST hash that gates programs-as-holons identity. So **EDN is already a wire format in the project**, just not a general-purpose read/write surface. Whatever shape we pick for general EDN read/write should compose with that existing infrastructure — or at minimum not contradict it.
