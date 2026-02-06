# Verification Guide

Follow these steps to verify that LightShark-mini is working correctly.

## Prerequisites
- Docker & Docker Compose installed.

## Step 1: Run the Test Stack
We have prepared a test topology that pairs an Nginx server with LightShark.

1.  Navigate to the project root.
2.  Run the test stack:
    ```bash
    docker-compose -f topologies/docker-compose.test.yml up --build
    ```
    *This will build the `lightshark-mini` image and start Nginx.*

## Step 2: Generate Traffic
Once the stack is up, open a new terminal. The Nginx server is listening on port **8081**, and due to `network_mode: service`, LightShark is listening on port **3001** (mapped via the Nginx container).

1.  **Generate HTTP traffic** by accessing Nginx:
    ```bash
    # Run this multiple times
    curl http://localhost:8081
    curl http://localhost:8081/some-random-path
    ```

## Step 3: Verify Capture
Now queries LightShark's API to see if it caught those packets.

1.  **Check Live Stats**:
    ```bash
    curl http://localhost:3001/api/live
    ```
    **Expected Output**: You should see a JSON object with `connections` containing an entry for your IP -> `127.0.0.1:80`.

2.  **Check History**:
    ```bash
    curl "http://localhost:3001/api/history?limit=5"
    ```
    **Expected Output**: A list of packet metadata objects (Timestamp, SrcIP, DstIP, Protocol: TCP).

## Step 4: Cleanup
Press `Ctrl+C` in the docker-compose terminal, or run:
```bash
docker-compose -f topologies/docker-compose.test.yml down
```
