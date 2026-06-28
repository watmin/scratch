# wat from wire to service — the whole userland networking stack in wat (AF_XDP, smoltcp as oracle)

> **Provenance / status (2026-06-28, co-design).** The **thesis is the builder's** — a wat daemon that
> implements *its own* userland networking stack, AF_XDP from the wire up through TCP to an HTTP service,
> *"wat from wire to http service"*; smoltcp named as the reference; the TCP dragon declared **queued for
> demise, not feared** (*"have you not witnessed me slay every dragon that stands before me — they are
> queued, awaiting their demise"*). The **grounding (the two ends already on disk, the AF_XDP/XDP redirect
> path), the filter+stack composition, the smoltcp-as-oracle structure, the AOT-forcing-function read, and
> the typed-stack synthesis are the apparatus's**, marked. **Status: thesis capture — vision-level, not
> modeled-to-build.** Rides `005` (the filter), `001` (the AOT tier), and the `2026/05/009–011` HTTP arcs.

---

## The thesis

A **wat daemon that brings its own networking stack.** Frames arrive at the NIC; `wat-pf` (XDP, in-kernel)
decides allow / deny / rate-limit; the survivors are redirected to an **AF_XDP** socket; the daemon reads
them zero-copy and runs **its own Ethernet / ARP / IP / TCP / UDP**, terminating into the **wat HTTP
service** — and writes the responses back out the wire. **Every layer is wat. The kernel's TCP stack is
never in the path.** Wire to service, nothing foreign in between. *"wat from wire to http service."*

## You already have both ends — this is the missing middle

This is not a green field; it is a **gap between two designed ends**:

- **The bottom end — `005-wat-native-ebpf`** (`wat-pf`, `006`): the XDP program, wat at the NIC. The
  filter. Already the target of its own thread.
- **The top end — `2026/05/009-wat-http-server` + `010-wat-http-router` + `011-wat-http-client`**: the L7
  service, already designed.

What is unbuilt is the **stack between them** — the AF_XDP socket and the Ethernet/ARP/IP/TCP/UDP layers
that carry a frame from the redirect ring up to an HTTP request object. `007` is that middle.

## The architecture — one continuous wat path, filter and stack as the same redirect

```
  NIC ─▶ XDP (wat-pf, in-kernel) ──┬─▶ XDP_DROP            (deny / over rate-limit)
                                   ├─▶ XDP_PASS            (let the kernel stack have it, if ever wanted)
                                   └─▶ XDP_REDIRECT ─▶ AF_XDP socket (xsk)
                                                            │  zero-copy via UMEM
                                                            ▼
                              wat daemon (userland) ── reads frames off the RX ring
                                   Ethernet / ARP ─▶ IPv4/IPv6 ─▶ TCP / UDP / ICMP
                                                                  │
                                                                  ▼
                                                       wat HTTP service (009/010)
                                                                  │  response
                                   writes frames to the TX ring ◀─┘
                              NIC ◀────────────────────────────────
```

The **filter and the stack are one motion:** `wat-pf` chooses who survives at the NIC; the AF_XDP redirect
hands the survivors to the daemon; the daemon is the stack *and* the app. wat-pf decides *who gets in*; the
wat stack decides *what happens to them.* Two halves of one packet path, both wat.

## AF_XDP — the wire seam (grounded)

AF_XDP is the Linux best-of-breed primitive for *"raw frames to userland at line rate"*
([[project_wat_is_linux_best_of_breed]]): a socket whose **UMEM** is a shared memory region the kernel and
the daemon both map, and **four rings** — `FILL` / `COMPLETION` (UMEM management) and `RX` / `TX` (frames)
— move descriptors with **zero copy**. An XDP program `XDP_REDIRECT`s matched frames straight into the
`xsk` RX ring, bypassing the kernel network stack entirely. Production-proven (Cloudflare, Cilium).

For wat, the load-bearing fact: **the rings are fd-events.** The daemon arms the RX ring and waits the same
way it waits on any peer — *time and I/O arrive on the wire, not via a sleep* ([[mora]]); an AF_XDP socket
is one more **rx-only peer** in the three-loci `select'` model ([[project_three_loci_one_interface]]), and
the TX ring a tx peer. The networking stack is a reactor over ring-fd readiness — exactly the io_uring-native
shape the kernel/process work already grows.

## The oracles — smoltcp, and the HTTP arcs (the dual-impl doctrine, again)

The same doctrine that governs `wat-pf` (aya is a reference impl we satisfy) governs the stack:

- **smoltcp is the stack oracle.** A standalone, event-driven, `no_std`, no-heap-required TCP/IP stack in
  Rust — Ethernet/ARP/IPv4/IPv6/TCP/UDP/ICMP/DHCP. It **proves a userland stack is bounded and tractable**,
  and it is the **differential-test reference**: the same frames through smoltcp and through the wat stack
  must produce the same segments on the wire. *smoltcp is a reference impl we satisfy* — not a dependency we
  bind. (The kernel stack is a second oracle for conformance: real peers must interoperate.)
- **The `009`/`010` HTTP arcs are the L7 end** — the request/response surface the TCP layer terminates into.

## The honest hard edges (four-questions, no hype)

- **TCP is the dragon — and it is queued, not feared.** The hard part is not framing; it is the **state
  machine**: the three-way handshake, retransmission + RTO/RTT estimation (Karn, Jacobson), sliding windows
  + flow control, **congestion control** (Reno/CUBIC), SACK, delayed ACK, TIME_WAIT, and the long tail of
  edge cases that took smoltcp years. This is a **major arc**, the biggest in this note. The builder's
  framing, recorded: *"have you not witnessed me slay every dragon that stands before me — they are queued,
  awaiting their demise."* UDP / ICMP / ARP are the easy wins; **TCP-done-right is the whole fight**, and it
  is *scheduled*, not dodged. smoltcp's existence is the proof it is finite.
- **Line-rate TCP termination wants the AOT tier — so this is a *forcing function* for the compile-on-load
  work.** A tree-walker will not terminate connections at wire speed: the stack is **glue**, and glue is
  exactly what the interpreter taxes (`001`'s Amdahl point — wat is fast where work is primitive-dominated;
  a TCP state machine is the opposite). So `007` is the workload that finally uses wat *against its grain*
  and **earns native codegen** — one of the best reasons to build the AOT compile-on-load path
  ([[001-metered-evaluation/CEK-MIGRATION.md]] — the JIT annihilated, AOT the road). Honest: a *correct*
  stack is reachable on today's interpreter; a *line-rate* one is a post-AOT capability. Both are on the map.
- **The typed stack is the edge over smoltcp — the type system as armor, pointed at TCP.** wat's types can
  encode the TCP state machine so **illegal transitions are unrepresentable** (typed states / session-type
  shape) — a `CLOSED` socket has no `send`; a segment in the wrong state has no constructor. Same move as
  `wat-bpf` passing the verifier by construction: not parity with smoltcp, an edge — *the stack that cannot
  enter an illegal state because the state has no representation.*

## What is genuinely ours — the combination

Each piece exists; the **assembly does not**: a userland TCP/IP stack (smoltcp's domain) **+** an app server
(hyper's domain) **+** kernel-bypass ingress (AF_XDP) **+** a line-rate filter (`wat-pf`, eBPF) **+** all of
it **one signed, typed, homoiconic, migratable language** with **no foreign code in the path.** Nobody has
*wire-to-L7 in a single typed homoiconic signed substrate.* It is the **field-programmable host that brings
its own stack** — it does not borrow the kernel's TCP, it *is* the TCP.

## The convergence — the field-programmable host, completed to the SYN

`FIELD-PROGRAMMABLE-HOSTS.md` describes a host that becomes its installed purpose by signed `eval`. `007`
completes it **down to the wire**: *"you are now an HTTPS service"* now means it down to the **SYN** — the
installed program is not an app on top of the kernel's stack, it is the **whole stack plus the app**, signed
and typed end to end. Compose with `wat-pf` and a single wat daemon owns a packet's *entire journey* — NIC
to HTTP response — with the kernel providing only the redirect and the wire. The origin (a firewall,
on-host, line-rate) and this (the whole stack, on-host, line-rate) are the same instinct at full extension:
**the endpoint that brings everything it needs to protect and serve itself, on the silicon it already has.**

## Reachability — a component map, no person-years

- **UDP / ICMP / ARP + the AF_XDP socket + the reactor** — *near.* Bounded protocols, the rings are fds, the
  peer/`select'` model already fits.
- **A correct TCP** (handshake → established → teardown, retransmit, windowing) on the interpreter — *a real
  arc, bounded by smoltcp as the oracle.* The dragon, queued.
- **Line-rate TCP** — *gated on the AOT tier* (`001`); this is its forcing function, not a blocker for a
  correct-but-slower first cut.
- **The typed state machine** — the deepest design work and the genuine edge; where most of the effort earns
  its keep.

## Cross-references

- `005-wat-native-ebpf/DESIGN.md` — the XDP filter half (the `XDP_REDIRECT` that feeds the AF_XDP socket);
  the all-in-wat eBPF toolchain.
- `006-programmable-firewall/DESIGN.md` — `wat-pf`, the filter that decides who reaches the stack; the
  founding voice + the name.
- `../../2026/05/009-wat-http-server/` · `010-wat-http-router/` · `011-wat-http-client/` — the L7 end the
  TCP layer terminates into.
- `001-metered-evaluation/CEK-MIGRATION.md` — the AOT compile-on-load path this workload forces (glue-bound,
  against-the-grain → earns native codegen).
- `001-metered-evaluation/FIELD-PROGRAMMABLE-HOSTS.md` — the host that brings its own stack; "you are now an
  HTTPS service," down to the SYN.
- smoltcp (Rust) — the stack oracle we satisfy; the kernel stack — the interop oracle.
