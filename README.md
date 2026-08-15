# rsnetstat

A high-fidelity, CIDR-complete alternative to macOS `netstat -rn`.

## Why

macOS's `netstat -rn` abbreviates destination networks in a lossy, historical
way: it drops trailing zero octets and hides host-route prefixes. You end up
reading entries like this and guessing at the real prefix:

| `netstat -rn` shows | actual destination |
| ------------------- | ------------------ |
| `default`           | `0.0.0.0/0`        |
| `10`                | `10.0.0.0/8`       |
| `13.64/10`          | `13.64.0.0/10`     |
| `127`               | `127.0.0.0/8`      |

`rsnetstat` reads the **same** kernel routing table — via the BSD routing
socket (`sysctl(NET_RT_DUMP)`) — and prints every destination as an
unambiguous CIDR. It also resolves interface indices to names and restores the
`%zone` suffix on link-local IPv6 addresses (the KAME convention macOS embeds
in the address body).

Each route carries its kernel `rtm_flags` in a `Flags` column, with the same
single-letter mnemonics `netstat` uses (`U`p, `G`ateway, `H`ost, `S`tatic,
IF`S`cope `I`, `g`lobal, …). macOS installs an interface-scoped (`I`) copy of
the default route for every uplink; `rsnetstat` marks the one **up, unscoped**
default per family — the route the kernel actually uses for ordinary traffic,
the same one `route -n get default` returns — as the *best default*.

## Install

From the project root:

```sh
cargo install --path .
```

Or build it directly:

```sh
cargo build --release
./target/release/rsnetstat
```

Reading the routing table is unprivileged — **no `sudo` required**.

## Usage

```
rsnetstat [OPTIONS]
```

| Option            | Description                                                       |
| ----------------- | ---------------------------------------------------------------- |
| `-4`, `--ipv4`    | Show only IPv4 routes                                            |
| `-6`, `--ipv6`    | Show only IPv6 routes                                            |
| `-s`, `--sort <KEY>` | Sort each table by `destination` (default), `prefix`, `gateway`, or `interface` |
| `-r`, `--reverse` | Reverse the sort order (within each family)                      |
| `-j`, `--json`    | Emit JSON instead of a text table                               |
| `-h`, `--help`    | Print help                                                       |
| `-V`, `--version` | Print version                                                    |

With neither `-4` nor `-6`, both families are shown (like `netstat -rn`).

## Examples

Default output — both families, sorted by destination:

```
$ rsnetstat

Internet (IPv4):
Destination         Gateway         Flags   Interface
-----------------------------------------------------
0.0.0.0/0           192.168.1.1     UGScg   en0 *
0.0.0.0/0           192.168.1.1     UGScIg  en0
10.0.0.0/8          10.0.0.1        UGSc    utun3
127.0.0.0/8         link#1          UCS     lo0
127.0.0.1/32        link#1          UH      lo0
192.168.1.0/24      link#11         UCS     en0
192.168.1.42/32     link#11         UHL     en0
224.0.0.0/4         link#11         UmCS    en0
255.255.255.255/32  link#11         UHLb    en0

Internet6 (IPv6):
Destination     Gateway  Flags  Interface
-----------------------------------------
::1/128         link#1   UHL    lo0
fe80::%en0/64   link#11  UCI    en0
ff00::/8        link#1   UmCI   lo0
```

A `*` marks each family's best (primary) default route — the up, unscoped `/0`
(flags with `S`/`g` but no `I`) that the kernel uses for ordinary traffic.

A directly-connected (on-link) route has no next-hop IP; its gateway is shown
as `link#N`, matching `netstat`.

Sort IPv4 routes by prefix length, widest mask first:

```sh
rsnetstat -4 --sort prefix --reverse
```

Machine-readable output for `jq` and friends:

```sh
$ rsnetstat -4 --json
[
  {
    "destination": "0.0.0.0/0",
    "prefix": 0,
    "type": "default",
    "gateway": "192.168.1.1",
    "interface": "en0",
    "ifindex": 11,
    "family": "inet",
    "flags": "UGScg",
    "flags_bits": 1073809411,
    "flags_decoded": [
      { "letter": "U", "name": "UP", "description": "route usable" },
      { "letter": "G", "name": "GATEWAY", "description": "destination is a gateway" },
      { "letter": "S", "name": "STATIC", "description": "manually added" },
      { "letter": "c", "name": "PRCLONING", "description": "protocol requires cloning" },
      { "letter": "g", "name": "GLOBAL", "description": "route to destination of the global internet" }
    ],
    "best_default": true
  }
]
```

JSON keys are always present (absent values become `null`), so consumers see a
stable schema. The `type` field classifies each route as one of `default`,
`unicast host`, `unicast network`, `multicast`, or `broadcast`. Route flags are
exposed three ways: `flags` is the raw `netstat`-style letter string,
`flags_bits` is the numeric `rtm_flags` bitmask, and `flags_decoded` names each
set flag. `best_default` is `true` for the single up, unscoped default route of
each family.

```sh
# the interface of each family's best (primary) default route
rsnetstat --json | jq -r '.[] | select(.best_default) | .interface'
```

## Platform

Built for **macOS**, whose routing table this mirrors. Interface-name and
link-local IPv6 zone handling rely on BSD/macOS conventions.
