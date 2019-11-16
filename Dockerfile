FROM rust:latest as cargo-build

RUN rustup default nightly
RUN cargo build
