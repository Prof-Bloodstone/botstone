FROM rust:1.46 as builder

WORKDIR /botstone

ENV USER=root
# create a new empty shell project
RUN cargo init --bin

# copy over manifests
COPY Cargo.* ./

# cache dependencies
RUN cargo build

RUN rm -rf src/ target/debug/deps/botstone*

# copy source tree
COPY ./src ./src

RUN cargo build

FROM debian:buster-slim

COPY --from=builder /botstone/target/debug/botstone /botstone

CMD ["/botstone"]
