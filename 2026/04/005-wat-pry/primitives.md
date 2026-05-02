# Substrate primitives — the `:wat::pry::*` namespace

Eight primitives at the substrate. Plus one wat-level macro and
one wat-level loop. All registered conditionally — present only
when the cli passes `--pry` (see `gating.md`).

## Rust-side primitives (in `crates/wat-pry/`)

These are the `#[wat_dispatch]` shims the pry battery contributes.
Each lives in `crates/wat-pry/src/lib.rs` (or split across modules
as the surface grows).

### `:wat::pry::main`

```
:wat::pry::main (stdin  :wat::io::IOReader)
                (stdout :wat::io::IOWriter)
                (stderr :wat::io::IOWriter)
                -> :()
```

The cli's pry-mode entry-point. Implementation: prints a banner,
calls `:wat::pry::serve` in a loop, handles top-level command
dispatch (e.g., `:reload` → exit non-zero so the cli's parent
re-forks).

Could be a wat-level define (in `wat/std/pry.wat`) instead of a
Rust shim. The choice is whether we want the entry-point in
substrate Rust or substrate wat. Wat-level is more honest (pry
is "just a wat program") and follows the pattern of every other
shipped service. Lean: ship as wat. Rust shim only if the
banner / signal-handling needs Rust-level capabilities the wat
substrate doesn't expose at the moment.

### `:wat::pry::serve`

```
:wat::pry::serve (in  :wat::io::IOReader)
                 (out :wat::io::IOWriter)
                 (err :wat::io::IOWriter)
                 -> :()
```

The read-eval-print loop itself. **Implementation lives in
`wat/std/pry.wat`**, not Rust — it's a tail-recursive wat
function that:

1. Writes prompt to `out`.
2. Reads one line from `in`.
3. Recognizes pry commands (`:ls`, `:show`, `:exit`, etc.) and
   dispatches.
4. For non-command input: parses via `:wat::edn::read`, evaluates
   via `:wat::eval-edn!`, writes result via `:wat::edn::write`.
5. Recurses (tail call; arc 003 TCO keeps stack constant).

Bare-mode: invoked by `:wat::pry::main` against the program's
stdio.

Break-mode: invoked by `:wat::pry::break-with-stdio` against the
program's stdio with a captured Environment as eval context. Same
function, different scope.

### `:wat::pry::break-with-stdio`

```
:wat::pry::break-with-stdio
  (in  :wat::io::IOReader)
  (out :wat::io::IOWriter)
  (err :wat::io::IOWriter)
  -> :()
```

The substrate primitive. Captures current Environment + CALL_STACK,
calls `:wat::pry::serve` with the captured scope as eval context,
returns `:()` when user types `:continue`. See `break-primitive.md`
for details.

This is the only primitive that requires substrate-level Rust —
because Environment capture has to happen INSIDE `eval_form`,
which knows the current scope. Everything else can live in wat.

### `:wat::pry::completions`

```
:wat::pry::completions (prefix :String) -> :Vec<:String>
```

Walks the SymbolTable for entries whose path starts with `prefix`.
Returns matching FQDN paths sorted alphabetically.

Implementation:

```rust
fn eval_pry_completions(args: &[Value], world: &FrozenWorld) -> Result<Value, RuntimeError> {
    let prefix = expect_string(&args[0])?;
    let matches: Vec<Value> = world
        .symbols
        .iter()
        .filter(|(path, _)| path.starts_with(prefix))
        .map(|(path, _)| Value::String(path.clone()))
        .collect();
    Ok(Value::Vec(Arc::new(matches)))
}
```

Called by the rustyline frontend on every TAB. The frontend
displays the suggestions; the user picks one.

### `:wat::pry::ls`

```
:wat::pry::ls (prefix :Option<String>) -> :Vec<:Symbol>
```

Same walk as `completions` but returns `:Symbol` values (which
print as keyword-paths) for the user to read directly. Used by
the `:ls` pry command. Optional prefix narrows results.

### `:wat::pry::show`

```
:wat::pry::show (sym :Symbol) -> :Option<:String>
```

Reconstructs the source for the named symbol from the frozen
SymbolTable. Returns:

- For functions: function signature + body, pretty-printed.
- For structs: `struct NAME (FIELD :TYPE) ...` with auto-generated
  accessors listed.
- For enums: `enum NAME` with each variant.
- For typealiases: the aliased type expression.
- `:None` if the symbol doesn't exist.

The frozen world already caches each define's parsed AST; show
just walks it and pretty-prints.

### `:wat::pry::where`

```
:wat::pry::where () -> :Option<:wat::kernel::Location>
```

Returns the current break point's Span (file/line/col). `:None`
if not in a break. Used by the `:where` command + the prompt
header in break-mode.

### `:wat::pry::frames`

```
:wat::pry::frames () -> :Vec<:wat::kernel::Frame>
```

Returns the CALL_STACK snapshot captured at break-time. Each
frame has its location + (post-extension; see below) its
Environment. Used by `:up` / `:down` / `:frames`.

**Substrate addition required:** today's `FrameInfo` carries
`Span` but not `Environment`. To support `:up` / `:down`, frames
must also record their lexical scope. The change: `FrameInfo`
gains `env: Environment`; `FrameGuard::new` clones the current
env at frame-entry. Cheap (Arc clone). No semantic change to
existing failure-trace output.

### `:wat::pry::last-error`

```
:wat::pry::last-error () -> :Option<:wat::kernel::Failure>
```

Returns the last panic/error captured by the cli's panic hook
(arc 016). `:None` if no error has occurred this session. Used
by `:wtf`.

The cli's parent-process panic_hook already records `Failure`
values. Pry just exposes a reader. Substrate addition: a
`OnceLock<Option<Failure>>` populated by the panic hook,
read-only from this primitive.

## Wat-side definitions (in `wat/std/pry.wat`)

The pry battery's `wat_sources()` returns this file. Loaded
conditionally with the rest of the battery.

### The `break` macro

```scheme
(:wat::core::defmacro
  (:wat::pry::break) -> :wat::WatAST
  `(:wat::pry::break-with-stdio stdin stdout stderr))
```

Expansion grabs the lexically-scoped stdio names. Works in
`:user::main` and any function threading them as args. Verbose
form for other contexts.

### The `serve` loop

Tail-recursive wat function that:

1. Prompts, reads a line.
2. Trims whitespace; if empty, recurses.
3. Tries command dispatch via prefix match on `:`.
4. Falls through to expression eval.
5. Recurses.

Pseudocode:

```scheme
(:wat::core::define
  (:wat::pry::serve
    (in  :wat::io::IOReader)
    (out :wat::io::IOWriter)
    (err :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((_ :()) (:wat::io::IOWriter/print out "wat-pry> "))
     ((line-opt :Option<String>) (:wat::io::IOReader/read-line in)))
    (:wat::core::match line-opt -> :()
      (:None ())                                    ;; EOF — exit cleanly
      ((Some line)
        (:wat::core::cond -> :()
          ((:wat::pry::is-command? line) (:wat::pry::dispatch-command line in out err))
          (else                          (:wat::pry::eval-and-print line out err)))
        (:wat::pry::serve in out err)))))           ;; tail call
```

Each command handler (`:wat::pry::dispatch-command`,
`:wat::pry::eval-and-print`, `:wat::pry::is-command?`) is its own
wat define, also in `wat/std/pry.wat`.

### Command dispatchers

Wat-level defines for each pry command:

```scheme
(:wat::core::define (:wat::pry/cmd-ls   (args :String) (out :IOWriter) -> :()) ...)
(:wat::core::define (:wat::pry/cmd-show (args :String) (out :IOWriter) -> :()) ...)
(:wat::core::define (:wat::pry/cmd-where (args :String) (out :IOWriter) -> :()) ...)
;; etc.
```

The dispatcher uses pattern matching on the leading keyword:

```scheme
(:wat::core::cond
  ((:wat::core::string::starts-with? line ":ls")    (:wat::pry/cmd-ls    rest out))
  ((:wat::core::string::starts-with? line ":show")  (:wat::pry/cmd-show  rest out))
  ((:wat::core::string::starts-with? line ":exit")  (:wat::pry/cmd-exit))
  (else (:wat::io::IOWriter/println err
          (:wat::core::format "unknown command: {}" line))))
```

## What's NOT in the substrate

- **Pretty-printing.** `:wat::edn::write` produces compact EDN
  today. A `:wat::edn::pp` primitive (or a `:pretty` flag on
  write) for indented output is a future addition.
- **Doc strings.** No `///` syntax that survives parse to
  symbol metadata. Could be added as a separate arc.
- **Plugin manifest.** Batteries that want to add pry commands
  drop `.wat` files into a known location and the loader picks
  them up. Convention only; no manifest format.
- **Color / terminal control.** ANSI codes for syntax
  highlighting / coloring break headers. Frontend (rustyline)
  concern, not substrate.
- **History storage.** `~/.wat_pry_history` file. Frontend.

## What the cli needs

The cli's role is small:

1. Parse `--pry` from argv.
2. If set, prepend the pry battery to the battery list.
3. If pry mode, look up `:wat::pry::main` instead of
   `:user::main` after freeze.
4. Hook rustyline as the parent's terminal reader; route lines
   through the child's stdin pipe.

Items 1-3 are ~20 lines in wat-cli's `run`. Item 4 is the
rustyline integration; ~150 lines including history persistence
and tab-completion plumbing.

The substrate stays library-shape; the cli stays orchestration.

## Total surface estimate

- **Substrate Rust:** ~600 lines across `crates/wat-pry/src/`
  (8 primitives, the panic-hook reader, the FrameInfo extension).
- **Substrate wat:** ~250 lines in `wat/std/pry.wat` (loop,
  command dispatchers, the break macro).
- **wat-cli additions:** ~170 lines (argv handling + rustyline
  integration).
- **Tests:** ~400 lines (round-trip integration tests for each
  primitive + the worked break example).

Roughly 1,400 lines for a pry surface with `binding.pry`-style
mid-program inspection. Compares favorably to Ruby's pry
(~25,000 lines). The size delta is the substrate's payoff —
because the substrate already had Environment, CALL_STACK,
EDN+newline, polymorphic eval, and frozen-world semantics, pry
is mostly a thin loop over what was already shipped.
