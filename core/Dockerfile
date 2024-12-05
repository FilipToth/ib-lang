FROM rust:1.81

RUN apt-get update && \
    apt-get install -y libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ibcore

COPY . .

RUN cargo build --release --manifest-path=ib_server/Cargo.toml

EXPOSE 8080

CMD ["./target/release/ib_server"]