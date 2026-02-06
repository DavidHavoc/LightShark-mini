# How to Use LightShark-mini

This guide covers the deployment of LightShark-mini as a Docker sidecar and how to consume its API.

## 1. Docker Sidecar Deployment

The primary use case for LightShark-mini is to monitor another container's network traffic. This is achieved using Docker's `network_mode: service:<container_name>`.

### Step 1: Build the Image

```bash
docker build -t lightshark-mini .
```

### Step 2: Configure `docker-compose.yml`

Add `lightshark` to your services, attached to the target you want to monitor.

```yaml
version: '3.8'

services:
  # The service you want to monitor
  my-api:
    image: my-existing-api:latest
    container_name: production-api
    ports:
      - "8080:8080" # External access to your API

  # The traffic analyzer
  lightshark:
    image: lightshark-mini
    container_name: lightshark-sidecar
    # CRITICAL: Share network namespace with target
    network_mode: "service:my-api" 
    depends_on:
      - my-api
    volumes:
      - ./traffic_data:/data # Persist SQLite DB
    command: ["/app/lightshark-mini", "--db-path", "/data/traffic.db"]
```

### Important Networking Note
When using `network_mode: service:my-api`, LightShark shares the **localhost** and **IP address** of `my-api`.
*   If `lightshark-mini` listens on port `3000`, it opens port `3000` on the `my-api` container's network interface.
*   To access the LightShark API from your host machine, you must map the port on the **TARGET** container (or use a shared internal network if accessing from another container).

**Revised Example for External Access:**

```yaml
services:
  my-api:
    image: ...
    ports:
      - "8080:8080" # Your App
      - "3000:3000" # LightShark API (mapped via the target container!)
```

## 2. API Reference

LightShark-mini exposes a JSON REST API on port `3000` (default).

### Health Check
**GET** `/api/health`

Checks if the sniffer is running and database is accessible.

```json
{
  "status": "ok",
  "active_connections": 12,
  "total_packets": 15430
}
```

### Live Traffic
**GET** `/api/live`

Returns aggregated statistics for active connections (Top 50 by packet count).

**Response:**
```json
{
  "connections": [
    {
      "connection": "172.18.0.3:5432 -> 172.18.0.2:49152",
      "stats": {
        "bytes_sent": 2048,
        "bytes_received": 0,
        "packets_count": 32
      }
    }
  ],
  "total_packets": 100,
  "total_bytes": 5000
}
```

### Traffic History
**GET** `/api/history`

Query parameters:
*   `limit`: Number of records to return (default: 100, max: 1000).

**Example:** `/api/history?limit=5`

**Response:**
```json
[
  {
    "timestamp": 1678886400123,
    "src_ip": "10.0.0.5",
    "dst_ip": "142.250.1.1",
    "src_port": 45678,
    "dst_port": 443,
    "protocol": "TCP",
    "length": 1500
  }
]
```

## 3. Configuration

The binary accepts the following command-line arguments:

*   `--interface <NAME>`: Specific interface to bind (e.g., `eth0`). If omitted, it attempts to auto-discover the default interface.
*   `--port <PORT>`: Port to serve the API (default: `3000`).
*   `--db-path <PATH>`: Location of the SQLite database file (default: `traffic.db`).
