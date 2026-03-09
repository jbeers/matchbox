# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# BoxLang WASM Container
#
# This image uses the WASM OCI image spec: the image contains only the
# compiled .wasm binary; the container runtime (Wasmtime shim) executes it.
# No OS, no shell, no libc — just the binary and the WASM runtime.
#
# Build (must specify --platform so the manifest matches the run command):
#   rustup target add wasm32-wasip1
#   matchbox --target wasi service.bxs   # compile BoxLang → service.wasm
#   docker buildx build --platform wasi/wasm32 -t matchbox-service .
#
# Run (Docker Desktop 4.15+ with built-in WASM support):
#   docker run -d --name matchbox-svc \
#              --runtime=io.containerd.wasmtime.v1 \
#              --platform=wasi/wasm32 matchbox-service
#   docker logs matchbox-svc && docker rm matchbox-svc
#
# Note: the containerd WASM shim buffers stdout to Docker's log driver.
# Use -d (detached) and docker logs to see output.
# ---------------------------------------------------------------------------

FROM scratch

COPY service.wasm /service.wasm

ENTRYPOINT ["/service.wasm"]
