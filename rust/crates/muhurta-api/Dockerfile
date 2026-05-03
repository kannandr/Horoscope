FROM rust:bookworm AS build
WORKDIR /src
COPY rust/Cargo.toml ./Cargo.toml
COPY rust/Cargo.lock ./Cargo.lock
COPY rust/crates ./crates
RUN cargo build --release -p muhurta-api

FROM debian:bookworm-slim
RUN useradd -r -u 10001 appuser && apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/muhurta-api /usr/local/bin/muhurta-api
USER appuser
ENV BIND_ADDR=0.0.0.0:8090
# PANCHANG_MCP_BASE_URL=https://<panchang-mcp host>     (selects wire-mode MCP client)
# MCP_SHARED_SECRET=<bearer secret for /mcp>            (forwarded as Authorization header)
EXPOSE 8090
ENTRYPOINT ["/usr/local/bin/muhurta-api"]
