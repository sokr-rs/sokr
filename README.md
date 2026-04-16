# SOKR — Sovereign Open Kernel Runtime

> Immutable sovereign core. Everything else a plugin.

**Early design phase. No API is stable. No runnable code yet.**

[![Crates.io](https://img.shields.io/crates/v/sokr.svg)](https://crates.io/crates/sokr)
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
SOKR Core          ← immutable, no_std, C ABI
  Capability → Dispatch → Completion
        ↓
Substrate Plugin   ← swappable: GPU, CPU, QPU, Neuromorphic, Photonic, or future
        ↓
Hardware
```

Everything above and below the core is a plugin. The core contains
no assumptions — not about computation representation, not about
hardware model, not about language, not about security policy.

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

## Target Substrates

Current and future:

- GPU — NVIDIA (PTX/CUDA), AMD (HIP), Intel Arc (SPIR-V)
- CPU — fallback, always available
- QPU — quantum processors (OpenQASM 3)
- Neuromorphic — Intel Loihi, IBM NorthPole
- Photonic — Lightmatter and successors
- WebGPU — browser and edge compute

---

## Status

`v0.1.0` — foundation phase. Crate reserved. Architecture and roadmap in progress.

See [TODO.md](TODO.md) for the full roadmap and [ARCHITECTURE.md](ARCHITECTURE.md) for the design.

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
