FROM rust:slim-bookworm AS builder
WORKDIR /build

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
         build-essential \
         pkg-config \
         cmake \
         libclang-dev \
         clang \
         libz3-dev \
         python3 \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY canonicalize-server/Cargo.toml canonicalize-server/Cargo.toml
COPY eq677/Cargo.toml eq677/Cargo.toml
COPY eq677/build.rs eq677/build.rs
RUN mkdir -p canonicalize-server/src eq677/src \
    && echo 'fn main(){}' > canonicalize-server/src/main.rs \
    && echo '' > eq677/src/lib.rs \
    && echo 'fn main(){}' > eq677/src/main.rs \
    && cargo build --release -p canonicalize-server \
    && rm -rf canonicalize-server/src eq677/src

COPY canonicalize-server/src canonicalize-server/src
COPY eq677/src eq677/src
RUN touch canonicalize-server/src/main.rs eq677/src/lib.rs eq677/src/main.rs \
    && cargo build --release -p canonicalize-server --bin canonicalize-server \
    && strip target/release/canonicalize-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libz3-4 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --home-dir /nonexistent --shell /usr/sbin/nologin canon

COPY --from=builder /build/target/release/canonicalize-server /usr/local/bin/canonicalize-server
USER canon
EXPOSE 8080
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/canonicalize-server"]
