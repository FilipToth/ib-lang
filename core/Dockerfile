# syntax=docker/dockerfile:1

# ---------- build stage ----------
FROM rust:1.81-bookworm AS builder

# rusqlite links against the system SQLite rather than bundling it
RUN apt-get update && \
    apt-get install -y --no-install-recommends libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ibcore

# Compile the dependency graph against the manifests alone. Docker reuses this
# layer until Cargo.toml or Cargo.lock changes, so editing source no longer
# rebuilds every crate from scratch.
COPY Cargo.toml Cargo.lock ./
COPY ibc/Cargo.toml ibc/Cargo.toml
COPY ib_server/Cargo.toml ib_server/Cargo.toml
RUN mkdir -p ibc/src ib_server/src && \
    : > ibc/src/lib.rs && \
    echo 'fn main() {}' > ibc/src/main.rs && \
    echo 'fn main() {}' > ib_server/src/main.rs && \
    cargo build --release --locked && \
    rm -rf ibc/src ib_server/src

# Real sources. Cargo decides staleness by mtime, so touch our own crates to
# make sure they rebuild over the placeholder artifacts.
COPY . .
RUN touch ibc/src/lib.rs ibc/src/main.rs ib_server/src/main.rs && \
    cargo build --release --locked -p ib_server

# ---------- runtime stage ----------
FROM debian:bookworm-slim AS runtime

# libsqlite3-0 for rusqlite; libssl3 + ca-certificates for reqwest's TLS when
# ib_server calls auth-server.
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libsqlite3-0 \
        libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Must stay /usr/src/ibcore: the server creates ./data relative to its working
# directory, and docker-compose mounts the ib-data volume at that path.
WORKDIR /usr/src/ibcore

COPY --from=builder /usr/src/ibcore/target/release/ib_server /usr/local/bin/ib_server

EXPOSE 8080

CMD ["ib_server"]
