# Command set — FQDN-explicit; tab completion as the navigator

User's framing 2026-04-29:

> "i don't know if cd even has meaning in wat.. everything is a
> fqdn..."

Right. Pry's `cd` doesn't translate. Wat has no ambient self, no
current namespace, no implicit scope to be "inside of." Every
reference is fully qualified. The functional replacement isn't
navigation; it's **completion**.

## What pry has that maps cleanly

| Pry | Wat-pry | Notes |
|---|---|---|
| Read-eval-print | `:wat::pry::serve` (the loop) | Tail-recursive wat function; one EDN value in, one EDN value out. |
| Line editing, history | rustyline integration in the cli's parent | Frontend concern; substrate untouched. |
| Multi-line input | Lexer's `UnclosedBracket` signal triggers continuation prompt | Existing lexer state; pry uses it. |
| Tab completion | `:wat::pry::completions :String -> :Vec<:String>` | Substrate primitive walks SymbolTable for prefix matches. |
| `ls` | `:ls` pry command | Lists visible names; optional `:ls <prefix>` filters. |
| `show-source FN` | `:show :symbol` | Prints function source / struct definition / enum variants. |
| `whereami` | `:where` | Shows current break point file:line:col + surrounding source via Span (arc 016). |
| `binding.pry` | `(:wat::pry::break)` | Substrate primitive; Environment + CALL_STACK capture. See `break-primitive.md`. |
| `next` / `step` / `finish` | `:next` / `:step` / `:finish` | Composed on `:wat::eval-step!` (arc 068). Slice 4. |
| `wtf?` | `:wtf` or `:last-error` | Prints last error's frames + message via arc 016 / arc 060. |
| Pretty-printing | `:wat::edn::write` (compact today; pretty later) | Substrate; pretty mode is a future addition. |
| `help FN` / `show-doc FN` | `:show :symbol` (signature) | Function schemes carry types; doc strings are a future addition (`/// foo` syntax). |

## What doesn't map — explicit non-translations

| Pry | Wat-pry decision |
|---|---|
| `cd Trader` | **Not implemented.** No ambient context to navigate. Use FQDN at every call site. |
| `edit METHOD` (modify live) | **Not implemented.** Hologram forbids in-place redefinition. Use `:reload` to rebuild from disk. |
| `pry-byebug` plugin | The base substrate covers stepping (slice 4); no plugin layer needed. |
| Method introspection (`Object.public_methods`) | `:ls` filtered by namespace covers it. The only "object" wat has is the SymbolTable. |

## The core commands (slice 1 minimum)

These are the commands the pry loop recognizes outside of
break-mode (the bare `wat --pry` session):

```
:ls [prefix]       — list visible names; optional FQDN prefix filter
:show :symbol      — show source / type signature / struct decl
:reload            — kill the pry backend, re-fork from current source
:exit              — shut down cleanly
:help              — list available commands

<any expression>   — evaluated against the frozen world; result printed
```

Plus tab completion at the rustyline layer, which calls
`:wat::pry::completions <prefix>` to populate suggestions.

## Break-mode commands (slice 2 additional)

Inside `(:wat::pry::break)`, the loop also recognizes:

```
:continue          — resume execution; pry::break returns :()
:where             — show current location (Span + surrounding source)
:env               — list captured locals + their values
:frames            — print the CALL_STACK
:up                — walk one frame up; eval context switches
:down              — walk one frame down
:wtf               — last-error info (if break was triggered by panic)
```

Plus all the bare-mode commands work — `:ls`, `:show`,
`<expression>`, etc. The eval scope is the captured Environment;
otherwise behavior is identical.

## Stepping commands (slice 4 additional)

```
:next              — eval-step one form; print the next-form
:step              — eval-step into a sub-form
:finish            — eval-step to the end of the current frame
```

Built on `:wat::eval-step!` (arc 068). The pry loop holds the
current evaluation state and steps through manually.

## Tab completion — the FQDN navigator

Without `cd`, the way users find what's available is type a prefix
and tab:

```
wat-pry> :trading::<TAB>
:trading::types::    :trading::vocab::    :trading::sim::
:trading::observer:: :trading::risk::

wat-pry> :trading::types::<TAB>
:trading::types::Candle      :trading::types::Direction
:trading::types::Outcome     :trading::types::PaperEntry
...

wat-pry> :trading::types::Candle/<TAB>
:trading::types::Candle/new        :trading::types::Candle/open
:trading::types::Candle/close      :trading::types::Candle/high
...
```

The substrate primitive `:wat::pry::completions` walks the
SymbolTable for entries whose name starts with the given prefix,
returns the matching paths.

This makes discovery FQDN-native. There's no state to maintain
("am I inside :trading::types right now?"); every completion is
absolute. The user types a prefix; the substrate suggests paths.
The user picks one. No `cd`; no `pwd`; just paths.

## `:show` examples — what the introspection trio prints

**`:show` on a struct:**

```
wat-pry> :show :trading::types::Candle
struct :trading::types::Candle (defined at trade.wat:14:1)
  open    :f64
  high    :f64
  low     :f64
  close   :f64
  volume  :f64

  auto-generated functions:
    :trading::types::Candle/new (open :f64) (high :f64) (low :f64) (close :f64) (volume :f64) -> :Candle
    :trading::types::Candle/open  (c :Candle) -> :f64
    :trading::types::Candle/high  (c :Candle) -> :f64
    :trading::types::Candle/low   (c :Candle) -> :f64
    :trading::types::Candle/close (c :Candle) -> :f64
    :trading::types::Candle/volume (c :Candle) -> :f64
```

**`:show` on a function:**

```
wat-pry> :show :trading::compute-decision
fn :trading::compute-decision (candle :Candle) -> :Action  (defined at trade.wat:42:1)

  (let* (((rsi :f64)        (:trading::rsi candle))
         ((vol :f64)        (:trading::vol candle))
         ((regime :Regime)  (:trading::classify rsi vol)))
    (:wat::pry::break)
    (:trading::action regime rsi vol))
```

**`:show` on an enum:**

```
wat-pry> :show :trading::types::Direction
enum :trading::types::Direction (defined at types.wat:8:1)
  variants:
    :Direction::Up
    :Direction::Down
    :Direction::Sideways
```

The source is reconstructed from the frozen AST cached in the
symbol table (every define stores its body for compilation
purposes; pry pretty-prints it).

## `:env` example — what break-mode shows

```
wat-pry (broken @ trade.wat:42:7) compute-decision> :env
captured Environment at compute-decision:
  candle  :  :trading::types::Candle  =  #trading.types.Candle {:open 50000.0 :high 50500.0 ...}
  rsi     :  :f64                     =  0.6234
  vol     :  :f64                     =  0.0083
  regime  :  :trading::types::Regime  =  :Regime::Trending

  parent scope (:user::main):
    stdin   :  :wat::io::IOReader  =  <IOReader>
    stdout  :  :wat::io::IOWriter  =  <IOWriter>
    stderr  :  :wat::io::IOWriter  =  <IOWriter>
```

Each binding shows name, type, value (rendered via
`:wat::edn::write`). Parent scopes appear under indent. Every
let* binding visible in the captured Environment is listed; pry
walks the parent-pointer chain and prints each scope.

## Aliases / domain-specific commands

The pry loop's command dispatcher is wat code in
`wat/std/pry.wat`. A battery can ship its own
`wat/pry-commands/*.wat` that adds domain shorthands:

```
;; in wat-telemetry-sqlite/wat/pry-commands/sql.wat:
(:wat::core::define
  (:wat::pry/sql (cmd-args :String) (env :Environment) -> :())
  ;; takes the rest of the line, treats it as SQL, runs against current ReadHandle
  ...)
```

When loaded, `:sql SELECT * FROM log LIMIT 5` at the prompt would
dispatch to this function. Plugin extensibility falls out of "the
pry loop is a wat program" — no Rust additions needed.

## Why this is enough

Pry's whole pitch is: stop the program, ask anything you want,
resume. The command set above gives that. The introspection
covers types, functions, enums, locals, frames. The expression
evaluation covers everything else (any wat expression at the
prompt evaluates against the captured scope).

There's no `cd` because there's no need. There's no `edit`
because the hologram forbids it (with `:reload` covering the
edit-test cycle). There's no plugin system because the loop is a
wat program — extension is wat code.

Slice 1 ships the bare-mode commands. Slice 2 adds break-mode.
Slice 4 adds stepping. Three slices to feature-complete pry. The
substrate doesn't need any new mechanisms — just exposing what
already exists through targeted primitives.
