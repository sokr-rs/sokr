# SOKR — References

> Curated references that inform SOKR's design decisions.
> Organized by relevance to core architectural concerns.

---

## Formal Foundations

### Automata & State Machines
- **Automata Theory** — formal basis for computation models that don't assume
  a specific execution substrate
- **Statecharts** (David Harel, 1987) — hierarchical state machines with
  well-defined semantics for concurrent and reactive systems
- **RTC — Run-to-Completion semantics** — execution model where a state
  machine processes one event completely before accepting the next.
  Relevant to SOKR's Completion signal contract.

### Formal Verification
- **seL4** — https://sel4.systems
  The only OS microkernel with a machine-checked formal proof of
  correctness and security enforcement. Capability-based design.
  The gold standard reference for what a sovereignty claim should
  aspire to — not just philosophical, but mathematically provable.
  Relevant to SOKR's long-term formal verification roadmap.

---

## Capability-Based Design

- **Capsicum** — https://www.cl.cam.ac.uk/research/security/capsicum
  Capability-based security model for UNIX. Fine-grained sandboxing
  via capability tokens. Informs SOKR's plugin isolation model.

- **cap-std** (Rust) — https://github.com/bytecodealliance/cap-std
  Capability-based filesystem and network access for Rust. Early
  concept predates WASI 0.2 complexity. Clean reference for
  capability design in Rust.

- **WASI 0.1** — original WebAssembly System Interface runtime concept.
  Clean capability-based design before committee consensus introduced
  complexity. The simplicity of 0.1 is a cautionary reference for
  SOKR — keep the core interface minimal.

- **WASI 0.2 / WIT / IDL** — https://github.com/WebAssembly/wasi
  Where WASI introduced the WebAssembly Interface Type (WIT) IDL.
  Cautionary reference: SOKR deliberately avoids IDL, schema
  generation, and code generation at the core layer for this reason.

---

## Existing Runtimes

### General Purpose
- **BEAM** (Erlang/Elixir runtime) — https://www.erlang.org
  Actor model runtime that is substrate-agnostic by design. Processes
  communicate via message passing with no shared memory. Different
  layer from SOKR but same principle: the computation model is
  independent of the execution substrate.

- **QNX** — https://blackberry.qnx.com
  POSIX-compatible microkernel OS with capability-limited message
  passing. Everything communicates via explicit IPC. Core is minimal,
  capabilities are declared. Closest OS-level analog to SOKR's design
  philosophy.

### WebAssembly
- **WASI** — https://wasi.dev
  WebAssembly System Interface. Runtime abstraction for WASM modules.
  SOKR's WebGPU substrate plugin will interact with WASM targets via
  this interface.

---

## Hardware Dispatch Models

### Minimal Dispatch Reference
- **RP2040 PIO** (Raspberry Pi Programmable I/O) —
  https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf
  Eight independent state machines programmable without touching the
  core processor. Each PIO program is a small instruction sequence
  dispatched to a state machine. Closest hardware analog to SOKR's
  three-function dispatch model — capability (does this SM support
  this program?), dispatch (load and run), completion (signal done).
  Relevant for SOKR's embedded/MCU substrate story.

---

## Compute Runtimes (Existing Landscape)

### GPU
- **CubeCL** — https://github.com/tracel-ai/cubecl
  Multi-platform high-performance Rust compute language with JIT
  compiler, automatic vectorization, and autotuning. Supports CUDA,
  WebGPU, HIP/ROCm, Metal, SPIR-V. Built around the GPU
  thread/workgroup model. Most mature Rust GPU compute project.
  SOKR is designed to sit below CubeCL as a substrate plugin layer.

- **wgpu** — https://github.com/gfx-rs/wgpu
  Cross-platform GPU abstraction in Rust over Vulkan, Metal, DX12,
  WebGPU. Targets the GPU thread/workgroup model. SOKR's Vulkan
  substrate plugin wraps `wgpu` or `ash` internally.

- **rust-cuda / rust-gpu** — https://github.com/Rust-GPU/rust-cuda
  Rust compiler backends for NVIDIA CUDA (via NVVM IR) and SPIR-V
  (via Vulkan). The Chimera demo proves write-once, run-anywhere GPU
  kernels in Rust. SOKR's CUDA and SPIR-V IR plugins build on this
  ecosystem.

- **ZLUDA** — https://github.com/vosen/ZLUDA
  Drop-in CUDA replacement for non-NVIDIA GPUs via PTX → HIP
  translation. Attacks CUDA vendor lock-in at the compatibility layer.
  Complementary to SOKR which attacks it at the contract layer.

### Quantum
- **CUDA-Q** (NVIDIA) — https://developer.nvidia.com/cuda-q
  Open-source QPU-agnostic platform for hybrid classical-quantum
  workloads. GPU + CPU + QPU from a single program. Built on CUDA,
  so vendor-locked to NVIDIA stack. SOKR's QPU substrate plugin
  targets the same problem space without the vendor dependency.

---

## Neuromorphic Hardware

- **Intel Loihi / Loihi 2** — https://www.intel.com/content/www/us/en/research/neuromorphic-computing.html
  Intel's neuromorphic research chip. Programmed via the LAVA
  framework. SOKR's `sokr-neuro` plugin targets this via LAVA bridge.

- **SpiNNaker** (University of Manchester) —
  https://www.manchester.ac.uk/research/groups/advanced-processor-technologies/projects/spinnaker
  Massively parallel neuromorphic architecture designed to simulate
  biological neural networks in real time. Event-driven, spike-based.

- **IBM TrueNorth** —
  https://research.ibm.com/publications/truenorth-design-and-tool-flow-of-a-65-mw-1-million-neuron-programmable-neurosynaptic-chip
  IBM's 1-million neuron neuromorphic chip. 65mW power consumption.
  Spike-based computation with no shared clock.

- **NIR — Neuromorphic Intermediate Representation** —
  https://github.com/neuromorphs/NIR
  A common IR for neuromorphic computations across 11 platforms.
  Python-based. Informs SOKR's `sokr-neuro` IR plugin design.
  Published in Nature Communications (2024).

---

## Photonic Compute

- **Lightmatter** — https://lightmatter.co
  Photonic computing hardware using light instead of electrons for
  matrix operations. Target for SOKR's future `sokr-photon` substrate
  plugin pending public SDK availability.

---

## Standards & Specifications

- **SPIR-V** — https://www.khronos.org/spir
  Khronos Group binary IR for GPU shaders and compute kernels.
  Vendor-neutral. Supported by Vulkan, OpenCL, OpenGL.
  SOKR's primary IR for GPU substrate plugins.

- **OpenQASM 3** — https://openqasm.com
  Open quantum assembly language. Standard IR for quantum circuits.
  SOKR's IR for QPU substrate plugins.

- **PTX** (Parallel Thread Execution) —
  https://docs.nvidia.com/cuda/parallel-thread-execution
  NVIDIA's virtual ISA for CUDA. SOKR's IR for CUDA substrate plugin.

---

## Community Feedback

References surfaced through community discussion that shaped
SOKR's design decisions:

- QNX / capability + message passing analogy → confirmed SOKR's
  three-function interface design
- WASI 0.1 vs 0.2 → confirmed no IDL in SOKR core
- seL4 → formal verification as long-term sovereignty proof target
- RP2040 PIO → minimal dispatch model as hardware analog
- Rabbit hole warning → confirmed scope constraint: core stays at
  three functions, all complexity delegated to plugins

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
