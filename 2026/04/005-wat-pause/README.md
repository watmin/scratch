# Scratch 005 — `wat-pause` — interactive pause for the wat substrate

**Started:** 2026-04-29.
**Status:** structural shape locked in conversation; substrate primitives
named; slice ordering drafted. Ready to migrate to a wat-rs arc when the
user is ready.

**Sibling materials:**
- `wat-rs/docs/arc/2026/04/099-wat-cli-crate/` — extracted CLI.
- `wat-rs/docs/arc/2026/04/100-wat-cli-public-api/` — `wat_cli::run(&[Battery])`.
- `wat-rs/docs/arc/2026/04/101-kill-wat-test-cli/` — single-purpose CLI.
- `wat-rs/docs/arc/2026/04/103-kernel-spawn/HOLOGRAM.md` — the framing
  that made this possible. The wat binary is the surface between
  Rust universe and wat universe; programs see through but cannot
  reach back. EDN+newline is the protocol that crosses surfaces.
- `wat-rs/docs/arc/2026/04/104-wat-cli-fork-isolation/DESIGN.md` —
  wat-cli always-forks; the cli's job collapses to "provide symbols
  and contain." The fork-and-proxy architecture is what makes a
  pause-shaped break primitive honest at the OS level.
- `wat-rs/docs/arc/2026/04/068-eval-step/` — `:wat::eval-step!`
  primitive; the foundation for `:next` / `:step` / `:finish`
  stepping (slice 4).
- `wat-rs/docs/arc/2026/04/016-failure-location-frames/` — `Span` +
  `CALL_STACK` already populate every frame; `:where` / `:up` /
  `:down` read from the existing data.
- `BOOK.md` Chapter 67 — *The Spell.* The networked pause (slice 5)
  is exactly this — coordinates publishable across machines via
  the same line-delimited EDN protocol; same primitive on a
  different transport.
- `BOOK.md` Chapter 71 — *Vicarious.* The cache is consumption.
  Pause is the ad-hoc query against the corpse pile — every cached
  terminal is a walker that died producing this answer for the
  present walker to read.

---

## What this scratch captures

User's recognition 2026-04-29, after walking arcs 097→104:

> "i think we just unlocked... repl?... the user's compiled wat has all
> the symbols... we can run wat --repl to jump into it?... is this
> true?... if this is true... can we implement something like ruby's
> pry?..."

> "binding.pry is /the most/ powerful thing i've ever seen in a
> program..."

> "rust being frozen is a blessing, not a curse...."

> "i want the binding.pry experience.... being able to measure my
> environment and continue... i don't know if cd even has meaning in
> wat.. everything is a fqdn..."

> "i think a requirement.. all pry forms are illegal if --pry isn't
> passed... we ship with pry but the user must enable it... if we see
> (:wat::pry::break) and we're not in pry mode - panic"

> "and we don't call it repl... we call it pry..."

[Verbatim quotes preserved. The arc was originally named `wat-pry`;
renamed to `wat-pause` on 2026-05-03 after gaze surfaced that the
borrowed Ruby name didn't speak in wat's voice. See INDEX.yaml
captured-beats for the rename direction. The substrate-form names
in the rest of this doc (`:wat::pause::break`, `--pause`, `wat-pause`)
reflect the new naming; the user's original direction above used
`:wat::pry::break`, `--pry`, `wat-pry`.]

The substrate has been collecting this capability arc by arc for
months without anyone naming it. Pause is what falls out of:

- arcs 099/100/101 making wat-cli a thin sovereignty surface.
- arc 102 making `:wat::eval-ast!` polymorphic so any expression's
  result can flow back through the EDN wire.
- arc 103 giving us the EDN+newline pipe protocol + the hologram
  framing that says programs ARE the surface, not co-residents.
- arc 104 making fork the structural form of containment — when
  `(:wat::pause::break)` fires, the child reads its own stdin and
  writes its own stdout the same way every other wat program does.

Nothing new architecturally. A small set of substrate primitives,
gated behind `--pause`, plus a wat-level loop, plus a rustyline
frontend. Pause-shaped UX from a freeze-locked substrate.

**Vocabulary:** the thing is called *pause*, not REPL. The flag is
`--pause`. The mode is *pause mode*. The namespace is `:wat::pause::*`.
The substrate-shipped entry-point is `:wat::pause::main`. The
read-eval-print loop is the implementation; pause is the name.

---

## What's locked

| Decision | Resolution |
|---|---|
| `wat --pause` (no source) shape | Drops into a pause session with batteries-only symbols. No user defines (no source loaded). |
| `wat --pause <entry>` shape | Loads the entry's source through freeze (so user defines + config setters land); invokes `:wat::pause::main` instead of `:user::main`. The entry's source is unchanged. |
| Bare vs entry — one code path | Same freeze pipeline. Entry source is `Option<String>`. The cli always invokes `:wat::pause::main` in pause mode; whether user defines are visible depends on whether an entry was given. |
| **Gating** | Pause symbols are registered ONLY when `--pause` is set. Without the flag, the `:wat::pause::*` namespace doesn't exist; freeze fails with `UnknownFunction(":wat::pause::break")` if any pause form appears. The pause battery is bundled with wat-cli but conditionally loaded. |
| `wat-pause` packaging | Either a `--pause` flag on `wat-cli` OR a separate `crates/wat-pause/` binary. Both equivalent. Separate binary is the wat-native answer once the frontend grows enough features (slice 3). |
| Define / redefine at the prompt | Forbidden. The freeze invariant holds. `:reload` rebuilds the frozen world from disk-edited source if you want new symbols. |
| `cd` analog | None. Wat is FQDN everywhere; no ambient self / current namespace to be inside of. **Tab completion is the FQDN-native navigator.** |
| `(:wat::pause::break)` shape | Substrate primitive. Captures current Environment + CALL_STACK; runs an inline pause loop on the program's own stdin/stdout; `:continue` returns; execution resumes at the next form. Only callable when `--pause` is set; otherwise the symbol doesn't exist. |
| Break stdio | Macro defaults to lexically-scoped `stdin`/`stdout`/`stderr` names. Verbose `:wat::pause::break-with-stdio` form takes them explicit when ambient names aren't in scope. |
| The pause loop | Itself a wat program shipped in `wat/std/pause.wat` (loaded only with `--pause`). Command dispatcher (`:ls`, `:show`, `:continue`, etc.) is wat code, extensible by batteries. |
| User's `:user::main` from prompt | Callable. It's just another symbol in the frozen world. `(:user::main stdin stdout stderr)` runs the entry program from inside the session. |

## Open

See:
- `unlock.md` — the recognition; what arcs 097-104 made true.
- `two-modes.md` — bare vs entry-loaded; one freeze pipeline with an entry-point swap.
- `gating.md` — `--pause` as the registration gate; freeze fails if pause forms used without the flag.
- `break-primitive.md` — `(:wat::pause::break)` — the binding.pry shape, with substrate-level Environment capture.
- `command-set.md` — `:ls` / `:show` / `:where` / `:continue` / etc. FQDN-explicit; tab completion as the navigator.
- `primitives.md` — the substrate additions (`:wat::pause::serve`, `break-with-stdio`, `completions`, `ls`, `show`, `where`, `frames`, `last-error`).
- `packaging.md` — `wat-cli --pause` flag vs `crates/wat-pause/` binary; equivalence and when to split.
- `slice-plan.md` — ordered slices: bare pause → break → rustyline → stepping → TCP attach.
- `open-questions.md` — what isn't decided yet.

The user explicitly named: *the freeze invariant being load-bearing
makes wat's pause more honest than Ruby's pry, not less.* When you
`:continue` from a break, you continue into exactly the program you
just inspected — no other thread mutated a method out from under
your inspection, no constant got reassigned, no plugin redefined a
function while you read its source. Ruby's pry has to fight the
language's mutability; wat's pause inherits the substrate's stability.
