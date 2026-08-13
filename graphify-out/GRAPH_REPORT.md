# Graph Report - rsnetstat  (2026-08-13)

## Corpus Check
- 6 files · ~4,277 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 66 nodes · 110 edges · 7 communities (5 shown, 2 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.75)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `de78403a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- RouteEntry
- rsnetstat
- routing.rs
- flags.rs
- main.rs
- graphify knowledge graph workflow
- rsnetstat

## God Nodes (most connected - your core abstractions)
1. `RouteEntry` - 15 edges
2. `rsnetstat` - 8 edges
3. `message_to_route()` - 7 edges
4. `JsonRoute` - 6 edges
5. `main()` - 6 edges
6. `fmt_addr()` - 6 edges
7. `route_table()` - 6 edges
8. `sort_entries()` - 5 edges
9. `gateway_sort_key()` - 5 edges
10. `print_section()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `JsonRoute` --references--> `DecodedFlag`  [EXTRACTED]
  src/main.rs → src/flags.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Routing table ingestion via BSD socket** — readme_rsnetstat, readme_bsd_routing_socket, readme_net_route_crate, readme_kernel_routing_table [EXTRACTED 0.85]

## Communities (7 total, 2 thin omitted)

### Community 0 - "RouteEntry"
Cohesion: 0.26
Nodes (9): fmt_addr(), gateway_sort_key(), if_name(), JsonRoute, RouteEntry, IpAddr, Option, String (+1 more)

### Community 1 - "rsnetstat"
Cohesion: 0.23
Nodes (12): BSD routing socket (PF_ROUTE / sysctl NET_RT_DUMP), CIDR-complete destination output, Interface index to name resolution, JSON output with stable schema, KAME link-local IPv6 %zone suffix, Kernel routing table, macOS platform target, net-route crate (+4 more)

### Community 2 - "routing.rs"
Cohesion: 0.29
Nodes (13): rt_msghdr, sockaddr, dump_table(), mask_to_prefix(), message_to_route(), RawRoute, route_table(), IpAddr (+5 more)

### Community 3 - "flags.rs"
Cohesion: 0.33
Nodes (6): decode(), DecodedFlag, FlagDef, letters(), String, Vec

### Community 4 - "main.rs"
Cohesion: 0.18
Nodes (13): Box, Error, Ipv6Addr, Item, Iterator, Cli, col_width(), is_zone_scoped() (+5 more)

## Knowledge Gaps
- **6 isolated node(s):** `FlagDef`, `rsnetstat`, `Sort and reverse options`, `Route type classification`, `graphify knowledge graph workflow` (+1 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `JsonRoute` connect `RouteEntry` to `flags.rs`, `main.rs`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `DecodedFlag` connect `flags.rs` to `RouteEntry`?**
  _High betweenness centrality (0.089) - this node is a cross-community bridge._
- **Why does `RouteEntry` connect `RouteEntry` to `main.rs`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **What connects `FlagDef`, `rsnetstat`, `Sort and reverse options` to the rest of the system?**
  _6 weakly-connected nodes found - possible documentation gaps or missing edges._