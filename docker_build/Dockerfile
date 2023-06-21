FROM rustlang/rust:nightly AS rust-build

RUN mkdir /app
WORKDIR /app

# Git clone
RUN git clone --recursive https://github.com/MonarchDevelopment/SquireCore 
WORKDIR /app/SquireCore

# Tools setup
RUN cargo install --locked trunk
RUN cargo install --locked wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

# Build
RUN cargo build 
RUN cargo build --manifest-path squire_web/Cargo.toml --target wasm32-unknown-unknown
RUN trunk build squire_web/index.html
