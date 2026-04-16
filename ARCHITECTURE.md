# SOKR Architecture

> Sovereign Open Kernel Runtime

---

## Guiding Principle

**Immutable sovereign core. Everything else a plugin.**

The core contains no assumptions — not about computation representation,
not about hardware model, not about language, not about security policy.
Any assumption baked into the core is a future constraint that no plugin
can override. The core must be immune to churn at both ends.

---

## The Problem SOKR Solves

Every existing compute runtime — CUDA, ROCm, Metal, WebGPU — binds the
algorithm to a substrate. When the substrate changes (new hardware
generation, export restriction, vendor decision), the algorithm breaks.

SOKR inverts this. The algorithm is the permanent asset. The substrate
is a runtime decision.

> The contract between algorithm and hardware, owned by no one.

---

## Core Interface

The SOKR core exposes exactly three operations over a stable, versioned
C ABI surface:

```
Capability  →  Can this substrate fulfill this computation?
Dispatch    →  Fulfill it.
Completion  →  Signal when fulfilled.
```

Nothing else crosses the core boundary. Every other concern —
memory layout, parallelism model, error handling, security policy —
is the plugin's internal problem.

A version handshake is embedded in the Capability query from v0.1.0
onwards. This ensures the core ABI never silently breaks existing plugins.

---

## Layered Architecture

```
┌─────────────────────────────────────────┐
│         User Code (any language)        │
└────────────────────┬────────────────────┘
                     │
┌────────────────────▼────────────────────┐
│              IR Plugin                  │
│  swappable: SOKR-native, SPIR-V,        │
│  OpenQASM 3, or any future IR           │
└────────────────────┬────────────────────┘
                     │
┌────────────────────▼────────────────────┐
│             SOKR Core                   │
│         (immutable, no_std)             │
│                                         │
│   Capability → Dispatch → Completion    │
│         + version handshake             │
│                                         │
│         C ABI surface                   │
└────────────────────┬────────────────────┘
                     │
┌────────────────────▼────────────────────┐
│           Substrate Plugin              │
│  swappable: GPU, CPU, QPU,              │
│  Neuromorphic, Photonic, or future      │
└────────────────────┬────────────────────┘
                     │
┌────────────────────▼────────────────────┐
│              Hardware                   │
└─────────────────────────────────────────┘
```

Every layer above and below the core is independently swappable.
Removing a layer does not touch the core. Adding a new layer does
not require a core change.

---

## Plugin Categories

### IR Plugins
Translate user computation into a form the substrate plugin can accept.

| Plugin | IR Format | Status |
|---|---|---|
| `sokr-ir` | SOKR-native (TBD) | Planned |
| `sokr-spirv` | SPIR-V | Planned |
| `sokr-ptx` | PTX (NVIDIA) | Planned |
| `sokr-openqasm` | OpenQASM 3 | Future |

### Substrate Plugins
Fulfill computations on physical or virtual hardware.

| Plugin | Target | Status |
|---|---|---|
| `sokr-cpu` | CPU (fallback, always available) | Phase 1 |
| `sokr-vulkan` | Vulkan-compatible GPUs | Phase 2 |
| `sokr-cuda` | NVIDIA CUDA | Phase 2 |
| `sokr-metal` | Apple Metal | Phase 2 |
| `sokr-webgpu` | Browser / Edge | Phase 3 |
| `sokr-qpu` | Quantum processors | Future |
| `sokr-neuro` | Neuromorphic hardware | Future |
| `sokr-photon` | Photonic compute | Future |

### Language Binding Plugins
Expose SOKR to language ecosystems.

| Plugin | Language | Mechanism |
|---|---|---|
| `sokr` (this crate) | Rust | native |
| `sokr-c` | C / C++ | `cbindgen` headers |
| `sokr-python` | Python | PyO3 |
| `sokr-wasm` | JavaScript / Browser | `wasm-bindgen` |
| `sokr-java` | JVM | `jni-rs` |

### Dispatch Policy Plugins
Decide which substrate handles which computation at runtime.

| Plugin | Strategy | Status |
|---|---|---|
| `sokr-dispatch-first` | First capable substrate wins | Phase 1 |
| `sokr-dispatch-perf` | Performance-profile-aware routing | Phase 3 |
| `sokr-dispatch-cost` | Cost-aware routing (cloud context) | Future |

---

## Substrate Compatibility Matrix

SOKR makes no assumption about how a substrate computes.
The three-function interface works across fundamentally different models:

| Substrate | Memory | Parallelism | Completion |
|---|---|---|---|
| GPU | addressable | SIMT threads | sync / async |
| CPU | addressable | OS threads | immediate |
| QPU | quantum state | superposition | measurement collapse |
| Neuromorphic | sparse events | spike timing | convergence signal |
| Photonic | optical circuit | waveguide | photon detection |

The plugin, not the core, handles the mapping.

---

## IR Strategy

SOKR uses a hybrid IR model:

- **SOKR-native IR** — high-level, substrate-agnostic. Users who want
  full portability write to this. The plugin translates down.
- **Direct passthrough** — users who need maximum performance or
  hardware-specific features pass substrate-native IR directly.
  The plugin accepts it without translation overhead.

The IR plugin declares at Capability query time which formats it accepts.
The core routes accordingly. The IR layer is itself swappable — a future
IR standard can replace SOKR-native without touching the core.

---

## Design Constraints

These constraints are invariants. Violating any of them is a breaking
change to SOKR's philosophy, not just its API.

1. **Core is `no_std`** — no OS dependency, no allocator assumption.
   SOKR must be deployable from MCU to datacenter.

2. **Core exposes C ABI only** — no Rust-specific types cross the
   plugin boundary. Any language, any runtime can implement a plugin.

3. **No assumption in core** — the core does not know what an IR is,
   what a GPU is, or what a thread is. These are plugin concerns.

4. **Version handshake is mandatory** — every plugin negotiates
   compatibility at load time. Silent ABI breaks are impossible by design.

5. **License imposes no conditions** — MIT OR Apache-2.0. No plugin
   author needs permission from anyone to build on SOKR.

---

## Non-Goals

- SOKR is not a shader language
- SOKR is not a ML framework
- SOKR is not a driver
- SOKR is not a replacement for CUDA in CUDA-specific workflows
- SOKR does not own the algorithm
- SOKR does not own the substrate

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
