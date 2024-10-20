FROM ubuntu:24.04

WORKDIR /app

ARG BUILD_TARGET=release-lto

# Copy the binary from the host machine
# This assumes that you have already built the binary using the
# `cargo build --profile release-lto` command.
RUN --mount=type=bind,src=target/${BUILD_TARGET}/event-api,dst=/mount/event-api \
    cp /mount/event-api /app/event-api && \
    chmod +x /app/event-api && \
    apt update && \
    apt install -y ca-certificates libjemalloc2 && \
    rm -rf /var/lib/apt/lists/*

STOPSIGNAL SIGTERM

USER 1000

# This is our default configuration for jemalloc
# See https://github.com/jemalloc/jemalloc/blob/dev/TUNING.md for more information
# on the various options.
ARG JEMALLOC_CONF="background_thread:true,tcache_max:4096,metadata_thp:always,dirty_decay_ms:3000,muzzy_decay_ms:3000,abort_conf:true"

# Set the environment variables for jemalloc
ENV _RJEM_MALLOC_CONF=${JEMALLOC_CONF} \
    MALLOC_CONF=${JEMALLOC_CONF} \
    LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so.2

ENTRYPOINT ["/app/event-api"]
