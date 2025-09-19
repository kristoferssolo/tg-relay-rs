FROM rust:1.90-slim-trixie as builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY . .

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates ffmpeg \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM python:3-slim AS runtime
FROM python:3.12-slim-trixie
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

ENV TZ=UTC
ENV RUST_LOG=info
ENV PATH=/home/app/.local/bin:$PATH

RUN useradd --create-home --shell /bin/bash app
WORKDIR /home/app
USER app

RUN uv tool install instaloader \
    && instaloader --version

RUN uv tool install yt-dlp \
    && yt-dlp --version

COPY --from=builder /app/target/release/tg-relay-rs /usr/local/bin/tg-relay-rs

CMD ["/usr/local/bin/tg-relay-rs"]
