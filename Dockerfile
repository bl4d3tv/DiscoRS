FROM rust:1.69-slim as builder

ARG GITHUB_RUN_NUMBER=N/A
ARG TARGETARCH

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    TZ=UTC \
    TERM=xterm-256color \
    USER="root" \
    GITHUB_RUN_NUMBER=$GITHUB_RUN_NUMBER

RUN case $TARGETARCH in \
    "amd64") echo "x86_64-unknown-linux-gnu" >> /.platform && \
    echo "" >> /.compiler;; \
    "arm64") echo "aarch64-unknown-linux-gnu" >> /.platform && \
	echo "gcc-aarch64-linux-gnu" >> /.compiler;; \
    esac

RUN apt update && \ 
    apt install -y --no-install-recommends \ 
    libssl-dev \
    cmake \
    perl \
    gcc \
    libopus0 \
    opus-tools \
    libopus-dev \
    make \
    git \
    pkg-config \
    $(cat /.compiler)

# Set minimal Rust profile
RUN rustup set profile minimal && \
    rustup target add $(cat /.platform)

# Create dumb project to cache dependencies
RUN USER=root cargo new --bin /disco-rs

WORKDIR /disco-rs

COPY Cargo.toml /disco-rs/

COPY Cargo.lock /disco-rs/

RUN cargo build --target $(cat /.platform) --release

RUN rm src/*.rs && \ 
    rm target/$(cat /.platform)/release/deps/disco_rs*

COPY . /disco-rs

RUN git config --global --add safe.directory /app

RUN cargo build --target $(cat /.platform) --release

RUN mkdir /app && \
    cp /disco-rs/target/$(cat /.platform)/release/disco-rs /app 

FROM debian:11-slim as runner

ENV RUST_LOG=none,disco_rs=error

RUN apt update \
    && apt install -y \ 
    dumb-init \
    python3 \
    openssl \
    wget \
    ca-certificates \
    xz-utils \
    opus-tools \
    libopus0

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

#RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb && \
    #dpkg -i libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb

WORKDIR /app

COPY --from=builder /app/disco-rs /app/disco-rs

RUN useradd bot && \
    chown -R bot:bot /app && \
    chmod +x /app/disco-rs

USER bot

CMD ["/usr/bin/dumb-init", "/app/disco-rs"]
