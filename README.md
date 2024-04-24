# rust-tracing

## Summary

Sample project to test opentelemetry in a distributed system written in Rust.

## Start locally 

To run and test this tracing setup you need to perform the following steps:

### Start a local jaeger instance

Run the all-in-one container setup for **jaeger**.

```sh
podman run --rm --name jaeger \
  -e COLLECTOR_ZIPKIN_HOST_PORT=:9411 \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 5778:5778 \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  -p 14250:14250 \
  -p 14268:14268 \
  -p 14269:14269 \
  -p 9411:9411 \
  jaegertracing/all-in-one:1.54
```

### Starting the applications

Ensure your terminal location is currently set to the root of this repository.

Start the [axum-downstream-api](./axum-downstream-api/).

```sh
cargo run --bin axum-downstream-api
```

Start the [axum-api](./axum-api/).

```sh
cargo run --bin axum-api
```

Start the [dotnet-api](./dotnet-api/).

```sh
dotnet run --project ./dotnet-api/DotnetApi.csproj
```

Start the [cli-client](./cli-client/).

```sh
cargo run --bin cli-client
```

### See the traces

1. You need to use the **cli-client** to send some requests
2. Open the [jaeger ui](http://localhost:16686/search)
