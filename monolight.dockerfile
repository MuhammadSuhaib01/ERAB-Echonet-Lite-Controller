FROM rust:latest

WORKDIR /app

COPY ./monolight .

RUN cargo build --release

CMD ["./target/release/monolight"]