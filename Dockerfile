FROM rust:1.85 as builder

WORKDIR /app
COPY . .
# Install libpcap dev headers for building
RUN apt-get update && apt-get install -y libpcap-dev
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
# Install libpcap runtime
RUN apt-get update && apt-get install -y libpcap0.8 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lightshark-mini /app/lightshark-mini

# Expose API port
EXPOSE 3000

# Set entrypoint
CMD ["/app/lightshark-mini"]
