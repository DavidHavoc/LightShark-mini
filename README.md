# LightShark-mini

A lightweight, high-performance network traffic analyzer written in Rust. Designed to run as a sidecar container in Docker and Kubernetes environments, providing real-time visibility into container-to-container traffic with near-zero resource impact.

## Highlights

| Metric | Value |
|--------|-------|
| **Release Binary** | 5.3 MB |
| **Stripped Binary** | 4.5 MB |
| **Memory Usage** | ~10-15 MB |
| **Startup Time** | < 1 second |

## Features

- **Real-time Monitoring** - Live dashboard via REST API + WebSocket streaming
- **Traffic Filtering** - Filter by port, IP, or protocol
- **Persistent History** - SQLite storage with configurable data retention
- **Low Footprint** - Targets <20MB memory using streaming capture
- **Sidecar Ready** - Native Docker and Kubernetes integration
- **Zero-Copy Parsing** - Efficient packet inspection with `etherparse`
- **Config File Support** - YAML configuration for complex setups

## Quick Start

### Build

```bash
cargo build --release
```

### Run

```bash
# Requires admin privileges for packet capture
sudo ./target/release/lightshark-mini --interface eth0
```

### Verify

```bash
curl http://localhost:3000/api/health
```

## CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --interface` | Network interface to capture | Auto-detect |
| `-p, --port` | API server port | `3000` |
| `--db-path` | SQLite database path | `traffic.db` |
| `--filter-port` | Only capture traffic on this port | - |
| `--filter-ip` | Only capture traffic to/from this IP | - |
| `--filter-protocol` | Only capture TCP or UDP | - |
| `--connection-timeout` | Stale connection cleanup (seconds) | `60` |
| `--data-retention` | Auto-delete packets older than (seconds) | disabled |
| `-c, --config` | Path to YAML config file | - |
| `-q, --quiet` | Suppress non-error logs | `false` |

## Kubernetes Deployment

LightShark-mini is optimized for Kubernetes sidecar containers:

- **Sidecar-native** - Shares Pod network namespace automatically
- **Small binary** - Fast container startup
- **Low memory** - Well under 20 MB limit
- **Configurable retention** - Prevents storage growth
- **No external dependencies** - SQLite is bundled

### Recommended Resource Limits

```yaml
resources:
  requests:
    memory: "16Mi"
    cpu: "10m"
  limits:
    memory: "32Mi"
    cpu: "100m"
```

See [HOW_TO_USE.md](HOW_TO_USE.md#kubernetes-deployment) for full deployment examples.

## Docker Usage

Deploy as a sidecar using `network_mode: service:<target>`. See [HOW_TO_USE.md](HOW_TO_USE.md) for detailed instructions.

## Prerequisites

- **Rust**: Stable toolchain (1.77+)
- **libpcap**: Required for packet capture
  - *Debian/Ubuntu*: `sudo apt-get install libpcap-dev`
  - *macOS*: Pre-installed
  - *Windows*: [Npcap](https://npcap.com/) (WinPcap API-compatible mode)

## License

Distributed under the GNU General Public License v3.0. See `LICENSE` for more information.
