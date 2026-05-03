# wat-help — DESIGN

Foundation-tier crate. Runtime reflection surface for every
interactive consumer.

---

## The four questions are the design compass

- **Obvious?** A user typing `(:wat::help :sym)` knows exactly
  what they're asking; matches every other language's idiom.
- **Simple?** One function; one input (symbol); one output
  (formatted EDN string). Crate scope: tiny.
- **Honest?** Output IS the form-as-stored. No editorial
  synthesis; no derived prose. The substrate's reflection
  primitives provide the data; wat-fmt's rules format it; what
  the user sees is what the substrate has.
- **Good UX?** REPL users land naturally; pause sessions get
  instant help; agents-via-MCP get typed reflection.

## Architecture

```
                    Caller (REPL / pause / MCP / agent)
                         │
                         │ (:wat::help :sym)
                         ▼
┌─────────────────────────────────────────────────────────┐
│ wat-help (THIS ARC)                                     │
│                                                         │
│   1. Look up :sym in the substrate's symbol table       │
│   2. Pull the definition + metadata via reflection      │
│      primitives (substrate-provided; arc 109 work)      │
│   3. Compose the form-as-EDN representation             │
│   4. Format via wat-fmt's rules                         │
│   5. Return :String                                     │
└────────────────────────┬────────────────────────────────┘
                         │ depends on
            ┌────────────┴────────────┐
            ▼                         ▼
     wat-fmt (arc 003)        wat-rs reflection
        formatter             primitives
                              (symbol table; AST
                               serialization; source
                               location; arc 109)
```

The pipeline is short: **lookup → fetch metadata → compose →
format → return**. Each step uses existing substrate
capabilities; wat-help's value is the canonical composition.

## The form

```scheme
(:wat::help :sym)
;; => :Result<:String, :HelpError>

;; Convenience variants (composed on top of :wat::help):
(:wat::help/source :sym)        ;; just the body, no signature/doc
(:wat::help/type :sym)          ;; just the type signature
(:wat::help/doc :sym)           ;; just the docstring
(:wat::help/list :namespace)    ;; symbols in a namespace
```

The primary form is `(:wat::help :sym)` returning the canonical
formatted EDN. Convenience variants are typed wrappers that
project subsets — useful for REPL commands like `:type` or
`:source` that want focused output.

## Per-symbol-kind handling

Different kinds of symbols return different shapes:

### Functions

```scheme
(:wat::help :my-app::transfer-funds)
;; Returns the formatted definition:
(:wat::core::define
  (:my-app::transfer-funds (from :AccountId)
                           (to   :AccountId)
                           (amount :i64)
                           (memo   :string)
                           -> :Result<:TransferReceipt, :TransferError>)
  ;; Transfer `amount` from one account to another.
  ;; (docstring extracted from definition; rendered as
  ;;  ;; comments at the top of the body)
  (:wat::core::let* (...)))
```

### Types (struct / enum / newtype / typealias)

```scheme
(:wat::help :my-app::User)
;; Returns the type definition:
(:wat::core::struct :my-app::User
  ((id    :uuid)
   (email :Email)
   (name  :string :min-length 1 :max-length 100))
  ;; A user account.
  )
```

### Constants / values

```scheme
(:wat::help :my-app::DEFAULT-TIMEOUT)
;; Returns the value definition:
(:wat::core::define :my-app::DEFAULT-TIMEOUT
  ;; Default timeout for outbound HTTP calls.
  (:wat::time::Duration/seconds 30))
```

### Core primitives (built-in functions)

```scheme
(:wat::help :wat::core::HashMap)
;; Returns the substrate-documented form:
;; (since core primitives don't have a wat definition, the
;;  reflection layer returns a synthesized form documenting
;;  the type's substrate-provided contract)
(:wat::core::primitive :wat::core::HashMap
  :kind :type-constructor
  :params [:K :V]
  :doc "A keyed collection mapping K to V."
  :verbs [:assoc :dissoc :get :contains? :keys :values :empty? :length])
```

(The synthesized form for core primitives is a special case
where the wat-vm has no source AST. The substrate provides
a stable description; wat-help formats it consistently.)

### Macros

```scheme
(:wat::help :wat::core::let*)
;; Returns the macro definition (form template) + docstring:
(:wat::core::defmacro :wat::core::let* ...)
```

## Substrate reflection primitives required (arc 109 supplies)

For wat-help to work, the substrate must expose:

| Primitive | Purpose |
|---|---|
| `:wat::reflect::lookup :sym` | Look up symbol in frozen table; returns a `:Result<:SymbolEntry, :NotFound>` |
| `:wat::reflect::ast :sym` | Get the symbol's full AST (HolonAST) |
| `:wat::reflect::kind :sym` | Get the symbol kind (:Function, :Type, :Macro, :Constant, :Primitive) |
| `:wat::reflect::doc :sym` | Get the docstring (if any); returns `:Option<:String>` |
| `:wat::reflect::source-loc :sym` | Get source file + line + col |
| `:wat::reflect::type-of :sym` | Get the type signature |
| `:wat::reflect::list :namespace` | List all symbols in a namespace |

These primitives are the user's current arc 109 work. wat-help
is a CLIENT of them; this arc doesn't design them.

## Integration with wat-fmt

The composition step builds a HolonAST representing the symbol's
form. wat-fmt's existing capability is:

```scheme
(:wat::fmt::format :ast :HolonAST -> :String)
```

(or whatever signature wat-fmt actually exposes; arc 003's slice
1 will firm this up)

wat-help calls this on its composed AST. **The output reads
identical to source code through wat-fmt** — same indentation,
same wrapping, same conventions. Users see help output that's
consistent with their codebase's formatting.

This is the load-bearing property: wat-help isn't inventing a
new "help format"; it's using the same formatter that produces
the user's source code on disk.

## Comparison to wat-doc (arc 006)

| | wat-doc | wat-help |
|---|---|---|
| Input | Source files (build-time) | Symbol table (runtime) |
| Output | Persistent docs (HTML / markdown / etc.) | EDN string (one symbol) |
| Consumers | Doc readers; CI doc-builds; web | Interactive (REPL / pause / MCP) |
| Per-symbol vs whole | Whole codebase | One symbol at a time |
| Cross-references | Rich (symbol graph; callers; callees) | Minimal (signature + body) |
| Live? | Static; built periodically | Live; reflects current frozen world |

**They're complementary, not redundant.** wat-doc is for the
"I'm reading the manual" use case; wat-help is for the "I'm
debugging at the REPL and need to know what this function does
RIGHT NOW" use case.

They might share a small lower-level "format-symbol-info" helper,
or they might not; the architecture sorts out at impl time.

## Error model

```scheme
(:wat::core::enum :wat::help::HelpError
  ((SymbolNotFound (sym :Symbol)))
  ((ReflectionError (cause :HolonAST)))    ; substrate reflection failed
  ((FormatError (cause :HolonAST)))        ; wat-fmt failed
  ((PrimitiveOpaque (sym :Symbol)))        ; core primitive with no
                                            ; reflection surface (rare)
  )
```

Failure-engineering position: never panics. Symbol-not-found is
the most common error; returns clearly. Reflection failure (if
the substrate's reflection primitive errors) propagates with
context. Format failure (if wat-fmt errors on the composed AST)
propagates with the AST that broke.

## Integration with wat-repl

Per user direction, wat-repl depends on wat-help. The repl's
special commands route through wat-help:

| Repl command | wat-help call |
|---|---|
| `:help <sym>` | `(:wat::help :sym)` |
| `:doc <sym>`  | `(:wat::help/doc :sym)` |
| `:source <sym>` | `(:wat::help/source :sym)` |
| `:type <sym>` | `(:wat::help/type :sym)` |
| `:ls <namespace>` | `(:wat::help/list :namespace)` |

Arc 012's DESIGN should be updated to reflect this dep. (Earlier
sketch had these commands as repl-internal; this arc extracts
them to wat-help so other consumers — pause sessions, MCP tools,
future LSP — get the same surface.)

## Integration with wat-cli — the "just works" property

Per user direction (2026-05-03):

> *"wat-repl needs to be depended on by wat-cli .... when a user
> compiles their own 'my-wat-cli' they can have the repl be used
> with whatever symbols their project created.. so help /just
> works/ in their env"*

wat-cli (per arc 099/100/101 of wat-rs proper) is the user-extensible
CLI surface. Each user compiles their own `my-wat-cli` with their
batteries:

```rust
// (sketch) in user's crate, building my-wat-cli:
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let batteries: Vec<Battery> = vec![
        // wat-rs default batteries
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register,    wat_sqlite::wat_sources),
        // wat-help is a default battery — ships with EVERY wat-cli
        (wat_help::register,      wat_help::wat_sources),
        (wat_repl::register,      wat_repl::wat_sources),
        (wat_fmt::register,       wat_fmt::wat_sources),
        // user's own batteries
        (my_app::register,        my_app::wat_sources),
    ];
    wat_cli::run_with_args(&args, &batteries)
}
```

After compilation:

```
$ my-wat-cli repl
my-wat-cli> (:wat::help :my-app::transfer-funds)
;; → formatted EDN of the user's transfer-funds function
my-wat-cli> :help :my-app::User
;; → repl command routing to (:wat::help :my-app::User)
;; → formatted EDN of the user's User type
my-wat-cli> :ls :my-app
;; → list of all symbols in :my-app namespace
```

**Zero extra setup.** The user's `my-wat-cli` ships wat-repl + wat-help
as defaults; help reflects their frozen symbol table; their custom
batteries' symbols are visible to help without registration.

This is the load-bearing UX win. Without it, help is a library users
must wire up. With it, help is an interactive surface every wat-cli
user gets for free.

Slice 5 of the SLICE-PLAN is dedicated to verifying this property
end-to-end with a sample user-compiled CLI.

## Integration with wat-mcp (arc 006)

Once wat-mcp ships, wat-help can be exposed as an MCP tool:

```
Tool: wat-help
Input: { symbol: "my-app::transfer-funds" }
Output: { formatted: "(:wat::core::define ...)" }
```

Claude (or any MCP-speaking agent) gets typed reflection over
a running wat-vm. This is a DIRECT enabler for the
Claude-as-measurer vision (arc 005's captured beat) — agents
need to UNDERSTAND the program they're inspecting, and wat-help
is the canonical "what is this symbol?" answer.

## Per the four questions

- **Obvious?** ✅✅ — `(:wat::help :sym)` reads as everyone
  expects; small surface; familiar pattern
- **Simple?** ✅ — one primary function; small set of typed
  variants; tight crate scope
- **Honest?** ✅✅ — output is the form (not synthesized prose);
  formatted via wat-fmt's rules (consistent with source);
  errors are typed
- **Good UX?** ✅✅ — every interactive consumer lands on the
  same surface; REPL users / pause sessions / MCP agents all
  see consistent output

Strong shape. Could be ✅✅✅ Honest if we structurally
guarantee that the formatted EDN is round-trip-identical to the
substrate's stored form (i.e., if a user could pipe the help
output back into wat-fmt and get the same string). Worth
confirming in slice planning.

## Cross-references

- **arc 003 (wat-fmt)** — formatter dep (foundation)
- **arc 005 (wat-pause)** — uses wat-repl which uses wat-help
- **arc 006 (wat-doc)** — complementary static docs
- **arc 006 (wat-mcp)** — exposes wat-help as MCP tool
- **arc 012 (wat-repl)** — primary consumer; routes special
  commands through wat-help
- **arc 109 (wat-rs mass refactor)** — supplies the reflection
  primitives this crate consumes
- **DEPENDENCY-DOCTRINE.md** — wat-fmt + reflection primitives
  as chosen deps; no new external Rust deps

## Open architectural questions

A. **Output as String vs HolonAST.** Currently `:wat::help`
   returns `:String` (formatted EDN). Should it return the
   raw HolonAST and let the consumer format? Lean: return
   `:String` from the canonical form; expose
   `:wat::help::ast :sym` as a separate function for consumers
   that want raw AST (would benefit MCP tools and agents that
   want to process structurally rather than display).

B. **Cross-reference depth.** Should help output include
   "called by" / "calls" / "imports" cross-references? Lean:
   minimal in slice 1 (just signature + body + doc); cross-refs
   in a later slice as separate `:wat::help/refs :sym` function.

C. **Macro expansion display.** For macros, should help show
   the template + an example expansion? Lean: yes for slice 2;
   non-trivial; depends on substrate macro reflection.

D. **Synthesized forms for core primitives.** Core types like
   `:wat::core::HashMap` have no wat source. The synthesized
   form is approximate. Should wat-help be honest about
   "this is a substrate primitive; here's the contract" framing,
   vs presenting it as a regular form? Lean: distinguish via a
   `:wat::core::primitive` wrapper form so the user can tell.

E. **Multilingual help.** Eventually agents may want help in
   different output formats (markdown for chat; plaintext for
   terminal; structured JSON for tools). Out of scope for v1;
   sibling crates (`wat-help-markdown`?) if needed.

## What's NOT in scope

- **Static documentation generation** — that's arc 006 wat-doc
- **Code search / find-symbol-by-pattern** — separate concern;
  sibling arc if needed
- **Autocomplete** — wat-help PROVIDES the surface autocomplete
  consumes; the autocomplete UX is application-tier
- **LSP server** — could be built on wat-help + wat-fmt later;
  not this arc's concern
- **Modifying source files** — wat-help is read-only reflection
- **Agent-driven editing** — agents can READ via wat-help; any
  WRITE flows through different surfaces (typed function calls;
  pause-session evaluations)
