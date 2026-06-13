---
name: runner-development
description: >-
  Guide for developing the MatchBox ESP32 Rust runner. Covers toolchain setup,
  ESP-IDF integration, build commands, and cross-compilation workflow.
applyTo: "**/*.rs **/Cargo.toml **/crates/**"
---

# Runner Development Skill

## Overview

The MatchBox ESP32 runner is a Rust application using `esp-idf-sys` and `boxlang-runtime` to execute BoxLang bytecode on ESP32 hardware. It handles HTTP routing, HID, WiFi, GPIO, and the VM fiber scheduler.

## Prerequisites

### Required Software

- **Rust** with `xtensa-esp32s3-espidf` target (nightly-based, not stable Rust)
- **ESP-IDF v5.2.3** at `/home/jacob/esp/esp-idf/`
- **ldproxy** linker (installed via `espup` or cargo)

### Activate ESP-IDF Environment

The runner requires ESP-IDF toolchain in the environment. **Must be sourced in every new terminal:**

```bash
source /home/jacob/esp/esp-idf/export.sh
```

## Build Commands

### Standard Release Build (~54s)

```bash
source /home/jacob/esp/esp-idf/export.sh
cd crates/matchbox-esp32-runner
cargo +esp build --release
```

Build output is at:
```
crates/matchbox-esp32-runner/target/xtensa-esp32s3-espidf/release/matchbox-esp32-runner
```

### Clean Build

```bash
cargo clean
cargo +esp build --release
```

### Cross Compile (from non-ESP-IDF host)

If ESP-IDF is not available locally, use the MatchBox CLI which handles the cross-compilation environment internally.

## Development Workflow

### 1. Make Changes to Runner Crate

Edit Rust source files in `crates/matchbox-esp32-runner/src/`. Key directories:

```
crates/matchbox-esp32-runner/
├── src/
│   ├── main.rs                # Entry point
│   ├── web/                   # HTTP server, route dispatch
│   ├── vm/                    # BoxLang VM integration
│   ├── fiber/                 # Background fiber scheduler
│   ├── hid/                   # USB HID support
│   ├── wifi/                  # Station + AP WiFi
│   ├── diagnostics.rs         # Boot/reboot tracking, NVS
│   └── storage/               # NVS persistence
└── partitions.csv             # ESP32 partition table
```

### 2. Build the Firmware

```bash
source /home/jacob/esp/esp-idf/export.sh
cargo +esp build --release --manifest-path crates/matchbox-esp32-runner/Cargo.toml
```

### 3. Flash to Device

Use a separate tool or the MatchBox CLI:

```bash
# Via ESPflash directly
espflash flash --chip esp32s3 --port /dev/ttyACM0 \
  crates/matchbox-esp32-runner/target/xtensa-esp32s3-espidf/release/matchbox-esp32-runner

# Via MatchBox CLI (recommended)
matchbox build-flash my-app/ --full-flash --esp32-web
```

### 4. Monitor Output

```bash
espflash monitor --chip esp32s3 --port /dev/ttyACM0
```

## Toolchain Details

### Rust Toolchain `esp`

The `esp` toolchain is a nightly-based fork maintained by Espressif. Install via `espup`:

```bash
espup install
```

This installs:
- `xtensa-esp-elf` — Xtensa cross-compiler
- `riscv32-esp-elf` — RISC-V cross-compiler (for ESP32-C3, C6, H2)
- Rust nightly fork with Xtensa target patches

Add to `~/.bashrc` or `~/.zshrc`:
```bash
source "$HOME/export-esp.sh"
```

### Target Triple

```
xtensa-esp32s3-espidf
```

This triple tells Cargo to use the Xtensa ESP32-S3 target with ESP-IDF as the platform. It is configured in:

```
crates/matchbox-esp32-runner/.cargo/config.toml
```

### Key Dependencies

See `crates/matchbox-esp32-runner/Cargo.toml`:

- `esp-idf-sys` — Raw bindings to ESP-IDF functions
- `esp-idf-svc` — High-level ESP-IDF service abstractions (WiFi, HTTP, etc.)
- `esp-idf-hal` — Hardware abstraction layer (GPIO, SPI, I2C, ADC)
- `boxlang-runtime` — MatchBox BoxLang VM crate from the same workspace

## Partition Table

The runner uses `crates/matchbox-esp32-runner/partitions.csv` for the ESP32 memory layout. Changes to this file require a full flash.

Current layout:

| Name | Type | Subtype | Offset | Size |
|------|------|---------|--------|------|
| nvs | data | nvs | 0x9000 | 16KB |
| otadata | data | ota | 0xd000 | 8KB |
| phy_init | data | phy | 0xf000 | 4KB |
| factory | app | factory | 0x10000 | 3MB |
| coredump | data | coredump | 0x310000 | 64KB |
| storage | data | spiffs | 0x320000 | ~3MB |

**Storage partition note:** The `spiffs` subtype is used for bytecode storage; this is a raw SPIFFS-like partition that stores serialized BoxLang bytecode.

## Troubleshooting Build Failures

### Linker errors (undefined reference)

Usually indicates `ldproxy` or `esp-idf-sys` is not properly configured. Ensure:
- ESP-IDF is sourced: `source /home/jacob/esp/esp-idf/export.sh`
- `ldproxy` is installed: `cargo install ldproxy`
- Target is installed: `rustup target add xtensa-esp32s3-espidf --toolchain esp`

### Compilation errors in boxlang-runtime

If `boxlang-runtime` fails to compile, check that it's using the `esp32` feature flag. This is configured in:
```toml
# crates/matchbox-esp32-runner/Cargo.toml
boxlang-runtime = { path = "../../boxlang-runtime", features = ["esp32"] }
```

### ELF file not found for coredump

If coredump decoding fails (no ELF symbol table), rebuild the runner in release mode:
```bash
cargo +esp build --release --manifest-path crates/matchbox-esp32-runner/Cargo.toml
```

The ELF with debug symbols is at:
```
crates/matchbox-esp32-runner/target/xtensa-esp32s3-espidf/release/matchbox-esp32-runner
```

## Adding Native BIFs

Adding a new BoxLang BIF that calls ESP32 hardware requires:

1. Define the BIF in `boxlang-runtime` with the `esp32` feature gate
2. Implement the platform call in `crates/matchbox-esp32-runner/src/` using `esp-idf-sys` or `esp-idf-hal`
3. Register the BIF in `crates/matchbox-esp32-runner/src/vm/`

Example of an ESP32 BIF registration:
```rust
// In runner/src/vm/bifs.rs or similar
#[cfg(feature = "esp32")]
pub fn esp32_heap_info() -> BIFResult {
    let heap = esp_idf_sys::heap_caps_get_free_size(ESP_IDF_SYS_HEAP_CAPS_DEFAULT);
    Ok(Value::Number(heap as i64))
}
```

## Related

- **matchbox-cli** — CLI workflows for building and flashing
- **build-flash-esp32** — End-to-end build and flash procedures
- **coredump-esp32** — Debugging runner crashes via coredumps
