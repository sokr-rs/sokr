# SOKR — Sovereign Open Kernel Runtime

> Immutable sovereign core. Everything else a plugin.

**Early design phase. No API is stable. No runnable code yet.**

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
| **SOKR** | **Rust** | **✅** | **✅** | **✅** | **✅** |

SOKR is the only runtime that satisfies all four simultaneously:
a sovereign, `no_std` Rust core with a stable C ABI plugin contract
that makes no assumption about the substrate model.

---

## Repository Structure

```
sokr/                    ← this repo (sokr-rs/sokr)
├── src/
│   ├── lib.rs           ← crate root, no_std
│   ├── types.rs         ← C ABI struct and enum definitions
│   ├── registry.rs      ← plugin registry
│   └── ffi.rs           ← #[no_mangle] extern "C" exports
├── docs/
│   ├── rfc/             ← RFC documents
│   └── references.md    ← curated references
├── Cargo.toml           ← single crate, no workspace
└── cbindgen.toml        ← C header generation config
```

Plugins: [github.com/sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins)

---

## Status

`v0.1.x` — foundation phase. Core ABI defined. No runnable code yet.

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
