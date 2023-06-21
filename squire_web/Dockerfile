# Compile frontend
FROM rustlang/rust:nightly AS rust-build

RUN mkdir /app
WORKDIR /app

# Git clone
RUN git clone --recursive https://github.com/MonarchDevelopment/SquireCore 
WORKDIR /app/SquireCore
RUN git switch wip-web-frontend

# Tools setup
RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v0.16.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-
RUN cargo install --locked wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

# Build
RUN cargo build --manifest-path squire_web/Cargo.toml --target wasm32-unknown-unknown
RUN ./trunk build squire_web/index.html

# Setup web server (NGINX)
FROM nginx:alpine

WORKDIR /usr/share/nginx/html
RUN rm -rf ./*
COPY --from=rust-build --chown=nginx /app/SquireCore/squire_web/dist/* .

EXPOSE 80/tcp

ENTRYPOINT ["nginx", "-g", "daemon off;"]
