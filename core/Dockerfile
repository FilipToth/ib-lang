FROM rust:1.81

WORKDIR /usr/src/ibcore

COPY . .

RUN cargo build --release --manifest-path=ib_server/Cargo.toml

EXPOSE 8080

CMD ["./ib_server/target/release/ib_server"]