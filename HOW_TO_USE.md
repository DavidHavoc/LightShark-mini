# How to Use LightShark-mini

This guide covers the deployment of LightShark-mini as a Docker sidecar, Kubernetes sidecar, and how to consume its API.

## 1. Docker Sidecar Deployment

The primary use case for LightShark-mini is to monitor another container's network traffic. This is achieved using Docker's `network_mode: service:<container_name>`.

### Step 1: Build the Image

```bash
docker build -t lightshark-mini .
```

### Step 2: Configure `docker-compose.yml`

Add `lightshark` to your services, attached to the target you want to monitor.

```yaml
services:
  # The service you want to monitor
  my-api:
    image: my-existing-api:latest
    container_name: production-api
    ports:
      - "8080:8080" # External access to your API
      - "3000:3000" # LightShark API (mapped via the target container!)

  # The traffic analyzer
  lightshark:
    image: lightshark-mini
    container_name: lightshark-sidecar
    network_mode: "service:my-api"  # Share network namespace with target
    cap_add:
      - NET_ADMIN  # Required for packet capture
    depends_on:
      - my-api
    volumes:
      - ./traffic_data:/data # Persist SQLite DB
    command: ["/app/lightshark-mini", "--db-path", "/data/traffic.db"]
```

### Important Networking Note
When using `network_mode: service:my-api`, LightShark shares the **localhost** and **IP address** of `my-api`.
*   If `lightshark-mini` listens on port `3000`, it opens port `3000` on the `my-api` container's network interface.
*   To access the LightShark API from your host machine, you must map the port on the **TARGET** container.

## 2. Kubernetes Deployment

In Kubernetes, containers in the same Pod automatically share the network namespace. This makes sidecar deployment even easier than in Docker.

### Example Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-with-lightshark
spec:
  replicas: 1
  selector:
    matchLabels:
      app: my-app
  template:
    metadata:
      labels:
        app: my-app
    spec:
      containers:
        # 1. Your Main Application
        - name: my-api
          image: nginx:alpine 
          ports:
            - containerPort: 80

        # 2. LightShark Sidecar
        - name: lightshark
          image: lightshark-mini:latest
          imagePullPolicy: IfNotPresent
          securityContext:
            capabilities:
              add: ["NET_ADMIN"] # Required for packet capture
          args:
             - "--interface"
             - "eth0" # eth0 is shared with my-api
             - "--quiet"
          ports:
            - containerPort: 3000
              name: lightshark-api
```

### 3. Injecting into Running Pods (Ephemeral Containers)

If you have a pod *already running* in production and you need to inspect its traffic without restarting it, you can "inject" LightShark using Kubernetes Ephemeral Containers.

**Prerequisites:**
*   Kubernetes v1.23+
*   `kubectl` installed locally

**Command:**
```bash
# Syntax: kubectl debug -it <POD_NAME> --image=<IMAGE> --target=<TARGET_CONTAINER>
kubectl debug -it my-app-pod-xyz --image=lightshark-mini --target=my-app-container -- /app/lightshark-mini --interface eth0
```

**What happens:**
1.  K8s spins up a new container *inside* the existing Pod.
2.  It shares the network namespace (so it sees `eth0`).
3.  LightShark starts capturing immediately.
4.  You can then `curl localhost:3000/api/live` from *inside* that debug console (if you have curl), or port-forward from your host:

**Accessing the API of the Injected Container:**
Since the debug container shares the network, you can port-forward to the Pod as usual:

```bash
kubectl port-forward my-app-pod-xyz 3000:3000
# Then on your machine:
curl http://localhost:3000/api/live
```

## 3. API Reference

LightShark-mini exposes a JSON REST API on port `3000` (default).

### Health Check
**GET** `/api/health`

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

```json
{
  "connections": [
    {
      "connection": "172.18.0.3:5432 -> 172.18.0.2:49152",
      "stats": { "bytes_sent": 2048, "packets_count": 32 }
    }
  ],
  "total_packets": 100,
  "total_bytes": 5000
}
```

### Traffic History
**GET** `/api/history?limit=5`

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

### Stats (NEW)
**GET** `/api/stats`

```json
{
  "uptime_seconds": 120,
  "total_packets": 5000,
  "total_bytes": 1234567,
  "active_connections": 15,
  "packets_per_second": 41.6,
  "bytes_per_second": 10288
}
```

### WebSocket Stream (NEW)
**GET** `/api/stream` (WebSocket)

Pushes stats every 1 second. Connect with:
```bash
websocat ws://localhost:3000/api/stream
```

## 4. Configuration

### CLI Arguments

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --interface` | Network interface | Auto-detect |
| `-p, --port` | API port | `3000` |
| `--db-path` | SQLite path | `traffic.db` |
| `--filter-port` | Filter by port | - |
| `--filter-ip` | Filter by IP | - |
| `--filter-protocol` | Filter by protocol (TCP/UDP) | - |
| `--connection-timeout` | Stale cleanup (sec) | `60` |
| `--data-retention` | Delete packets older than (sec) | disabled |
| `-c, --config` | YAML config file | - |
| `-q, --quiet` | Quiet mode | `false` |

### YAML Config File

Instead of CLI flags, you can use a YAML config:

```yaml
# config.yaml
interface: eth0
port: 3000
db_path: /data/traffic.db
filter_port: 80
connection_timeout: 120
data_retention_seconds: 86400  # Delete data older than 24 hours
quiet: true
```

Run with:
```bash
./lightshark-mini --config config.yaml
```
