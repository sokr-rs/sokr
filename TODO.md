# SOKR — Development TODO

> Sovereign Open Kernel Runtime
> Last Updated: 2026-04-16
> Legend: 🔴 Critical path · 🟡 Important · 🟢 Nice-to-have

---

## Vision

A sovereign compute runtime where the algorithm is the permanent asset
and the substrate is a runtime decision — for hardware that exists today
and hardware that does not yet exist.

---

## Phase 0 — Foundation `v0.1.x`
> Claim the name. Establish the philosophy. No runnable code yet.
> **Current phase.**

### 0.1 Identity
- [x] 🔴 Name locked: **SOKR — Sovereign Open Kernel Runtime**
- [x] 🔴 License decided: **MIT OR Apache-2.0**
- [x] 🔴 Copyright holder: **The SOKR Project**
- [x] 🔴 Crate reserved on crates.io (`v0.1.0`)
- [x] 🔴 GitHub org claimed: `sokr-rs`
- [ ] 🔴 GitHub repo made public with README, ARCHITECTURE, TODO
- [ ] 🔴 CONTRIBUTING.md — contribution guidelines, DCO sign-off requirement

### 0.2 Design Documents
- [x] 🔴 Core philosophy documented
- [x] 🔴 Three-function interface defined: Capability, Dispatch, Completion
- [x] 🔴 Plugin categories defined: IR, Substrate, Language Binding, Dispatch Policy
- [x] 🔴 IR hybrid strategy documented
- [x] 🔴 Architecture layering documented
- [ ] 🔴 C ABI surface specification — formal definition of types and function signatures
- [ ] 🔴 Version handshake protocol — specification for plugin compatibility negotiation
- [ ] 🟡 Plugin interface RFC — open for community comment before v0.2.0 freeze

### 0.3 Tooling
- [ ] 🔴 `cargo install cargo-audit` — security audit in CI
- [ ] 🔴 GitHub Actions CI — `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`
- [ ] 🔴 `.github/ISSUE_TEMPLATE/` — bug report, feature request, plugin proposal templates
- [ ] 🟡 `deny.toml` — license and dependency policy via `cargo-deny`
- [ ] 🟡 Dependabot configuration

---

## Phase 1 — Core Skeleton `v0.2.0`
> The immutable core exists. One substrate works. Nothing is final.

### 1.1 Core ABI
- [ ] 🔴 Define `SokrVersion` — version handshake struct (`major`, `minor`, `patch`)
- [ ] 🔴 Define `SokrCapabilityQuery` — computation descriptor struct, C-compatible
- [ ] 🔴 Define `SokrDispatchRequest` — dispatch payload struct, C-compatible
- [ ] 🔴 Define `SokrCompletionSignal` — completion enum, C-compatible
- [ ] 🔴 Define `SokrSubstratePlugin` — vtable struct: three function pointers + version
- [ ] 🔴 `sokr_capability()` — C ABI function, routes to substrate plugin
- [ ] 🔴 `sokr_dispatch()` — C ABI function, routes to substrate plugin
- [ ] 🔴 `sokr_completion()` — C ABI function, routes to substrate plugin
- [ ] 🔴 `cbindgen` configuration — generate `sokr.h` from Rust types
- [ ] 🔴 `no_std` enforcement — `#![no_std]` with `#![forbid(unsafe_code)]` in core

### 1.2 Plugin Registry
- [ ] 🔴 Plugin registration API — register a substrate plugin at runtime
- [ ] 🔴 Plugin deregistration API — remove a substrate plugin cleanly
- [ ] 🔴 Plugin version negotiation — reject incompatible plugin versions at load time
- [ ] 🟡 Plugin registry introspection — list registered plugins and their capabilities

### 1.3 CPU Substrate Plugin (`sokr-cpu`)
- [ ] 🔴 Scaffold `sokr-cpu` crate in workspace
- [ ] 🔴 Implement `Capability` — always returns capable for any computation
- [ ] 🔴 Implement `Dispatch` — naive CPU thread execution
- [ ] 🔴 Implement `Completion` — synchronous, immediate
- [ ] 🔴 Integration test — round-trip: register → capability → dispatch → completion
- [ ] 🟡 Benchmark baseline — CPU fallback performance reference

### 1.4 First Dispatch Policy Plugin (`sokr-dispatch-first`)
- [ ] 🔴 Scaffold `sokr-dispatch-first` crate
- [ ] 🔴 Strategy: iterate registered substrates, dispatch to first capable one
- [ ] 🔴 Fallback: if no substrate capable, return explicit error — never silent failure

### 1.5 Tests
- [ ] 🔴 Unit tests for version handshake — compatible, incompatible, future version
- [ ] 🔴 Unit tests for plugin registration — register, deregister, duplicate, invalid
- [ ] 🔴 Integration test — CPU substrate end-to-end
- [ ] 🟡 Compile tests — confirm `no_std` enforced across all core crates
- [ ] 🟡 Miri run — undefined behaviour check on core ABI types

---

## Phase 2 — First Real Substrate `v0.3.0`
> SOKR runs real GPU workloads. The plugin model is proven.

### 2.1 SPIR-V IR Plugin (`sokr-spirv`)
- [ ] 🔴 Scaffold `sokr-spirv` crate
- [ ] 🔴 Accept SPIR-V binary as computation representation
- [ ] 🔴 Validate SPIR-V at Capability query time
- [ ] 🟡 SPIR-V reflection — extract workgroup size, bindings, entry points

### 2.2 Vulkan Substrate Plugin (`sokr-vulkan`)
- [ ] 🔴 Scaffold `sokr-vulkan` crate
- [ ] 🔴 Implement `Capability` — query Vulkan device for compute support
- [ ] 🔴 Implement `Dispatch` — submit SPIR-V compute shader via `ash`
- [ ] 🔴 Implement `Completion` — Vulkan fence / semaphore signal
- [ ] 🔴 Multi-device support — enumerate and register all available Vulkan devices
- [ ] 🟡 Memory management — host-visible buffer allocation, staging buffers
- [ ] 🟡 Pipeline caching — reuse compiled pipelines across dispatches

### 2.3 Rust Language Binding
- [ ] 🔴 Ergonomic Rust API over the C ABI core
- [ ] 🔴 `ComputeContext` — safe Rust wrapper for plugin registry
- [ ] 🔴 `Kernel` — safe Rust wrapper for a dispatchable computation unit
- [ ] 🟡 Builder pattern for dispatch configuration

### 2.4 Benchmarks
- [ ] 🔴 Benchmark harness — `criterion` based
- [ ] 🔴 Baseline: same workload on CPU fallback vs Vulkan GPU
- [ ] 🟡 Compare SOKR-Vulkan against raw `ash` dispatch overhead

---

## Phase 3 — Ecosystem `v0.4.0 – v0.9.0`
> Multiple substrates. Python bindings. Performance-aware dispatch.

### 3.1 CUDA Substrate Plugin (`sokr-cuda`)
- [ ] 🟡 PTX IR plugin (`sokr-ptx`)
- [ ] 🟡 CUDA substrate via `cust` crate
- [ ] 🟡 NVIDIA GPU enumeration and capability query
- [ ] 🟡 Async dispatch and stream-based completion

### 3.2 Metal Substrate Plugin (`sokr-metal`)
- [ ] 🟡 Metal compute pipeline via `metal-rs`
- [ ] 🟡 Apple Silicon unified memory path

### 3.3 Python Bindings (`sokr-python`)
- [ ] 🟡 PyO3 binding crate scaffold
- [ ] 🟡 `sokr.register()`, `sokr.dispatch()`, `sokr.await_completion()`
- [ ] 🟡 PyPI publish pipeline

### 3.4 WebGPU Substrate Plugin (`sokr-webgpu`)
- [ ] 🟡 `wgpu` backend
- [ ] 🟡 WASM compilation target
- [ ] 🟡 `wasm-bindgen` JavaScript API

### 3.5 Performance Dispatch Policy (`sokr-dispatch-perf`)
- [ ] 🟡 Per-substrate performance profile database
- [ ] 🟡 Route based on historical dispatch latency per workload class
- [ ] 🟡 Profile persistence across sessions

### 3.6 C Headers
- [ ] 🟡 `cbindgen` generated `sokr.h`
- [ ] 🟡 C example: register plugin, dispatch, completion
- [ ] 🟡 C++ example: RAII wrapper over C ABI

---

## Phase 4 — Future Substrates `v1.x`
> QPU, Neuromorphic, Photonic. The horizon.

### 4.1 QPU Substrate Plugin (`sokr-qpu`)
- [ ] 🟢 OpenQASM 3 IR plugin (`sokr-openqasm`)
- [ ] 🟢 IBM Quantum backend via Qiskit Runtime REST API
- [ ] 🟢 Completion model: measurement collapse signal
- [ ] 🟢 Capability: qubit count, circuit depth, gate set query

### 4.2 Neuromorphic Substrate Plugin (`sokr-neuro`)
- [ ] 🟢 Spike graph IR representation
- [ ] 🟢 Intel Loihi backend (via LAVA framework bridge)
- [ ] 🟢 Completion model: convergence signal

### 4.3 Photonic Substrate Plugin (`sokr-photon`)
- [ ] 🟢 Optical circuit IR representation
- [ ] 🟢 Lightmatter backend (pending public API)
- [ ] 🟢 Completion model: photon detection signal

### 4.4 Sovereign IR (`sokr-ir`)
- [ ] 🟢 SOKR-native IR specification — substrate-agnostic computation graph
- [ ] 🟢 Compiler: SOKR-IR → SPIR-V
- [ ] 🟢 Compiler: SOKR-IR → PTX
- [ ] 🟢 Compiler: SOKR-IR → OpenQASM 3
- [ ] 🟢 Compiler: SOKR-IR → spike graph

---

## SemVer Policy

| Version | Meaning |
|---|---|
| `0.1.x` | Name reserved. No API. |
| `0.2.x` | Core ABI draft. Breaking changes expected. |
| `0.3.x` | First real substrate. Plugin interface stabilizing. |
| `0.4.x – 0.9.x` | Ecosystem expansion. API approaching stable. |
| `1.0.0` | Core ABI frozen. Plugin interface stable. |
| `1.x.x` | Backwards compatible additions only. |
| `2.0.0` | Core ABI breaking change. Major justification required. |

---

## Contribution Policy

- All contributions require DCO sign-off (`Signed-off-by:` in commit)
- Core ABI changes require RFC and community comment period
- Plugin contributions welcome without RFC — plugins are sovereign
- Copyright assigned to **The SOKR Project**
- License: **MIT OR Apache-2.0** — no exceptions

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
