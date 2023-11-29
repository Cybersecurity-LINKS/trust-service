# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rustlang/rust:nightly-alpine AS build
WORKDIR /usr/src/app
COPY . .
COPY .env .env
COPY .mongo.env .mongo.env
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo install --path ./actix-server


# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:latest
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
COPY --from=build /usr/local/cargo/bin/actix-trust-service /usr/local/bin/actix-trust-service
COPY --from=build /usr/src/app/.env /.env
COPY --from=build /usr/src/app/.mongo.env /.mongo.env
EXPOSE 8081
ENTRYPOINT [ "actix-trust-service" ] 