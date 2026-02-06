# LightShark-mini

LightShark-mini is a lightweight, high-performance network traffic analyzer written in Rust. It is designed to run as a sidecar container in Docker environments, providing real-time visibility into container-to-container traffic with near-zero resource impact.

## Features

*   **Real-time Traffic Monitoring**: LIVE dashboard data via REST API.
*   **Low Footprint**: Targets <20MB memory usage using streaming capture and in-memory aggregation.
*   **Persistent History**: Stores packet metadata in SQLite (WAL mode) for historical analysis.
*   **Docker Sidecar Ready**: seamless integration with `network_mode: service:target`.
*   **Zero-Copy Parsing**: Uses `etherparse` for efficient packet inspection.

## Prerequisites

*   **Rust**: Stable toolchain (1.70+ recommended).
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

## Docker Usage

See [HOW_TO_USE.md](HOW_TO_USE.md) for detailed instructions on deploying as a sidecar.

## License

Distributed under the GNU General Public License v3.0. See `LICENSE` for more information.
