FROM rust:latest as builder

ADD . /app
WORKDIR /app

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:latest
LABEL maintainer="sean@seanmooney.info"
# add image name and tag information
LABEL org.opencontainers.image.title="shorter"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.description="A simple URL shortener"
LABEL org.opencontainers.image.authors="Sean Mooney <sean@seanmooney.info>"
LABEL org.opencontainers.image.url="grc.io/seanmooney/shorter"
LABEL org.opencontainers.image.src="github.com/seanmooney/shorter"
LABEL org.opencontainers.image.licenses="BSD-3-Clause"
LABEL org.opencontainers.image.base.name="gcr.io/distroless/cc-debian12"
LABEL org.opencontainers.image.base.version="latest"
LABEL org.opencontainers.image.docker.cmd.devel = "docker run --rm -it -p 8000:8000 --name shorter"


COPY --from=builder /app/target/release/shorter /usr/local/bin/shorter
COPY --from=builder /app/Rocket.toml /Rocket.toml

EXPOSE 8000
VOLUME [ "/data" ]
ENV SHORTER_DATA_DIR=/data
ENV ROCKET_CONFIG=/Rocket.toml
ENTRYPOINT ["/usr/local/bin/shorter"]