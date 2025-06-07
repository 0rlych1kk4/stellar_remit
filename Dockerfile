FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /app/target/release/stellar_remit /usr/local/bin/
CMD ["stellar_remit"]

