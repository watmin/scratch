# wat-mcp — transport & security posture (2026-05-25)

## stdio-only, by construction

wat-mcp speaks MCP over **stdio** — the client launches the daemon and talks
JSON-RPC over its stdin/stdout. Not a fallback; the chosen, correct transport.
("wat remote programs" — UDS, loopback, TLS, mTLS — come later, as a separate
surface; see below.)

## The loopback-exploit class ceases to exist

An HTTP/SSE MCP server listens on a socket, and **loopback is not a trust
boundary.** A `127.0.0.1:PORT` listener is reachable by: any other local
process (no auth by default), a browser tab via localhost-CSRF / DNS
rebinding, a localhost port scan, any other user on a shared host. A real,
well-trodden exploit class for local services.

**stdio has no listener.** The daemon is a child spawned by the client; the
only channel is the inherited stdin/stdout fds. No address, no port, nothing
to `connect()` to. The entire class — loopback connect, DNS rebinding,
localhost CSRF, port scan, neighbor-on-host — does not get *defended*; it
**ceases to exist.** You cannot attack a socket that was never opened.

This is the doctrine — **"we don't sandbox, we refuse"** — applied to
transport: don't harden the port; have no port. The failure mode is
*structurally unavailable*, the same shape as the compile-time deadlock
walkers.

## Honest scope — stdio eliminates the *transport* surface, only

Two surfaces remain, different problems:

- **Payload.** The daemon has hands (spawns processes, hits sqlite, touches
  the world). stdio doesn't care what `{"msg": edn}` *says* — a malicious or
  prompt-injected agent can send a harmful form. Defense: `def-restricted`
  (arc 198) — whitelist which forms/callers may reach the dangerous verbs.
  Optionally **selective hermetic eval**: run *untrusted* forms in a hermetic
  child (the daemon spawns it, runs the form, gets EDN back). **Honest
  tradeoff:** a hermetic child cannot see the daemon's warm state, so
  structural analysis against the warm store must run *in* the daemon, not
  hermetic. Hermetic is therefore *per-form, gated by trust* — never global
  ("always hermetic" trades away the data-analysis use case).
- **Trust boundary.** It doesn't vanish — it *collapses* to one clear actor:
  the parent that spawned the daemon and holds the fds (the MCP client). Trust
  your launcher, and only your launcher — vastly smaller and more auditable
  than "anything that can reach a port."

## Lifecycle

Parent dies → pipes close → daemon can exit. No orphaned listening service.
The Pidfd / `spawn_lifelined` discipline (arc 170/213) is exactly the
clean-teardown machinery.

## Remote is a separate, future, explicitly-authenticated surface

"wat remote programs" (UDS, loopback, TLS, mTLS) come later — cryptographic,
not network-positional (per `WAT-NETWORK.md`). **The absence of a network
primitive in wat-rs is a feature,** not a gap: it keeps the local daemon
surfaceless. Adding HTTP/SSE would be a *deliberate* decision to opt back into
the loopback/network surface, separately authenticated — never an accidental
open port, never a default.
