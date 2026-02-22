FROM rust:latest

WORKDIR /app

# Copy project
COPY ./controller .

# Build release
RUN cargo build --release

CMD ["./target/release/controller"]