# The programmable firewall — iptables/nftables replaced by a real rules engine + firewalls-as-wat-programs

> **Provenance / status (2026-06-28, co-design).** The **reframe is the builder's** — *"replace iptables"*
> is a static packet enforcer with ordering problems; the universal need is *allow/deny/rate-limit* done by
> a **proper rules engine**, not a linear-scan ordered list; the new work is the **ergonomics** (firewalls
> expressed as wat programs that emit rules-as-EDN, enforced via rete-in-kernel). The **grounding (the engine
> is already proven; it already eats EDN) and the layering/synthesis are the apparatus's**, marked.
> **Status: thesis capture — the engine is SHIPPED (3–4 months ago); the ergonomics is the new build.**

---

## The reframe — the niche vs the universal

The DDoS/anomaly work (engrams, CCIPCA online-subspace detection, autonomous rule derivation) is **niche** —
most operators don't run a DDoS scrubber and don't have the NICs that matter. But **everyone** runs a packet
filter: allow / deny / rate-limit. **That is iptables / nftables** — and the hate they get is real: ordered
rule lists where the worst case is *every rule evaluated per packet*, hit-count maintenance, the chain/table
model, arcane syntax. The builder: *"i thoroughly despised ordering problems… manage rules as a tree."*

The universal product is **iptables/nftables replaced by a proper rules engine that does *real* rule eval** —
constant per-packet cost regardless of rule count, **no ordering**, expressed ergonomically as **programs**.

## The engine is ALREADY PROVEN — `series-003`, the DDoS lab (grounded 2026-06-28)

The hard half is on disk and has been since February. **Rete-in-kernel:**

- **1,000,000 rules, ~5 tail calls/packet, O(field-depth) not O(rules)** (`series-003-003`, `de755615`). The
  Rete **beta network** is DAG-compiled in userspace at rule-load (shared subtrees, content-hash dedup); the
  **alpha network** is the eBPF tail-call **walker** that traverses the pre-joined tree, one field per level.
  *"The tree doesn't evaluate rules, it prunes the space."*
- **Rule ordering eliminated entirely** (`series-003-003`, Likely-Contributions, verbatim): *"There's no rule
  ordering because there's no rule list… irrelevant rules don't cost anything — they're in branches never
  entered… per-packet cost bounded by field count, not rule count."* **This is the iptables indictment,
  answered.**
- **The rule format is EDN** (`series-003-004`): `{:constraints [(= proto 17) (= src-port 53) (> ttl 200)]
  :actions [(rate-limit 500 :name ["ns" "name"]) (drop)] :priority 200}` — equality, range, bitmask
  (`mask-eq`, `tcp-flags-match`), and `l4-match` (sparse byte patterns with masks) predicates; named/shared
  rate-limit buckets. The engine **already consumes rules-as-EDN.**
- **Blue/green atomic deploy** (two tree slots; a single `TREE_ROOT` flip; zero drop during updates).

**The hard part is done.** What remains for the iptables replacement is *not* the engine — it is the
ergonomics on top of it.

## The new work — the ergonomics: firewalls-as-wat-programs (the builder's vision)

iptables makes you *hand-write an ordered rule list and pray about ordering.* The replacement: **write the
policy as a program.** A user writes a **wat program** describing the firewall — *how to manage TCP state,
how to manage rate limits, ip-sets, allow/deny* — and the program **emits rules-as-EDN**, which the proven
rete-in-kernel enforces. The builder: *"that's just wat forms who emit rules-as-edn and the rules-as-edn are
enforced via rete-in-kernel."*

The pipeline, and it is the whole wat thesis pointed at firewalls — *programs emit data; data is enforced:*

```
  wat program (the policy)  →  rules-as-EDN (the compiled artifact)  →  rete-in-kernel (the enforced tree)
   TCP-state / rate-limits        {:constraints … :actions …}            DAG beta-tree + eBPF alpha-walker
   ip-sets / allow-deny           (already the engine's input)            O(fields), no ordering, 1M rules
```

The program is the **source of truth** (composable, typed, testable, **signable** — it is wat); the EDN is
the compiled rule set; the rete tree is the enforced form. A firewall stops being a fragile ordered list and
becomes a **program you can read, type-check, test, sign, and diff** — that compiles to a rule set an engine
enforces with no ordering and constant per-packet cost.

## Reachability — better than it looks, because the engine already eats EDN

Two builds, and the valuable one is **near**:

1. **The ergonomics layer — `wat → rules-as-EDN`. NEAR.** Because the rete-in-kernel engine already consumes
   EDN rules (`rules_parser.rs` / the tree compiler in the DDoS lab), a wat-policy-program that emits EDN
   **plugs into the existing, proven engine today.** No eBPF rewrite required to ship the iptables
   replacement — just the wat→EDN front-end, and the existing engine enforces it unchanged. This is the
   product.
2. **The all-in-wat rewrite (`005-wat-native-ebpf`) — the trust-purity upgrade, AFTER.** Author the
   rete-in-kernel itself (the alpha walker + the loader) in wat→eBPF, so no Rust/aya sits in the loop. Not
   required to ship the ergonomics; it is the end-state where the *whole* stack — policy program, rule
   compiler, kernel filter — is wat, signed, in the verified substrate (`005`).

So: ship the firewalls-as-programs ergonomics on the **existing** engine now; migrate the engine to all-wat
later for trust purity.

## Honest scope

- **Proven for L3/L4** — the packet fields (proto, addrs, ports, TTL, flags, `l4-match` byte patterns). That
  *is* iptables/nftables; the replacement sits squarely in the proven domain.
- **L7 / WAF is named future work** (`series-003-003`/`-005`): *"the same architecture could evaluate
  WAF-scale rule sets at depths traditional linear-scan engines can't touch — the L7 work hasn't started."*
  "Replace the WAF" is the next horizon (the epilogue's `wat-schema`), not this note's claim.
- **The `1.3M pps` figure was an accident** (a broken rate limiter), honestly documented — *not* the
  capability. The capability is **O(fields)-per-packet at 1M rules**; cite that, never the pps headline.

## The name — least load-bearing, flagged honestly

The builder floated **`wat-packet-filter` / `wpf`** (*"cheesy as fuck but… not wrong"*). Two issues: **`WPF`
collides with Microsoft's Windows Presentation Foundation** (the acronym will confuse), and "filter"
undersells it — it is a **firewall you write as a program**, not a filter. Closer working names: `fwat`,
`wat-fw`, "the rete firewall." The name can wait; the substance — *iptables, done as a program, enforced by
an engine that doesn't care how many rules you have* — is the thing.

## The convergence

The epilogue (2026-06-01) already named the end: *"`wat-schema` — the WAF replacement — the firewall I
started with, reborn in the language the firewall made me build."* This note is the **L3/L4** half of that
(iptables/nftables), and it is *closer than the WAF* because the engine is already proven and already eats
EDN. The firewall was the project's origin; the rules engine that replaces iptables is the firewall reborn,
and the wat-policy-program is the ergonomics it always lacked.

## Cross-references

- `series-003-{003,004,005}` (algebraic-intelligence.dev blog) — the proven rete-in-kernel: 1M rules,
  O(fields), no ordering, rules-as-EDN, DAG beta-tree + eBPF alpha-walker, blue/green deploy. The engine.
- `holon-lab-ddos/veth-lab/` — the running implementation (the `filter-ebpf` alpha walker + the `sidecar`
  EDN→tree compiler). What the wat ergonomics feeds; what `005` rewrites in wat.
- `../005-wat-native-ebpf/DESIGN.md` — the all-in-wat eBPF toolchain (the trust-purity substrate for the
  engine itself).
- `algebraic-intelligence.dev` epilogue — *"the firewall I started with, reborn in the language the firewall
  made me build"* (`wat-schema`, the WAF horizon).
