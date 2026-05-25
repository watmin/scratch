# wat-mcp — shipped substrate surface (audit, 2026-05-25)

What's real to build a v1 from *today* (wat-rs), and the honest constraints.
Source: audit of `wat-rs/` docs / arc / stdlib, 2026-05-25.

**One-line:** a v1 is buildable today, in-process, over stdio, with a thin
JSON shim — strict EDN wire, the *manual* service-program pattern,
spawn/fork/process, SQLite, telemetry, capability bounds, holon encode +
Hologram cache. The only not-yet-built pieces are the two named frontiers
(durable-recall EngramHologram; the coherence gate).

## SHIPPED — build from these

| Piece | Name / shape | Arc |
|---|---|---|
| EDN wire | `:wat::edn::{read,write,write-notag}`, clojure-compatible | 219 |
| Service pattern | **manual** (`SERVICE-PROGRAMS.md`, `service-template.wat`); bounded `Channel<T>` + paired Request/Ack, `select`→`Chosen<T>`, `HandlePool` | 078/095/119 |
| spawn/fork/process | `:wat::kernel::{spawn-program,spawn-program-ast,fork-program,fork-program-ast,spawn-process,spawn-thread}`, `join`/`join-result`, `ProgramHandle<T>`; process tier = EDN-over-stdio pipes | 012/103/170 |
| stdio trio | `:wat::kernel::services::{StdInService,StdOutService,StdErrService}` | 170 |
| SQLite L3 | `:wat::sqlite::{open,execute-ddl,execute}`, `Param`, `ReadHandle` | 083-085 |
| Hologram cache | `:wat::holon::Hologram/{make,put,get,find,...}`; `:wat::holon::lru::HologramCache` + `HologramCacheService` — coordinate-keyed HolonAST→HolonAST, cosine filtered-argmax | 074/076/077 |
| encode + match | `:wat::holon::encode <ast>`→Vector; `coincident?`/`presence?` + floors; bridge family `:wat::holon::{to-holon,from-holon,to-wat,from-wat}` | 023/024/225 |
| capability bounds | `(:wat::core::def-restricted :name :restricted-to [:prefix::] expr)`; `#[restricted_to(...)]`; `:struct-restricted` | 198/203/210 |
| telemetry | `:wat::telemetry::Service<E,G>` (Request+Ack), `MetricsCadence<G>`, `Event`(Metric/Log), `:wat::telemetry::Sqlite` sink | 080-096 |

## DESIGNED-ONLY / NOT BUILT — design around these

- **`defservice`** — arc 209 open (DESIGN + SCORE-SLICE-1, *no INSCRIPTION*).
  **Use the manual service pattern.**
- **`EngramHologram`** (compaction-by-merge eviction) — does not exist.
  `HologramCache` is **LRU drop-on-evict**, the wrong polarity for durable
  memory (a coord with many deposits that hasn't been queried lately is *more*
  valuable, not less). A v1 `recall` works on `HologramCache`; durable wisdom
  waits on EngramHologram. (See
  `2026/04/004-inverse-hologram/ENGRAM-HOLOGRAM-DESIGN.md`.)
- **No network primitive** — no TCP/HTTP/RPC/RemoteProgram in wat-rs. stdio is
  the transport; a thin Rust host (e.g. `wat-cli --mcp`) owns the JSON-RPC /
  MCP framing + the `{"msg": edn}` envelope. Remote (UDS/TLS/mTLS) = future
  "wat remote programs."

## Renames to track (or the design will reference dead names)

- bridge family renamed: `atomize` / `materialize` / `atom-value` →
  `to-holon` / `from-holon` / `to-wat` / `from-wat` (arc 225).
- `RunDbService` / `LogEntry::Telemetry` names **superseded** — `LogEntry` is
  user-defined by design; telemetry ships zero entry variants.
