FROM ubuntu:lunar

WORKDIR /app

RUN --mount=type=bind,src=target/release-lto/event-api,dst=/mount/event-api \
    cp /mount/event-api /app/event-api && \
    chmod +x /app/event-api

STOPSIGNAL SIGTERM

USER 1000

ARG JEMALLOC_CONF="background_thread:true,tcache_max:4096,metadata_thp:always,dirty_decay_ms:3000,muzzy_decay_ms:3000,abort_conf:true"

ENV _RJEM_MALLOC_CONF=${JEMALLOC_CONF} \
    MALLOC_CONF=${JEMALLOC_CONF}

ENTRYPOINT ["/app/event-api"]
