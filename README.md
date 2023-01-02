# envoy-wasm-filter

Proof-of-concept WASM Envoy Filter which makes gRPC requests to enhance a given request. Developed with Rust.

Demonstrates the following:
- Making (different) gRPC requests from an Envoy Plugin
- Using `proxy-wasm-rust-sdk` to configure the plugin, log metrics and enhance the request
- Packaging the plugin as OCI image
- Sample setup in docker

# Local Setup

Runs the filter as an addon in manually configured envoy proxy.

Requires:
- `rust` >= 1.66.0
- `docker` and `docker compose`
- `wasm32` target. Install with: `rustup target add wasm32-unknown-unknown`

No `protoc` is necessary, since `protoc-bin-vendored` crate downloads and uses the appropriate binary.

    cargo build --target wasm32-unknown-unknown --release
    docker compose up

Test with:

    curl -H 'token: hello' -v localhost # Should add '"my-token": "hello! from grpc"' to upstream headers
    curl -H 'other: hi:' -v localhost   # Should add '"my-token": "hi! from grpc"' to upstream headers

# Minikube Setup

Runs the filter in istio configured in a `minikube` cluster. 

Requires:
- `minikube`
- `kubectl`
- `helm`

Guide uses `default` minikube profile, please clean up before starting with `minikube delete`.

    minikube start --insecure-registry "10.0.0.0/24"
    minikube addons enable registry

Install `istio` with `helm`. This should be the same as described by [istio docs](https://istio.io/latest/docs/setup/install/helm/):

    helm repo add istio https://istio-release.storage.googleapis.com/charts
    helm repo update

    kubectl create namespace istio-system
    helm install istio-base istio/base -n istio-system
    helm install istiod istio/istiod -n istio-system --wait

Prepare the protobuf files and stubs to be used by gripmock. It will simulate our internal gRPC server:

    kubectl apply -f k8s/0_namespace.yaml
    kubectl create configmap stubs -n greeter --from-file=./stubs
    kubectl create configmap proto -n greeter --from-file=./proto
    kubectl apply -f k8s/1_grpc_service.yaml

Now build and push the OCI image of the plugin to docker registry residing in `minikube`:

    # Run the following in a seperate terminal. Has to be open during each docker push
    kubectl port-forward -n kube-system service/registry 5000:80

    docker build -t envoy-wasm-plugin .
    docker tag envoy-wasm-plugin:latest localhost:5000/envoy-wasm-plugin:latest
    docker push localhost:5000/envoy-wasm-plugin:latest 

Deploy the application
    kubectl apply -f k8s/2_echo.yaml

Test with:
    kubectl run curl --image=curlimages/curl -it --rm -- /bin/sh

# Notes

1. Difference between `wasm32-unknown-unknown` and `wasm32-wasi`:
  
  `wasm32-unknown-unknown` is closer to bare metal - file i/o, env. vars, and other functionality that's normally supported by the OS is available. `wasm32-wasi` can access some of these functionality, because `wasi` interface brings it. More info [here](https://users.rust-lang.org/t/wasm32-unknown-unknown-vs-wasm32-wasi/78325/5)

2. Which crate to use: `proxy-wasm` vs `envoy-sdk`?:

  `envoy-sdk` ([github](https://github.com/tetratelabs/envoy-wasm-rust-sdk/tree/master/envoy-sdk)) is deprecated, although still appears in some early examples. The `proxy-wasm` ([github](https://github.com/proxy-wasm/proxy-wasm-rust-sdk)) is the preferred SDK now, and it's not Envoy specific. Notably, `nginx` will be supporting `proxy-wasm` ABI in the future.

## Resources:
- [WASM Workshop from Tetratelabs](https://tetratelabs.github.io/wasm-workshop/) Very useful tutorial, uses go to build a sampel filter and deploy to Istio.
- [Extending Envoy with WASM](https://events.istio.io/istiocon-2021/slides/c8p-ExtendingEnvoyWasm-EdSnible.pdf) Explains WASM filter development in detail (for C++ SDK).