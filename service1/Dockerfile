# ---- Builder Stage ----

FROM rust:1.77 as builder
LABEL authors="vighn"

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./

RUN cargo build --release --locked --target-dir /usr/src/target
RUN rm -rf /usr/src/target/release/deps/sensor_publisher*

COPY src ./src

RUN cargo build --release --locked --target-dir /usr/src/target

# ---- Runtime Stage ----

FROM debian:stable-slim

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/sensor_publisher /usr/local/bin/sensor_publisher

CMD ["sensor_publisher"]