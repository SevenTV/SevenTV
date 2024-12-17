# Services

We have a few different services that make up the platform:

- `api`
- `cdn`
- `event-api`
- `mongo-change-stream`
- `mongo-typesense`
- `website`
- `website-old`
- `image-processor`
- `nats` - message broker
- `mongodb` - database
- `typesense` - search engine
- `cloudflare` - cdn
- `cloudflared` - reverse proxy
- `wasabi` - s3 bucket
- `hetzner` - server provider
- `clickhouse` - analytics
- `victoriametrics` - metrics
- `grafana` - monitoring
- `minio` - s3 bucket (for monitoring)
- `loki` - logging
- `tempo` - tracing

## API

The main API server used for serving:

- v3/rest (old rest api)
- v3/graphql (old gql api)
- v4/rest (new rest api)
- v4/graphql (new gql api)

Remaining work:

The new v4 api is not fully implemented yet and almost everyone is still using the old api.

In production we serve the api via `cloudflared` tunnels.

## CDN

The CDN server is used for serving image files to users.

This also acts as a cache for images and the source of truth is a s3 bucket.

We use `wasabi` in production and `minio` in development.

In production we serve the CDN by directly binding to ports `80` and `443` on the server for both TCP & QUIC.

The reason being is if we use cloudflare we have to pay for the bandwidth and we serve a lot of traffic.

We could serve it behind a reverse proxy but that is inefficient so we just serve it directly.

## Event API

The event api serves websockets to users to give live updates when something happens.

The event api gets events from NATs and then forwards them to the desired users.

In production we bind directly to port `443`, however we still use cloudflare to reverse proxy.

We do not use a reverse proxy such as `nginx` or `cloudflared` because memory is a problem. In production we have ~1mil websocket connections and using any reverse proxy would double the memory needed per connection. We dont need to bind to port `80` or use QUIC because cloudflare does that for us.

## Mongo Change Stream

The mongo change stream service (as the name suggests) consumes the mongo change stream and forwards it to NATs so that the `mongo-typesense` service can update the typesense index.

## Mongo Typesense

This service digests the change stream (in nats) and then reindexes the typesense index.

We also periodically query the database and update any documents that we somehow missed in the change stream.

## Website

The website for the platform. https://7tv.app (powered via v4)

## Website Old

The old website for the platform. https://old.7tv.app (powered via v3)


## Other Services

These services are used internally and are not exposed publicly.

- `nats` - message broker
- `mongodb` - database
- `typesense` - search engine
- `cloudflare` - cdn
- `cloudflared` - reverse proxy
- `wasabi` - s3 bucket
- `hetzner` - server provider
- `clickhouse` - analytics
- `victoriametrics` - metrics
- `grafana` - monitoring
- `minio` - s3 bucket (for monitoring)
- `loki` - logging
- `tempo` - tracing

### Image Processor

A service written by `ScuffleCloud` which we use for processing our images and resizing them to the correct sizes.

This service needs a mongo db database to write the task list and then uses NATs to respond when the task is complete.

the ImageProcessor uploads the images to the s3 bucket on completion.

This service is not exposed publicly and is only used internally.

### NATs

NATS is used for inter-service communication as a message broker.

We use NATs for the `mongo-change-stream` service to forward the mongo change stream to the `mongo-typesense` service.

We also use NATs for the `event-api` service to forward events to the desired users.

### MongoDB

MongoDB is used as a database for the platform.

### Typesense

Typesense is used as a search engine for the platform.

### Cloudflare

Cloudflare is a reverse proxy we use to serve the api and cdn.

The cdn is primarily served on our own servers however in regions other than europe we use cloudflare to cache the images.

### Cloudflared

Cloudflared is used as a reverse proxy for the platform.

### Clickhouse

Clickhouse is used for analytics.

### VictoriaMetrics

VictoriaMetrics is used for application metrics. Accessed via grafana. Manages its own storage.

### Grafana

Grafana is used for visualizing metrics.

You can login to grafana at https://grafana.k8s.7tv.io/ with a google email (@7tv.app)

### Tempo

Tempo is used for tracing. Accessed via grafana. (uploads traces to minio)

### Loki

Loki is used for logging. Accessed via grafana. (uploads logs to minio)

### Minio

Minio is used for storing logs and traces.

### Hetzner

Hetzner is our cloud provider we have around ~30 bare metal servers with them.
