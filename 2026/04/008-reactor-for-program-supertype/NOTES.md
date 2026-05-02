# Reactor for the Program<I,O> supertype — sketch

User direction 2026-04-30 during arc 112 slice 4 sweep:

> the threads we're working here... they are kernel threads.. not
> "green threads" like in go?...
>
> can you jot some notes down ~/work/holon/scratch/ -- i want to
> think on this later... this basically means we need something
> like a reactor who runs whatever is pending?..

This is a design-exploration scratch — not an arc, not a DESIGN.
Captured so the question doesn't get lost.

## Today's substrate

Every running Program in wat is a kernel thread or an OS process:

| Verb | Mechanism | Cost (rough) |
|---|---|---|
| `:wat::kernel::spawn` (arc 060) | `std::thread::spawn` | ~1MB stack, syscall context-switch |
| `:wat::kernel::spawn-program` → `Thread<I,O>` | same | same |
| `:wat::kernel::fork-program` → `Process<I,O>` | `libc::fork(2)` | ~MBs (COW), waitpid + IPC pipes |

NOT today:

- Green threads (Go's M:N goroutines, ~KBs each, userspace-scheduled)
- Async tasks (Tokio-style cooperative tasks; pollable Futures)
- Virtual threads (JVM Loom)

## Where this pinches

**Scaling** — kernel threads top out at ~1000s on most systems
before pthread limits or memory pressure bite. wat Programs scale
accordingly. For high-fanout workloads (10k+ concurrent
Programs) the substrate hits walls.

**Cost per spawn** — `spawn-program` is heavy by green-thread
standards. Fine for "tens of services" architectures, expensive
for "spawn one Program per request" patterns.

**Where wat will eventually want lighter** — if the trading lab,
DDoS lab, or any future domain wants per-event Programs (one per
candle, one per packet, one per LLM call), kernel threads are the
wrong unit. Need something cheaper.

## The reactor question

User's intuition: "we need something like a reactor who runs
whatever is pending?"

Right. To get green-thread economics, the substrate needs:

1. **An executor** — schedules pending tasks (when one yields,
   pick another to run). Owns a queue of runnable tasks +
   worker threads that pull from it.
2. **A reactor** — watches I/O and wakes tasks when their I/O
   becomes ready. epoll/kqueue/io_uring under the hood.
3. **Suspend/resume primitives** — tasks can yield (suspend),
   reactor wakes them (resume), executor schedules them again.

Tokio (Rust async runtime) IS this. `async/await` + Future trait
+ Waker. `tokio::spawn` returns a JoinHandle whose semantics
roughly mirror our `Thread<I,O>` — but scheduled cooperatively
in userspace.

## How this fits the Program<I,O> supertype

The `Program<I,O>` supertype (arc 109 § J) is naturally
extensible. Today two satisfiers (Thread, Process); a third
becomes possible:

```
Program<I,O>  ⟸  Thread<I,O>    |  Process<I,O>   |  Task<I,O> (?)
                  kernel thread     OS process       async-runtime
                                                     backed runnable
```

`Task<I,O>` would satisfy the same protocol — `stdin: ?`,
`stdout: ?`, `stderr: ?`, `Program/join-result → Result<unit,
TaskDiedError>`. Same `process-send` / `process-recv` API. Same
match-arm shape at every receiver.

The user-facing protocol (stdin/stdout/stderr/join-result) maps to
async-task internals via:

- `stdin` → an async-channel Sender that the task awaits on its
  `recv` side
- `stdout` → an async-channel Receiver the parent awaits to read
- `stderr` → same shape
- `join-result` → the JoinHandle's await + outcome translation

The principle "hosting is user choice; protocol is fixed"
(arc 114) covers this exactly. User picks Task<I,O> when they
want millions; Thread<I,O> when they want preemptive blocking;
Process<I,O> when they want isolation.

## Open design questions (think on later)

### 1. Does adding a reactor mean wat itself becomes async?

Today wat's evaluator is synchronous-blocking. A `process-recv`
call calls `read_line` which blocks the calling kernel thread.
For a Task<I,O> producer, the parent's recv would need to be
ASYNC-AWAIT-able — meaning the wat-level call site has to suspend
the task it's running in.

That's a runtime-deep change. Either:
- (a) Wat evaluator gains async support — every comm verb is a
  potential suspension point. Big substrate addition.
- (b) Tasks are run BY a reactor but consumed by Thread/Process
  callers via blocking adapters. Limits the win — you can spawn
  10k Tasks but you still need 10k Threads to talk to them.
- (c) Stackful coroutines — Tasks run on dedicated user-space
  stacks the reactor switches between. wat's evaluator stays
  synchronous; the SCHEDULING is what changes. Closer to Go's
  goroutine model.

(c) is probably the right shape for wat — preserves the
synchronous-look-and-feel at every call site while enabling
cooperative scheduling underneath. Crates like
`stackful-future` / `mio` + custom stack-switch could prototype
this.

### 2. What's the cost model the user reaches for at the call site?

Today: `spawn-thread` for cheap-ish, `fork-program` for isolated,
both heavy. With Task<I,O>: `spawn-task` (or whatever) for
cheap-as-in-millions-per-runtime.

The four questions for naming the verbs:

- Obvious? `spawn-task` mirrors `spawn-thread`; both produce
  Programs. ✓
- Simple? Three verbs is finite; user picks based on a clear
  cost trade-off. ✓
- Honest? "Task" implies cooperative scheduling, less isolation
  than Thread, much less than Process. ✓
- UX? Three options, each with a clear cost narrative. ✓

### 3. What about the reactor's lifetime?

A Tokio runtime is global state — typically one per process,
started at main, dropped at shutdown. Wat would need similar:
- ONE reactor process-wide
- Started when first `spawn-task` fires (or eagerly at startup)
- Dropped when the last Task completes + the process exits

This is a SHARED resource. Wat's ZERO-MUTEX doctrine applies —
the reactor's internal state has to be lock-free or futex-based.
Tokio uses Mutex internally; wat's reactor would either accept
that as a substrate footnote OR build a custom one.

### 4. How does this interact with arc 113's Vec<ProgramDiedError>?

Same shape works. A Task that dies has a `TaskDiedError`
satisfying `ProgramDiedError`. The chain conjs across hosts
(Thread → Task → Process all valid hops). The runtime that
discovers a Task panicked can synthesize the error the same way
arc 060's `catch_unwind` does for spawn.

### 5. Does it interact with arc 112's transport asymmetry?

Yes. Transport for Task<I,O>:

| Host | Wire format | Cost |
|---|---|---|
| Thread<I,O> | crossbeam Sender<Value> (Arc-on-channel) | zero-copy |
| Process<I,O> | OS pipe + line-delimited EDN | serialize + write |
| Task<I,O> | async-channel (tokio::sync::mpsc?) Sender<Value> | zero-copy in-runtime |

Task and Thread share "in-memory typed values" — both can pass
Arc<Value>. The difference is scheduling, not transport.
process-send / process-recv would dispatch on the concrete
satisfier and route through the appropriate channel kind.

### 6. Could we just lean on Tokio?

Pragmatic shortcut: bind wat's runtime to Tokio. Every `Program`
becomes a Tokio task. spawn-program / spawn-thread both produce
Tokio tasks; fork-program still uses fork(2) but the parent waits
in the Tokio runtime.

Pros:
- Battle-tested executor + reactor
- Solves scheduling, I/O, timer, channel infra in one dependency
- Future Rust interop story (a wat program is a Future)

Cons:
- Forces wat's runloop async (at the Rust level, not necessarily
  at the wat level if we use stackful coroutines on top)
- Tokio's threading model has its own gotchas (work-stealing
  scheduler, unbounded-task hazards, runtime shutdown ordering)
- Tokio uses Mutex internally — substrate footnote against
  ZERO-MUTEX doctrine

Honest read: Tokio is the obvious tool, but "we built our own
lock-free scheduler + reactor on top of mio + stackful coroutines"
is the more substrate-honest answer for wat's character. Not
worth the price today; might be worth it eventually.

## Pointers

- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` — current
  spawn shape (kernel threads, ProgramHandle).
- `docs/arc/2026/04/103-kernel-spawn/` — spawn-program substrate.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § J — Program
  supertype split. Naturally extensible to Task<I,O>.
- `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` — names the
  meta-principle "hosting is user choice; protocol is fixed."
  This document's existence makes Task<I,O> additive when the
  time comes.

## When to pick this back up

Probably when the user surfaces a workload that genuinely needs
10k+ concurrent Programs — high-fanout RPC, per-event handlers,
or a "spawn one program per X" architecture where X is in the
millions. Until then, the kernel-thread substrate is sufficient,
and the Program<I,O> supertype keeps the door open.
