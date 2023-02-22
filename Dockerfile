FROM rust:1.67 as builder

ARG GITHUB_RUN_NUMBER=N/A

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    TZ=UTC \
    TERM=xterm-256color \
    CARGO_HOME="/root/.cargo" \
    USER="root" \
    GITHUB_RUN_NUMBER=${GITHUB_RUN_NUMBER}

RUN mkdir -pv "${CARGO_HOME}" \
    && rustup set profile minimal

RUN apt update && \ 
    apt install -y --no-install-recommends \ 
    libssl-dev \
    cmake

WORKDIR /app

COPY . .

RUN cargo build --release

FROM ubuntu:22.04

ENV RUST_LOG=none,disco_rs=error

RUN apt update \
    && apt install -y --no-install-recommends \ 
    dumb-init \
    ffmpeg \
    python3 \
    python3-pip \
    openssl \
    wget

RUN pip3 install youtube_dl

# Install missing libssl1 needed for some crates
WORKDIR /tmp
RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb && \
    dpkg -i libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb

RUN useradd -M disco

WORKDIR /app
COPY --chown=disco:disco --from=builder /app/target/release/disco-rs /app/disco-rs

RUN chmod +x /app/disco-rs

USER disco

CMD ["/usr/bin/dumb-init", "/app/disco-rs"]
