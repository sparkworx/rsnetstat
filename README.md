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
socket (`PF_ROUTE` / `sysctl(NET_RT_DUMP)`, wrapped by the
[`net-route`](https://crates.io/crates/net-route) crate) — and prints every
destination as an unambiguous CIDR. It also resolves interface indices to
names and restores the `%zone` suffix on link-local IPv6 addresses (the KAME
convention macOS embeds in the address body).

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
Destination         Gateway         Interface
---------------------------------------------
0.0.0.0/0           192.168.1.1     en0
10.0.0.0/8          10.0.0.1        utun3
127.0.0.0/8         link#1          lo0
127.0.0.1/32        link#1          lo0
192.168.1.0/24      link#11         en0
192.168.1.42/32     link#11         en0
224.0.0.0/4         link#11         en0
255.255.255.255/32  link#11         en0

Internet6 (IPv6):
Destination     Gateway  Interface
----------------------------------
::1/128         link#1   lo0
fe80::%en0/64   link#11  en0
ff00::/8        link#1   lo0
```

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
    "family": "inet"
  }
]
```

JSON keys are always present (absent values become `null`), so consumers see a
stable schema. The `type` field classifies each route as one of `default`,
`unicast host`, `unicast network`, `multicast`, or `broadcast`.

```sh
# every interface carrying a default route
rsnetstat --json | jq -r '.[] | select(.type == "default") | .interface'
```

## Platform

Built for **macOS**, whose routing table this mirrors. Interface-name and
link-local IPv6 zone handling rely on BSD/macOS conventions.
