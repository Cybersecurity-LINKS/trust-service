# FROM rustlang/rust:nightly-alpine AS build
# # ENV PKG_CONFIG_ALLOW_CROSS=1
# WORKDIR /usr/src/app
# COPY . .
# RUN cd actix-server && apk add --no-cach make musl-dev clang && RUSTFLAGS="-C target-feature=-crt-static" cargo install --path .

# FROM rust:latest AS build
# FROM ekidd/rust-musl-builder AS build
FROM rustlang/rust:nightly-alpine AS build

# RUN apt-get update
# RUN apt-get install musl-tools clang libclang-dev -y && ln -s /bin/g++ /bin/musl-g++
# RUN rustup target add x86_64-unknown-linux-musl && rustup component add rustfmt

WORKDIR /usr/src/app
COPY . .
# RUN RUSTFLAGS=-Clinker=musl-gcc cargo install --path ./actix-server --target=x86_64-unknown-linux-musl
RUN  apk add --no-cach make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc
# ENV CXX=clang++
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN RUSTFLAGS="-C target-feature=-crt-static" && cargo install --path ./actix-server


# FINAL

FROM alpine:latest
ENV ADDR=127.0.0.1
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080
COPY --from=build /usr/local/cargo/bin/actix-trust-service /usr/local/bin/actix-trust-service
ENTRYPOINT [ "actix-trust-service" ]