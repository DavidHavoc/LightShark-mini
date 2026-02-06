# LightShark-mini

LightShark-mini is a lightweight, high-performance network traffic analyzer written in Rust. It is designed to run as a sidecar container in Docker environments, providing real-time visibility into container-to-container traffic with near-zero resource impact.

LightShark-mini is designed to be as light as possible, and to be able to run in a container with minimal resources. Recommended to run in a container with at least 20MB of memory. Works best with k8s and docker.

## Features

*   **Real-time Traffic Monitoring**: LIVE dashboard data via REST API + WebSocket streaming.
*   **Traffic Filtering**: Filter by port, IP, or protocol via CLI flags.
*   **Low Footprint**: Targets <20MB memory usage using streaming capture and in-memory aggregation.
*   **Persistent History**: Stores packet metadata in SQLite (WAL mode) for historical analysis.
*   **Docker Sidecar Ready**: Seamless integration with `network_mode: service:target`.
*   **Zero-Copy Parsing**: Uses `etherparse` for efficient packet inspection.
*   **Config File Support**: YAML configuration for complex setups.
*   **Connection Cleanup**: Auto-removes stale connections from memory.

## Prerequisites

*   **Rust**: Stable toolchain (1.77+ recommended).
*   **libpcap**: Required for packet capture (dev headers for building, runtime lib for execution).
    *   *Debian/Ubuntu*: `sudo apt-get install libpcap-dev`
    *   *Windows*: [Npcap](https://npcap.com/) (install in WinPcap API-compatible mode).

## Quick Start (Local)

1.  **Build**:
    ```bash
    cargo build --release
    ```

2.  **Run**:
    ```bash
    # Run with admin privileges (required for capturing)
    ./target/release/lightshark-mini --interface eth0
    ```

3.  **Check Health**:
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
| `-c, --config` | Path to YAML config file | - |
| `-q, --quiet` | Suppress non-error logs | `false` |

## Docker Usage

See [HOW_TO_USE.md](HOW_TO_USE.md) for detailed instructions on deploying as a sidecar.

## Kubernetes Usage

LightShark-mini works natively as a Kubernetes sidecar container. See [HOW_TO_USE.md](HOW_TO_USE.md#kubernetes-deployment) for an example.

## License

Distributed under the GNU General Public License v3.0. See `LICENSE` for more information.
