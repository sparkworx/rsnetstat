//! rsnetstat — a high-fidelity, CIDR-complete alternative to macOS `netstat -rn`.
//!
//! macOS's `netstat -rn` abbreviates destinations in a lossy, historical way:
//! it drops trailing zero octets and hides host-route prefixes, so you see
//! `default`, `10`, or `13.64/10` instead of `0.0.0.0/0`, `10.0.0.0/8`, or
//! `13.64.0.0/10`. This tool reads the same kernel routing table (via the BSD
//! routing socket, `PF_ROUTE` / `sysctl(NET_RT_DUMP)`, which the `net-route`
//! crate wraps for us) and prints every destination as an unambiguous CIDR.
//!
//! Flags: `-4` / `-6` filter by family, `--sort` and `--reverse` order the
//! output, and `--json` emits machine-readable JSON. See `--help`.

use clap::{Parser, ValueEnum};
use net_route::Handle;
use serde::Serialize;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv6Addr};

/// High-fidelity, CIDR-complete alternative to macOS `netstat -rn`.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Show only IPv4 routes
    #[arg(short = '4', long)]
    ipv4: bool,

    /// Show only IPv6 routes
    #[arg(short = '6', long)]
    ipv6: bool,

    /// Column to sort each table by
    #[arg(short, long, value_enum, default_value = "destination")]
    sort: SortKey,

    /// Reverse the sort order (within each family)
    #[arg(short, long)]
    reverse: bool,

    /// Emit JSON instead of a text table
    #[arg(short, long)]
    json: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum SortKey {
    /// Destination network address, then prefix length (default)
    Destination,
    /// Prefix length (mask width)
    Prefix,
    /// Next-hop gateway address
    Gateway,
    /// Outgoing interface name
    Interface,
}

/// One route, kept structured so sorting, filtering, table rendering, and JSON
/// all derive from the same source of truth.
struct RouteEntry {
    destination: IpAddr,
    prefix: u8,
    gateway: Option<IpAddr>,
    ifindex: Option<u32>,
    /// Outgoing interface name, resolved once from `ifindex`.
    interface: Option<String>,
}

/// JSON shape for `--json`. Keys are always present (absent values become
/// `null`) so consumers like `jq` see a stable schema.
#[derive(Serialize)]
struct JsonRoute {
    destination: String,
    prefix: u8,
    /// Classification of the destination (see `RouteEntry::route_type`).
    #[serde(rename = "type")]
    kind: &'static str,
    gateway: Option<String>,
    interface: Option<String>,
    ifindex: Option<u32>,
    family: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // With neither -4 nor -6, show both families (like `netstat -rn`).
    let (show_v4, show_v6) = match (cli.ipv4, cli.ipv6) {
        (false, false) => (true, true),
        choice => choice,
    };

    // `Handle::new()` opens a routing socket; `list()` dumps the whole table.
    // Neither requires root — reading the routing table is unprivileged.
    let handle = Handle::new()?;
    let routes = handle.list().await?;

    // Partition by family so each table stays grouped, applying the filter.
    let mut v4: Vec<RouteEntry> = Vec::new();
    let mut v6: Vec<RouteEntry> = Vec::new();
    for r in routes {
        let entry = RouteEntry {
            destination: r.destination,
            prefix: r.prefix,
            gateway: r.gateway,
            ifindex: r.ifindex,
            interface: r.ifindex.map(if_name),
        };
        if entry.destination.is_ipv4() {
            if show_v4 {
                v4.push(entry);
            }
        } else if show_v6 {
            v6.push(entry);
        }
    }

    sort_entries(&mut v4, cli.sort, cli.reverse);
    sort_entries(&mut v6, cli.sort, cli.reverse);

    if cli.json {
        let out: Vec<JsonRoute> = v4.iter().chain(v6.iter()).map(RouteEntry::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_section("Internet (IPv4):", &v4);
        print_section("Internet6 (IPv6):", &v6);
    }

    Ok(())
}

/// Sort one family's routes in place by the chosen key, then optionally reverse.
/// Every key falls back to (destination, prefix) so output is deterministic.
fn sort_entries(routes: &mut [RouteEntry], key: SortKey, reverse: bool) {
    match key {
        SortKey::Destination => routes.sort_by_key(|e| (e.destination, e.prefix)),
        SortKey::Prefix => routes.sort_by_key(|e| (e.prefix, e.destination)),
        SortKey::Gateway => {
            routes.sort_by_key(|e| (gateway_sort_key(e), e.destination, e.prefix))
        }
        SortKey::Interface => routes.sort_by(|a, b| {
            a.interface
                .cmp(&b.interface)
                .then_with(|| (a.destination, a.prefix).cmp(&(b.destination, b.prefix)))
        }),
    }
    if reverse {
        routes.reverse();
    }
}

/// Sort key for the gateway column: real next-hop IPs first (ordered
/// numerically), then on-link routes (no gateway) grouped after them.
fn gateway_sort_key(e: &RouteEntry) -> (u8, Option<IpAddr>) {
    match e.gateway {
        Some(ip) => (0, Some(ip)),
        None => (1, None),
    }
}

impl RouteEntry {
    fn family(&self) -> &'static str {
        if self.destination.is_ipv4() {
            "inet"
        } else {
            "inet6"
        }
    }

    /// Classify the route by its destination, using only std `IpAddr`
    /// inspection (no extra crates). Precedence matters: a `/0` is always the
    /// default route, and multicast/broadcast destinations are caught before
    /// the prefix-based host/network split so they aren't mislabelled as
    /// "unicast" (e.g. `255.255.255.255/32` is broadcast, not a unicast host).
    fn route_type(&self) -> &'static str {
        if self.prefix == 0 {
            return "default";
        }
        match self.destination {
            IpAddr::V4(v4) => {
                if v4.is_multicast() {
                    "multicast" // 224.0.0.0/4
                } else if v4.is_broadcast() {
                    "broadcast" // 255.255.255.255
                } else if self.prefix == 32 {
                    "unicast host"
                } else {
                    "unicast network"
                }
            }
            IpAddr::V6(v6) => {
                if v6.is_multicast() {
                    "multicast" // ff00::/8
                } else if self.prefix == 128 {
                    "unicast host"
                } else {
                    "unicast network"
                }
            }
        }
    }

    /// Destination as a full CIDR, e.g. `10.0.0.0/8` or `fe80::%utun6/64`.
    fn cidr(&self) -> String {
        format!("{}/{}", fmt_addr(self.destination, self.ifindex), self.prefix)
    }

    /// Next-hop gateway as a string, or `None` for directly-connected routes.
    fn gateway_addr(&self) -> Option<String> {
        self.gateway.map(|gw| fmt_addr(gw, self.ifindex))
    }

    /// Gateway column for the table: the next hop, or `link#N` for an on-link
    /// route (the kernel reaches the destination over the interface itself,
    /// with no next-hop IP — netstat renders these the same way).
    fn gateway_cell(&self) -> String {
        match self.gateway_addr() {
            Some(gw) => gw,
            None => match self.ifindex {
                Some(idx) => format!("link#{idx}"),
                None => "-".to_string(),
            },
        }
    }

    fn interface_cell(&self) -> &str {
        self.interface.as_deref().unwrap_or("-")
    }

    fn to_json(&self) -> JsonRoute {
        JsonRoute {
            destination: self.cidr(),
            prefix: self.prefix,
            kind: self.route_type(),
            gateway: self.gateway_addr(),
            interface: self.interface.clone(),
            ifindex: self.ifindex,
            family: self.family(),
        }
    }
}

/// Render an address. For zone-scoped IPv6 — unicast link-local (`fe80::/10`)
/// and interface/link-local multicast (`ff01::`, `ff02::`) — append a textual
/// `%zone` and strip the zone id that the macOS kernel embeds in the second
/// 16-bit group (the KAME convention). This mirrors `netstat -rn`, which turns
/// the kernel's `fe80:13::` into `fe80::%en13`.
fn fmt_addr(ip: IpAddr, fallback_ifindex: Option<u32>) -> String {
    let IpAddr::V6(v6) = ip else {
        return ip.to_string();
    };
    if !is_zone_scoped(&v6) {
        return v6.to_string();
    }

    let mut segs = v6.segments();
    let embedded = segs[1];
    segs[1] = 0; // move the zone id out of the address body
    let clean = Ipv6Addr::from(segs);

    // Prefer the embedded zone id — it's what netstat shows, and it can differ
    // from the route's interface (e.g. an `fe80::…%llw0` host route installed
    // on lo0). Fall back to the route's interface when the zone was already
    // stripped upstream, as net-route does for gateways.
    let zone = if embedded != 0 {
        Some(embedded as u32)
    } else {
        fallback_ifindex
    };
    match zone {
        Some(idx) => format!("{clean}%{}", if_name(idx)),
        None => clean.to_string(),
    }
}

/// True for IPv6 addresses that carry a zone id: unicast link-local
/// (`fe80::/10`) and interface- or link-local scoped multicast.
fn is_zone_scoped(ip: &Ipv6Addr) -> bool {
    let segs = ip.segments();
    let is_unicast_ll = (segs[0] & 0xffc0) == 0xfe80;
    let is_multicast = (segs[0] & 0xff00) == 0xff00;
    let multicast_scope = segs[0] & 0x000f; // 1 = interface-local, 2 = link-local
    is_unicast_ll || (is_multicast && (multicast_scope == 1 || multicast_scope == 2))
}

/// Map a kernel interface index to its name (e.g. 12 -> "en12") via the libc
/// `if_indextoname(3)` call. Falls back to `if#N` if the index can't be
/// resolved (e.g. the interface vanished between the dump and now).
fn if_name(ifindex: u32) -> String {
    // `if_indextoname` needs a caller-provided buffer of at least IF_NAMESIZE.
    let mut buf = [0u8; libc::IF_NAMESIZE];
    // SAFETY: `buf` is IF_NAMESIZE bytes as required; on success the C string
    // is NUL-terminated within it, and we read it back before `buf` is dropped.
    let ptr = unsafe { libc::if_indextoname(ifindex, buf.as_mut_ptr() as *mut libc::c_char) };
    if ptr.is_null() {
        format!("if#{ifindex}")
    } else {
        // SAFETY: non-null `ptr` points at the NUL-terminated name in `buf`.
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

/// Print one address-family section as an aligned table. Column widths are
/// computed from the content so long IPv6 CIDRs don't break alignment.
fn print_section(title: &str, rows: &[RouteEntry]) {
    if rows.is_empty() {
        return;
    }

    // Pre-render the variable-width columns once, so we size and print from
    // the same strings rather than recomputing them.
    let cidrs: Vec<String> = rows.iter().map(RouteEntry::cidr).collect();
    let gateways: Vec<String> = rows.iter().map(RouteEntry::gateway_cell).collect();

    let dest_w = col_width(cidrs.iter().map(String::len), "Destination");
    let gw_w = col_width(gateways.iter().map(String::len), "Gateway");

    println!("\n{title}");
    println!("{:<dest_w$}  {:<gw_w$}  Interface", "Destination", "Gateway");
    println!("{}", "-".repeat(dest_w + gw_w + "Interface".len() + 4));
    for ((cidr, gateway), row) in cidrs.iter().zip(&gateways).zip(rows) {
        println!(
            "{:<dest_w$}  {:<gw_w$}  {}",
            cidr,
            gateway,
            row.interface_cell()
        );
    }
}

/// Width of a column: the widest value, but never narrower than its header.
fn col_width(value_lens: impl Iterator<Item = usize>, header: &str) -> usize {
    value_lens.max().unwrap_or(0).max(header.len())
}
