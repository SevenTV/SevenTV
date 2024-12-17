# Deployment

Currently we deploy things manually.

We build the docker images for the services we want to deploy and push them to our docker registry (ghcr.io/seventv)

- API: `ghcr.io/seventv/api-new:latest`
- CDN: `ghcr.io/seventv/cdn-new:latest`
- Event API: `ghcr.io/seventv/event-api-new:latest`
- Mongo Change Stream: `ghcr.io/seventv/mongo-change-stream-new:latest`
- Mongo Typesense: `ghcr.io/seventv/mongo-typesense-new:latest`
- Website: `ghcr.io/seventv/website-new:latest`

After running

```
cargo build -r --bin api
```

We can build the docker image with

```
docker build -f ./apps/api/Dockerfile . --tag ghcr.io/seventv/api-new:<some-new-tag> --push --build-arg="BUILD_TARGET=release"
```

We can then deploy the version via k8s.

```
kubectl patch deployment -n app api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","image":"ghcr.io/seventv/api-new:<some-new-tag>"}]}}}}'
```

Or via a GUI like [lens](https://k8slens.dev/) (preferred way).

## Build notes

When building the API you need to have a `./local/GeoLite2-Country.mmdb` file so that it can be copied into the docker image. You can get this file from [MaxMind](https://dev.maxmind.com/geoip/geoip2/geolite2/).

## Configs & Secrets

Kubernetes secrets are stored the `app` namespace and each service has its own secret with values it needs.

Configs are also stored in the `app` namespace where there is one for each service and we use Jinja2 templates to do env var substitution (from the secrets).

## Deployment vs DaemonSet

Deployments are generally used for most services. The only exceptions are the `CDN` and `Event API` which are deployed as DaemonSets. This is because they bind to the node port directly and therefore cannot run multiple instances on the same node.

## Future Work

Add a CI/CD pipeline to automatically deploy the latest version of the services when a new commit is pushed to the repository.
