FROM rust:1.63 as builder
RUN apt-get update && apt-get -y install libclang-dev
WORKDIR .
COPY . .
RUN cargo build --release
RUN mkdir -p /build-out
RUN cp target/release/discord-word-tracker /build-out/

FROM debian:bullseye-slim
RUN apt-get update && apt-get -y install ca-certificates libssl-dev libc6-dev
COPY --from=builder /build-out/discord-word-tracker /
CMD /discord-word-tracker