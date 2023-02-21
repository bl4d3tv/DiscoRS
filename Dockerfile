FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \ 
    dumb-init \
    ffmpeg \
    python3 \
    python3-pip \
    openssl \
    wget

RUN pip3 install youtube_dl

RUN adduser --no-create-home --disabled-login --uid 1001 --group disco

WORKDIR /disco-rs
COPY --chown=disco:disco ./target/release/disco-rs ./run

RUN chmod +x run

USER disco

CMD ["/usr/bin/dumb-init", "/disco-rs/run"]
