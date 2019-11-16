FROM rust:latest as cargo-build

RUN rustup default nightly
RUN rm -f target/release/deps/unbase*
RUN cargo build
