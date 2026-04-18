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

## Theoretical Foundations

SOKR's three-function interface is grounded in well-established
formal models, not invented from scratch.

**Run-to-Completion (RTC) semantics** — the Completion signal maps
directly to RTC: a computation runs to completion before the next
event is accepted. The core makes no assumption about how long
completion takes — microseconds for CPU, seconds for QPU — but the
contract is the same.

**Automata and Statecharts** — every substrate plugin is a state
machine. The three-function interface defines the transitions:
`Capability` (can I accept this input?), `Dispatch` (transition to
running state), `Completion` (transition to terminal state). The core
does not know or care what happens inside the state machine.

**Capability-based security** — the plugin vtable is a capability
token. Possessing a registered `substrate_id` is the only permission
needed to dispatch to that substrate. No ambient authority, no global
state, no implicit permissions. Directly informed by seL4's capability
model and Capsicum's UNIX capability design.

---

## Design Lineage

SOKR does not exist in a vacuum. These prior systems directly
informed specific design decisions:

**QNX** — microkernel with POSIX compatibility and capability-limited
message passing. Lesson: keep the core minimal, make capabilities
explicit, route everything through a defined interface. The
three-function vtable is SOKR's message-passing contract.

**seL4** — the only formally verified capability-based microkernel.
Lesson: sovereignty claims must eventually be provable, not just
philosophical. seL4 is the long-term target for what formal
verification of SOKR's core ABI contract should look like.

**WASI 0.1** — clean capability-based runtime before committee
consensus introduced complexity. Lesson: the original WASI design
was correct. WASI 0.2/WIT/IDL added schema generation and code
generation that SOKR deliberately avoids. No IDL in SOKR core.

**BEAM** — Erlang/Elixir runtime with actor model, substrate-agnostic
by design. Lesson: processes that communicate only via defined message
interfaces can be migrated across substrates transparently. SOKR
applies this principle at the compute kernel level.

**RP2040 PIO** — Raspberry Pi's Programmable I/O state machines,
programmable without touching the core processor. Lesson: the minimal
hardware dispatch model already exists in silicon — capability query
(does this PIO program fit this state machine?), dispatch (load and
run), completion (signal done). SOKR formalises this pattern in
software.

**CubeCL / wgpu / rust-cuda** — the current Rust GPU compute
ecosystem. Lesson: all three assume the GPU thread/workgroup model.
QPU, neuromorphic, and photonic compute require fundamentally
different models. SOKR sits below these runtimes as a substrate
plugin layer, not beside them as a competitor.

---

## Scope Constraint

The problem space SOKR addresses is vast — automata theory, formal
verification, neuromorphic computing, quantum information theory,
photonic circuits. Each is a legitimate rabbit hole.

SOKR avoids this by a strict rule: **the core has no opinions.**

The core has no opinions about:
- RTC vs preemptive execution
- Memory addressing models
- Thread/workgroup topology
- Error propagation strategies
- Security policy
- IR schema design

All of these are plugin concerns. The moment the core adopts an
opinion on any of them, it gains a constraint that cannot be removed
without a breaking change.

The three-function interface is the scope boundary. Everything outside
it is a plugin. This is not a limitation — it is the design.

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
