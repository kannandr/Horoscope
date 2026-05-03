FROM rust:bookworm AS build
WORKDIR /src
COPY rust/Cargo.toml ./Cargo.toml
COPY rust/Cargo.lock ./Cargo.lock
COPY rust/crates ./crates
RUN cargo build --release -p horoscope-mcp

FROM debian:bookworm-slim
RUN useradd -r -u 10001 appuser && apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/horoscope-mcp /usr/local/bin/horoscope-mcp
USER appuser
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/horoscope-mcp"]
