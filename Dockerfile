FROM rust:1.76.0 as backend

RUN USER=root cargo new --bin sei
WORKDIR /sei

COPY ./apps/api/Cargo.lock ./Cargo.lock
COPY ./apps/api/Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./apps/api/src ./src

ARG DATABASE_URL

COPY .env .env

RUN rm ./target/release/deps/sei*
RUN cargo build --release

FROM node:20-alpine as node_base

RUN apk add --no-cache libc6-compat
RUN apk update
# Set working directory
WORKDIR /sei


FROM node_base AS builder

RUN yarn global add turbo
COPY . .
RUN turbo prune @sei/web --docker

FROM node_base as frontend

COPY .gitignore .gitignore
COPY --from=builder /sei/out/json/ .
COPY --from=builder /sei/out/yarn.lock ./yarn.lock
RUN yarn install

COPY --from=builder /sei/out/full/ .
COPY .env .env
RUN yarn turbo run build --filter=@sei/web

FROM debian:bookworm-slim as runner

WORKDIR /sei

RUN apt-get update && apt install -y openssl ca-certificates
RUN mkdir -p /sei/public

# Don't run production as root
RUN addgroup --system --gid 1001 sei
RUN adduser --system --uid 1001 sei
USER sei

COPY --from=backend /sei/target/release/sei /sei/sei
COPY --from=frontend /sei/apps/web/dist/  /sei/public

CMD ["/sei/sei"]