FROM rust:1-alpine3.17 as builder
RUN apk add --no-cache musl-dev pkgconf git

WORKDIR /build
COPY . /build
RUN cargo build --bins --release

FROM scratch

COPY --from=builder /build/target/release/geoip-server /
COPY --from=builder /build/*.mmdb /
CMD ["/geoip-server"]

