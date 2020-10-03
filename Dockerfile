FROM rust:1.46 as builder

WORKDIR /botstone
ENV USER=root

# create a new empty shell project
RUN cargo init --bin
# copy over manifests
COPY Cargo.* ./
# Temporarily remove build script
RUN sed -i '/^build/d' Cargo.toml
# cache dependencies
RUN cargo build
RUN rm -rf src/ target/debug/deps/botstone*

ENV SQLX_OFFLINE=true
# copy everything, since we use git status
COPY ./ ./
RUN cargo build

FROM debian:buster-slim
RUN apt-get update \
      && apt-get install -y --no-install-recommends \
        libssl1.1 \
      && apt-get clean \
      && rm -rf /var/lib/apt/lists/*

COPY --from=builder /botstone/target/debug/botstone /botstone
CMD ["/botstone"]
