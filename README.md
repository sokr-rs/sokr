# SOKR — Sovereign Open Kernel Runtime

> Immutable sovereign core. Everything else a plugin.

[![Crates.io](https://img.shields.io/crates/v/sokr.svg)](https://crates.io/crates/sokr)
[![CI](https://github.com/sokr-rs/sokr/actions/workflows/ci.yml/badge.svg)](https://github.com/sokr-rs/sokr/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

---

## What is SOKR?

SOKR is a sovereign compute runtime written in Rust where the core is
immutable and everything else is a plugin: IR representations, substrate
backends, language bindings, and dispatch policy.

The core exposes exactly three operations across a stable C ABI surface:

| Operation | Question |
|---|---|
| **Capability** | Can this substrate fulfill this computation? |
| **Dispatch** | Fulfill it. |
| **Completion** | Signal when fulfilled. |

No assumption is made about memory model, parallelism, execution time,
or computation representation. Any substrate that can answer three
questions is a valid SOKR backend — including substrates that do not
yet exist.

---

## Philosophy

The algorithm is the permanent asset. The substrate is a runtime decision.

SOKR imposes no conditions on the algorithm, the substrate, or the user.
No vendor can revoke it. No export restriction can strand it. No hardware
generation can obsolete it.

---

## Architecture

```
User code (any language)
        ↓
IR Plugin          ← swappable: SOKR-native, SPIR-V, OpenQASM, or future
        ↓
SOKR Core          ← immutable, no_std, C ABI  ← this repo
  Capability → Dispatch → Completion
        ↓
Substrate Plugin   ← swappable: GPU, CPU, QPU, Neuromorphic, Photonic, or future
        ↓
Hardware
```

**This repo is the core only.** Reference plugins live in
[sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins).
Third-party plugins need only depend on `sokr` and implement the
`SokrSubstratePlugin` vtable. No permission required from anyone.

---

## How SOKR Differs

Every existing approach solves part of the problem:

| Project | Language | `no_std` | C ABI Plugin | QPU/Neuro/Photonic | Sovereign |
|---|---|---|---|---|---|
| CUDA | C/C++ | ❌ | ❌ | ❌ GPU only | ❌ NVIDIA |
| CubeCL | Rust | ⚠️ partial | ❌ | ❌ GPU only | ✅ |
| wgpu | Rust | ⚠️ partial | ❌ | ❌ GPU only | ✅ |
| CUDA-Q | C++/Python | ❌ | ❌ | ✅ QPU | ❌ NVIDIA |
| NIR | Python | ❌ | ❌ | ✅ Neuromorphic | ✅ |
| hetGPU | Research | ❌ | ❌ | ❌ GPU only | ✅ |
| **SOKR** | **Rust** | **✅** | **✅** | **contract-ready**¹ | **✅** |

¹ *contract-ready*: the three-function ABI admits these substrates by
design. No QPU, neuromorphic, or photonic plugin ships yet — see
[sokr-plugins Phase 4](https://github.com/sokr-rs/sokr-plugins/blob/main/TODO.md#phase-4--future-substrates).
CUDA-Q ships QPU support today; NIR ships neuromorphic today. SOKR's
claim in this column is structural, not operational.

SOKR is the only runtime whose core makes no assumption about the
substrate model: a sovereign, `no_std` Rust core with a stable C ABI
plugin contract. Whether that contract carries real workloads on
non-GPU substrates is a question the plugin layer answers, not the core.

---

## Repository Structure

```
sokr/                          ← this repo (sokr-rs/sokr)
├── src/
│   ├── lib.rs                 ← crate root, no_std
│   ├── types.rs               ← C ABI struct and enum definitions
│   ├── registry.rs            ← plugin registry
│   └── ffi.rs                 ← #[no_mangle] extern "C" exports
├── include/
│   ├── sokr.h                 ← generated C header (committed)
│   └── sokr.hpp               ← C++ RAII wrapper
├── examples/
│   ├── c/hello_compute.c      ← C usage example
│   └── cpp/hello_compute.cpp  ← C++ usage example
├── benches/
│   └── dispatch_overhead.rs   ← FFI overhead benchmark (<0.01%)
├── docs/
│   ├── rfc/                   ← RFC documents
│   └── references.md          ← curated references
├── xtask/                     ← cargo xtask (header generation)
├── agents/                    ← harness agent definitions
├── audit.toml                 ← cargo-audit configuration
├── deny.toml                  ← cargo-deny license/dep policy
├── cbindgen.toml              ← C header generation config
├── ARCHITECTURE.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── TODO.md
```

Plugins: [github.com/sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins)

---

## Usage

### Rust (plugin author)

```toml
[dependencies]
sokr = "0.3"
```

Include the generated C header in your plugin:

```c
#include "sokr.h"   // from include/sokr.h in this repo
```

### C ABI surface

The entire runtime is three functions:

```c
// Can this substrate handle this computation?
SokrResult sokr_capability(
    const SokrCapabilityQuery *query,
    SokrCapabilityResponse    *response);

// Dispatch to a substrate; returns a completion token.
SokrResult sokr_dispatch(
    const SokrDispatchRequest *request,
    SokrDispatchResponse      *response);

// Poll the completion token.
SokrResult sokr_completion(
    const SokrCompletionQuery *query,
    SokrCompletionSignal      *signal);
```

Register your plugin before calling them:

```c
SokrSubstratePlugin plugin = { ... };
uint64_t id = 0;
sokr_register_substrate(&plugin, &id);
```

See [`include/sokr.h`](include/sokr.h) for the full type definitions and
[`docs/rfc/0001-plugin-interface.md`](docs/rfc/0001-plugin-interface.md)
for the plugin contract.

---

## Status

`v0.3.0` — **ABI Frozen.** Core is production-ready with full integration testing against
[sokr-cpu](https://github.com/sokr-rs/sokr-plugins/tree/main/sokr-cpu). C and C++ bindings
are complete with examples. Dispatch overhead measured at ~3 ns/call absolute
(validation + registry lookup + indirect call) — negligible against any real substrate
workload. Ready for third-party substrate plugins.

**Toward v1.0 (formal verification).** The version-handshake protocol is specified
in TLA+ ([`docs/formal_spec.tla`](docs/formal_spec.tla)) and machine-checked with the
TLC model checker — every reachable state preserves the compatibility invariant, so an
incompatible plugin can never occupy a registry slot. See
[`docs/formal-spec-guide.md`](docs/formal-spec-guide.md) for the run command and results.

See [TODO.md](TODO.md) for the roadmap and
[ARCHITECTURE.md](ARCHITECTURE.md) for the design.

---

## License

Licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your discretion.

Any contribution intentionally submitted for inclusion in this work
shall be dual licensed as above, without any additional terms or conditions.

---

*Copyright 2026 The SOKR Project*
