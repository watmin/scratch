# wat-common-erlang-flavor — Erlang as a wat surface — sketch

User direction 2026-05-01, after the Haskell + ML demos in 011
landed:

> we can do erlang too?...
>
> [...later, after the demos walked through inline...]
>
> did you get your erlang rosetta stone down?.. i don't want to
> miss that either

This is a design-exploration scratch — not an arc, not a DESIGN.
The Erlang demos given in conversation didn't make it onto disk
during 011/012's drafting; this entry preserves them.

## Why Erlang is the most aligned flavor

The other flavor sketches (009 Clojure, 011 Haskell + ML) are
**data-flavored** — they shape how Option / Result / list look.
The Erlang flavor is **concurrency-flavored** — it shapes how
processes / messages / supervision look.

And the load-bearing observation: **wat-rs's concurrency model
already IS Erlang's actor model**, just with strong typing
layered on. Arc 114's `Program<I,O>` contract is essentially
BEAM's process model:

- spawn a process → `:wat::kernel::spawn-thread`
- send messages in → `:wat::kernel::send` over a typed channel
- receive messages out → `match (recv chan) ...`
- "let it crash" → panic propagates via `Vec<DiedError>` chain
  (arc 113); supervision tree IS that chain.

An Erlang dev would feel *more* at home with wat than a Haskell
or Clojure dev would.

## Demo 1 — Ping/pong (the canonical Erlang first program)

### Erlang

```erlang
-module(pingpong).
-export([start/0]).

server() ->
    receive
        {From, ping} ->
            From ! pong,
            server()
    end.

start() ->
    Pid = spawn(fun server/0),
    Pid ! {self(), ping},
    receive
        pong -> ok
    end.
```

What's distinctive:
- `spawn(fun)` — every process is lightweight; spawn returns a
  `Pid` (process identifier)
- `Pid ! Msg` — `!` is the send operator; mailboxes are implicit
- `receive ... end` — pattern-matched mailbox read; blocks until
  a matching message arrives
- `self()` — every process knows its own Pid
- Atoms (`ping`, `pong`, `ok`) are first-class identifiers, no
  quoting

### Wat-rs FQDN canonical (post-Channel rename, post-arc-109)

```scheme
(:wat::core::define
  (:my::server (in :wat::kernel::Receiver<wat::core::keyword>)
               (out :wat::kernel::Sender<wat::core::keyword>)
               -> :wat::core::unit)
  (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
    ((Ok (Some :ping))
      (:wat::kernel::send out :pong)
      (:my::server in out))
    ((Ok :None) ())
    ((Err _) ())))
```

### Wat-common-erlang-flavor

```scheme
(erlang/defn (server)
  (receive
    (ping  (! From pong)
           (server))))

(erlang/defn (start -> :atom)
  (let* ((Pid (spawn server)))
    (! Pid ping)
    (receive
      (pong  :ok))))
```

The flavor's `spawn` macro pre-allocates the channel pair, hides
the explicit `in`/`out` parameters, and expands `(! Pid msg)` to
the substrate's `send` plus the auto-allocated reply mailbox.
`From` becomes an implicit binding inside `receive`'s pattern
arms.

## Demo 2 — Pattern-matched receive with multiple message kinds

### Erlang

```erlang
worker() ->
    receive
        {set, Key, Value} ->
            io:format("setting ~p = ~p~n", [Key, Value]),
            worker();
        {get, From, Key} ->
            From ! {ok, lookup(Key)},
            worker();
        stop ->
            io:format("done~n")
    end.
```

What's distinctive:
- One `receive` block, multiple patterns (each with its own
  action)
- Tuples `{set, Key, Value}` carry a tag (atom) plus payload —
  Erlang's idiomatic discriminated union
- A bare atom (`stop`) terminates the loop

### Wat-common-erlang-flavor

```scheme
(erlang/defn (worker)
  (receive
    ((set Key Value)
      (io/format "setting ~p = ~p~n" Key Value)
      (worker))
    ((get From Key)
      (! From (ok (lookup Key)))
      (worker))
    (stop
      (io/format "done~n"))))
```

The macro lowers each pattern to a `match` arm against typed enum
variants. The flavor probably defines a per-process `Message`
enum at expansion time from the receive patterns — Erlang's
untyped messages become wat's typed sum-type under the hood. So
Erlang devs gain compile-time exhaustiveness for free; the
substrate enforces it at freeze time.

## Demo 3 — Stateful counter (mini-GenServer)

### Erlang

```erlang
counter(N) ->
    receive
        {From, get}     -> From ! N,    counter(N);
        {From, incr}    -> From ! ok,   counter(N + 1);
        {From, reset}   -> From ! ok,   counter(0)
    end.
```

State is the parameter; the recursive call is the loop; pattern
matching dispatches the request kind.

### Wat-common-erlang-flavor

```scheme
(erlang/defn (counter (N :int))
  (receive
    ((From get)
      (! From N)
      (counter N))
    ((From incr)
      (! From :ok)
      (counter (+ N 1)))
    ((From reset)
      (! From :ok)
      (counter 0))))
```

This is exactly how wat services already work today (arc 089's
Console / CacheService driver pattern is this shape, modulo
naming). The Erlang flavor just gives it Erlang-y syntax.

## Concept-by-concept alignment

| Erlang concept | wat-rs substrate | Notes |
|---|---|---|
| `Pid` (process identifier) | `Thread<I,O>` (or `Process<I,O>`) | typed; satisfies arc 114's Program<I,O> contract |
| Mailbox | `Channel<T>` (post-rename) + Receiver<T> end | typed messages instead of untyped |
| `spawn(fn)` | `:wat::kernel::spawn-thread` | identical semantic shape |
| `Pid ! Msg` | `(:wat::kernel::send sender msg)` | typed send |
| `receive ... end` | `(match (recv chan) ...)` | typed pattern match |
| Atoms (`ok`, `error`, `noreply`) | `:wat::core::keyword` | already there |
| Tagged tuples `{set, K, V}` | enum variant `(set K V)` | wat enums; flavor maps tagged-tuple syntax to typed variants |
| `link(Pid)` / `monitor(Pid)` | (none directly — substrate gap) | future arc |
| Supervisor + restart | `Thread/join-result` + chained `Vec<DiedError>` (arc 113) | supervision tree IS the death-chain shape |
| "Let it crash" | substrate-as-teacher discipline; panic propagates via DiedError chain | already the model |
| `gen_server` patterns | wat's Service driver + HandlePool | the lab's services ARE gen_servers |
| Hot code reload | (none — substrate gap) | future arc |
| `process_info` / introspection | (none — substrate gap) | future arc |
| Distributed Erlang network protocol | (none — substrate gap) | future arc |

## Honest gaps (substrate work that would land for Erlang devs)

| Gap | What Erlang gives | Substrate work |
|---|---|---|
| **`link(Pid)` / monitor relationships** | Process A linked to B dies if B dies | Arc TBD: substrate adds `Thread/link` / `Process/link`; cascading panic via the existing arc 113 chain |
| **Hot code reload** | Replace a running process's code | Arc TBD: freeze-time symbol table grows a "swap" verb; live programs see new bindings on next dispatch |
| **`process_info` introspection** | List mailbox depth, status, etc. | Arc TBD: substrate exposes per-Thread/Process introspection verbs |
| **Distributed Erlang wire** | Pid-aware network protocol (`@host` syntax) | Arc TBD: spawn-remote (or similar); EDN over TCP with peer authentication |

These are the four arcs an Erlang shop would queue before
adopting wat. Each one is plausible substrate work; none is
substrate-impossible.

## Where the flavor strengthens Erlang

- **Typed messages.** Erlang's mailbox accepts anything; wat's
  channel is typed. The flavor generates a per-process Message
  enum at expansion. Compile-time exhaustiveness comes free.
- **Compile-time process IDs.** Erlang Pids are runtime values;
  wat Thread<I,O> are compile-time-typed. Misdirected sends
  become type errors at freeze time, not runtime crashes.
- **Cleaner supervision trees.** Arc 113's `Vec<DiedError>` chain
  is a typed tree; Erlang's exit signals are untyped atoms. Same
  semantic shape; richer carried data.

## Where the flavor *loses* Erlang

- **Untyped flexibility.** A real Erlang shop using `term()`
  (any-term) APIs everywhere wouldn't translate. The flavor
  pushes typed messages; users who want truly dynamic dispatch
  reach for `:wat::holon::HolonAST` (arc 057's universal
  algebra value).
- **Implicit `self()`.** Erlang's `self()` exposes the current
  process's Pid; wat doesn't have ambient process identity. The
  flavor would need to thread `self` through the spawn closure
  explicitly.
- **Unlimited mailbox depth.** Erlang mailboxes grow unbounded;
  wat channels are bounded (per arc 089's `make-bounded-queue`)
  by design. Producers block on full; not Erlang's async-by-
  default semantics.

## When this becomes real

Same lifecycle as the other flavors (009/010/011/012):

- **Phase 0** (now): Wait. Arc 109 mid-flight; substrate
  vocabulary moving.
- **Phase 1**: Clojure flavor lands first (lab proof — 009).
- **Phase 2**: Erlang flavor next is the high-value play because
  the substrate's concurrency model is already 90% there. The
  remaining 10% (link/monitor) is one or two future arcs.
- **Phase 3+**: Cross-language Erlang↔Clojure interop on the
  same wat-vm. Trading shops running Clojure-flavor business
  logic + Erlang-flavor reliability glue. One runtime; two
  surfaces.

## Cross-references

### Sibling scratch entries

- `009-substrate-fqdn-userspace-shorts/NOTES.md` — the principle.
- `010-clojure-emits-wat/NOTES.md` — Clojure as a compile-target
  language (different from a wat-side flavor).
- `011-wat-common-flavor-comparison/NOTES.md` — the data-side
  flavor demos (Haskell, ML, Clojure). Erlang gets its own entry
  here (013) because it's concurrency-flavored, not data-flavored.
- `012-wat-as-polyglot-lowering-target/NOTES.md` — the meta-doc;
  Erlang's gaps (link/monitor/hot-reload/distributed) are listed
  there as candidate substrate arcs.

### Substrate alignment

- `wat-rs/docs/arc/2026/04/057-wat-holon-namespace/` — HolonAST
  as the polymorphic value type; what an Erlang `term()` lowers
  to when the flavor's typed-message generator can't pin a type.
- `wat-rs/docs/arc/2026/04/089-wat-db-substrate/` — Service
  driver + HandlePool pattern; the gen_server analog. Lab's
  services proved this already.
- `wat-rs/docs/arc/2026/04/103-kernel-spawn/` — `spawn-program`
  + `fork-program`; the Process<I,O> shape that's the typed
  out-of-process analog to BEAM's distributed processes.
- `wat-rs/docs/arc/2026/04/113-cascading-runtime-errors/` —
  chained DiedError; supervision tree shape.
- `wat-rs/docs/arc/2026/04/114-spawn-as-thread/` — Program<I,O>
  contract; Thread<I,O> as the in-memory actor.
- `wat-rs/docs/arc/2026/04/117-scope-deadlock-prevention/` —
  the lockstep discipline; the structural rule that prevents
  the "actor + mailbox + supervisor" pattern from deadlocking.

## Status — append more here as the idea matures

- 2026-05-01: scratch captured during arc 109 slice 1f sweep,
  after the Erlang demos in conversation didn't make it into 011.
  Three demos preserved (ping/pong, pattern-matched receive,
  stateful counter); BEAM-alignment table; gaps identified as
  candidate substrate arcs; lifecycle phased. The strongest
  observation: wat's concurrency model is already Erlang's,
  with types added. Adoption requires only ~4 substrate arcs
  to close the gap.
