# Personal GoLinks

Inspired by golinks.io

This application is a redirect service written in Rust that uses routes statically defined in a YAML file to provide easy link shortening.

Although prior versions written in Go (`>=1.0 <1.1`) supported stateful updates to the running routes, this was removed from the Rust implementation (`>=1.1`) as I was only deploying the application in my cluster, and ultimately I still managed my available routes by synchronizing the running state with a YAML file that a client used.

Since the application now uses a `ConfigMap` to deploy to a cluster, I am able to scale up to as many replicas as I want without having to worry about persistance or synchronizing storage. This doesn't matter for a service that is only being used by me, but was a fun experiment.

## Quickstart

1. Install Rust
1. Clone repository
1. Write a `links.yaml` file
1. Run `cargo run`
1. Go to `localhost:8000/heartbeat` and see the JSON output

### Example `links.yaml` file

```yaml
routes:
  golinks: https://github.com/cryptaliagy/golinks
  google: https://google.com
  github: https://github.com
```

## Quickstart (Docker)

1. Create a `conf` directory and write a `links.yaml` file in it (see above for example).
1. Pull the container with `docker pull ghcr.io/cryptaliagy/golinks:latest`
1. Run the container with `docker run -e GOLINKS_ROUTES=/conf/links.yaml ROCKET_LOG_LEVEL=normal -v "$(pwd)"/conf:/conf -p 8000:8000 ghcr.io/cryptaliagy/golinks:latest`
1. Go to `localhost:8000/heartbeat` and see the JSON output

## Installation (Helm)

```bash
helm repo add golinks https://cryptaliagy.github.io/golinks/charts
helm repo update
helm install golinks/golinks
```

### K3s HelmChart CRD

> NOTE: Make sure you configure ingress/service in your values file contents appropriately.

```yaml
---
apiVersion: helm.cattle.io/v1
kind: HelmChart
metadata:
  name: golinks
  namespace: charts
spec:
  chart: golinks
  repo: https://cryptaliagy.github.io/golinks/charts
  targetNamespace: golinks
  valuesContent: |-
    routes:
      golinks: https://github.com/cryptaliagy/golinks
      google: https://google.com
```

## Installation (Manifest)

Generate a manifest from Helm

```bash
helm template --repo https://cryptaliagy.github.io/golinks/charts golinks -g > golinks.yaml
```

Edit it as desired then apply it to the cluster

```bash
kubectl apply -f golinks.yaml
```

## Additional Notes

This service is very simple, and as such is designed to be as minimal as possible. Running on my machine, the container took up ~1MB of memory, and simple route profiling showed that routes took ~20-40 μs.

The final container image size is <10MB and uses an image based on `scratch` with a statically-linked binary. This also means that the `latest` and `x.y.z` versioned containers do not have a shell or any additional tools. For potential debugging purposes, a `debug` (and `x.y.z-debug`) container is available. This uses the same binary that `latest` (and `x.y.z`) uses, but is based on `alpine:latest` to include a shell, package manager, etc.

### Deploying to Cloud Services

If you would like to deploy this to a cloud service and do not have the ability to mount a file (or otherwise would like the container to be fully atomic), you can build your own image that includes the route configs. To do this, first author a `links.yaml` file, then use the following Dockerfile:

```Dockerfile
FROM ghcr.io/cryptaliagy/golinks:latest

COPY links.yaml /links.yaml
```

This image can then be pushed to a container registry and deployed in the cloud service of your choice.
