FROM rustlang/rust:nightly-alpine AS build

WORKDIR /usr/src/app
COPY . .
RUN apk add --no-cach musl-dev && cargo install --path .

# FINAL

FROM alpine:latest
ENV ADDR=127.0.0.1
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080
COPY --from=build /usr/local/cargo/bin/actix-trust-service /usr/local/bin/actix-trust-service
ENTRYPOINT [ "actix-trust-service" ]