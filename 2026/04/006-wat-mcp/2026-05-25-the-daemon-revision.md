# wat-mcp — the daemon revision (2026-05-25)

Extends the April notes (`the-collapse`, `one-tool-surface`,
`break-as-notification`) and `2026/05/001-memory-as-hologram/` (the `recall`
consumer). Three weeks and the 109→170→236 arcs later, the design was
revisited cold. **The spine held; the reach grew exactly where the substrate
grew.**

## The convergence (now == then)

Re-derived without re-reading the April notes, the current thinking landed on
the *same spine* — independently, three weeks apart:

- one tool, "speak wat";
- EDN in, EDN out;
- `{"msg": "<edn>"}` in both directions (JSON only for MCP compliance);
- the server is a REPL; MCP is the dial-tone.

That's `the-collapse`, re-found. A self-convergence — the design sat at a
coordinate and the author walked back to it. So the spine is not provisional;
it is stable under a three-week re-think.

## The delta — growth, not drift

April framed `wat-eval` as **eval + pause** (it leaned on `wat-pause`: break,
inspect, override). The revisited thinking is broader: a **persistent daemon
that does real work** — spawns threads and processes, touches the external
world, evaluates forms like a long-lived REPL.

That reach wasn't in April because the primitives didn't exist yet. They
landed in between:

- `spawn-program` / `spawn-process` / `fork-program` + `ProgramHandle` /
  `join` (arc 170)
- the ambient kernel stdio trio — StdIn/StdOut/StdErr services (170)
- the matured **manual** service-program pattern + bounded paired channels
  (095/119)

The reach grew into precisely the new capability. Same spine, longer arm.

## spawn-program ≡ stdio → universe-residency, with Claude as a tier

The load-bearing recognition: **a wat program launched by an MCP client over
stdio is identical to a wat program spawned by a parent wat-vm.** Both read
EDN from stdin, write EDN to stdout. The program does not — *cannot* — know
which parent holds its pipes. That is **universe-residency**: the program is
universe-resident; the parent is the hosting decision; the program never sees
transport. The MCP client (Claude) is simply a new *tier*, alongside thread /
process / (future) remote.

Consequence: **wat-mcp is not a new mechanism.** It is `spawn-program` where
the parent happens to be Claude. The *only* delta from a wat-vm parent is the
JSON envelope — a wat-vm parent sends raw EDN; an MCP client wraps it
`{"msg": "<edn>"}`. That envelope is the entire new surface: a thin JSON↔EDN
shim at the boundary. Everything inside is shipped `spawn-program`.

## The daemon model

- A **persistent** wat-vm universe, launched by the MCP client, alive across
  many `tools/call`s.
- Holds state across calls: warm holon caches, long-lived spawned services, an
  accreting SymbolTable / environment.
- Each call: `:wat::edn::read` the `msg` → evaluate in the persistent
  universe → `:wat::edn::write` the result → back out as `{"msg": "<edn>"}`.
- Can spawn real work (threads / processes) that outlives a single call; can
  delegate *untrusted* forms to hermetic children (see transport-and-posture).
- `break` / pause (the April `break-as-notification`) is now *one capability*
  of the persistent universe, not the whole story.

See: `2026-05-25-transport-and-posture.md`,
`2026-05-25-purpose-think-in-functions.md`,
`2026-05-25-substrate-surface.md`.
