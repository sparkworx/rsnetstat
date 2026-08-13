# Graph Report - rsnetstat  (2026-08-13)

## Corpus Check
- Corpus is ~2,062 words - fits in a single context window. You may not need a graph.

## Summary
- 43 nodes · 72 edges · 13 communities (7 shown, 6 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.75)
- Token cost: 21,144 input · 0 output

## Community Hubs (Navigation)
- RouteEntry Accessors
- Platform & CLI Concepts
- Main & Error Handling
- JSON Route Serialization
- Table Formatting
- CLI Parsing & Sorting
- Gateway Field Access
- IPv6 Address Formatting
- Kernel Routing Ingestion
- netstat Compatibility
- Route Classification & JSON
- graphify Workflow
- rsnetstat Tool

## God Nodes (most connected - your core abstractions)
1. `RouteEntry` - 14 edges
2. `rsnetstat` - 8 edges
3. `main()` - 6 edges
4. `fmt_addr()` - 6 edges
5. `sort_entries()` - 5 edges
6. `gateway_sort_key()` - 5 edges
7. `JsonRoute` - 4 edges
8. `print_section()` - 4 edges
9. `col_width()` - 4 edges
10. `SortKey` - 3 edges

## Surprising Connections (you probably didn't know these)
- `gateway_sort_key()` --references--> `RouteEntry`  [EXTRACTED]
  src/main.rs → src/main.rs  _Bridges community 0 → community 3_
- `print_section()` --references--> `RouteEntry`  [EXTRACTED]
  src/main.rs → src/main.rs  _Bridges community 0 → community 4_
- `RouteEntry` --references--> `String`  [EXTRACTED]
  src/main.rs →   _Bridges community 0 → community 6_
- `sort_entries()` --references--> `RouteEntry`  [EXTRACTED]
  src/main.rs → src/main.rs  _Bridges community 0 → community 5_
- `fmt_addr()` --references--> `IpAddr`  [EXTRACTED]
  src/main.rs →   _Bridges community 3 → community 7_

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Routing table ingestion via BSD socket** — readme_rsnetstat, readme_bsd_routing_socket, readme_net_route_crate, readme_kernel_routing_table [EXTRACTED 0.85]

## Communities (13 total, 6 thin omitted)

### Community 1 - "Platform & CLI Concepts"
Cohesion: 0.60
Nodes (5): Interface index to name resolution, KAME link-local IPv6 %zone suffix, macOS platform target, rsnetstat, Sort and reverse options

### Community 2 - "Main & Error Handling"
Cohesion: 0.50
Nodes (4): Box, Error, Result, main()

### Community 3 - "JSON Route Serialization"
Cohesion: 0.50
Nodes (4): IpAddr, Option, gateway_sort_key(), JsonRoute

### Community 4 - "Table Formatting"
Cohesion: 0.50
Nodes (4): Item, Iterator, col_width(), print_section()

### Community 5 - "CLI Parsing & Sorting"
Cohesion: 0.83
Nodes (3): Cli, sort_entries(), SortKey

### Community 7 - "IPv6 Address Formatting"
Cohesion: 0.67
Nodes (3): Ipv6Addr, fmt_addr(), is_zone_scoped()

### Community 8 - "Kernel Routing Ingestion"
Cohesion: 0.67
Nodes (3): BSD routing socket (PF_ROUTE / sysctl NET_RT_DUMP), Kernel routing table, net-route crate

## Knowledge Gaps
- **5 isolated node(s):** `rsnetstat`, `net-route crate`, `Route type classification`, `Sort and reverse options`, `graphify knowledge graph workflow`
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `RouteEntry` connect `RouteEntry Accessors` to `JSON Route Serialization`, `Table Formatting`, `CLI Parsing & Sorting`, `Gateway Field Access`?**
  _High betweenness centrality (0.171) - this node is a cross-community bridge._
- **Why does `main()` connect `Main & Error Handling` to `Table Formatting`, `CLI Parsing & Sorting`?**
  _High betweenness centrality (0.091) - this node is a cross-community bridge._
- **Why does `col_width()` connect `Table Formatting` to `CLI Parsing & Sorting`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **What connects `rsnetstat`, `net-route crate`, `Route type classification` to the rest of the system?**
  _5 weakly-connected nodes found - possible documentation gaps or missing edges._