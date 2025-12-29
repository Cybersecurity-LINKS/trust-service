# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rust:alpine3.20 AS build
WORKDIR /usr/src/app
COPY . .
# COPY .env .env
# COPY .mongo.env .mongo.env
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cd abigen  \ \
    && cargo run -- --contract AssetFactory --abi-source "../smart-contracts/AssetFactory.json" \
    && cargo run -- --contract Asset --abi-source "../smart-contracts/Asset.json" \
    && cd ..
RUN cargo install --path ./actix-server

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:3.20
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
COPY --from=build /usr/local/cargo/bin/actix-trust-service /usr/local/bin/actix-trust-service
COPY --from=build /usr/src/app/actix-server/.env /.env
COPY --from=build /usr/src/app/actix-server/.mongo.env /.mongo.env
EXPOSE 8081
ENTRYPOINT [ "actix-trust-service" ] 