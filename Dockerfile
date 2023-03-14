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

FROM ubuntu:22.04 as runner

ENV RUST_LOG=none,disco_rs=error

RUN apt update \
    && apt install -y \ 
    dumb-init \
    python3 \
    openssl \
    wget \
    ca-certificates \
    xz-utils

WORKDIR /tmp

# Setup custom patched ffmpeg
RUN wget https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz && \
    tar -xvf ffmpeg-master-latest-linux64-gpl.tar.xz && \
    cp ffmpeg-master-latest-linux64-gpl/bin/* /usr/local/bin && \
    chmod +x /usr/local/bin/ffmpeg /usr/local/bin/ffplay /usr/local/bin/ffprobe

# Setup yt-dlp
RUN wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp && \
    cp yt-dlp /usr/local/bin && \
    chmod +x /usr/local/bin/yt-dlp

# Install missing libssl1 needed for some crates

RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb && \
    dpkg -i libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb

RUN useradd -m disco

WORKDIR /app
COPY --chown=disco:disco --from=builder /app/target/release/disco-rs /app/disco-rs

RUN chmod +x /app/disco-rs && chown -R disco:disco /app

USER disco

CMD ["/usr/bin/dumb-init", "/app/disco-rs"]
