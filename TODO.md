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
- [x] 🔴 GitHub repo made public with README, ARCHITECTURE, TODO
  - [x] Push all foundation files to `sokr-rs/sokr`
  - [x] Verify `README.md` renders correctly on GitHub
  - [x] Verify `ARCHITECTURE.md` renders correctly on GitHub
  - [x] Verify `TODO.md` renders correctly on GitHub
  - [x] Change visibility: Settings → Change repository visibility → Public
- [x] 🔴 CONTRIBUTING.md — contribution guidelines, DCO sign-off requirement
  - [x] Write contributor onboarding section — how to set up local dev environment
  - [x] Write DCO sign-off requirement — every commit must carry `Signed-off-by:`
  - [x] Write plugin contribution path — no RFC required for plugins
  - [x] Write core ABI change path — RFC required, community comment period
  - [x] Write code style section — `rustfmt`, `clippy` must pass before PR
  - [x] Write commit message convention — conventional commits format
  - [x] Write copyright assignment clause — all contributions to The SOKR Project

### 0.2 Design Documents
- [x] 🔴 Core philosophy documented
- [x] 🔴 Three-function interface defined: Capability, Dispatch, Completion
- [x] 🔴 Plugin categories defined: IR, Substrate, Language Binding, Dispatch Policy
- [x] 🔴 IR hybrid strategy documented
- [x] 🔴 Architecture layering documented
- [x] 🔴 C ABI surface specification — formal definition of types and function signatures
  - [x] Define `SokrVersion` struct — `{ major: u32, minor: u32, patch: u32 }`
  - [x] Define `SokrResult` enum — `Ok`, `CapabilityDenied`, `DispatchFailed`, `Timeout`, `VersionMismatch`, `NoCapableSubstrate`, `InvalidInput`, `InvalidIR`, `NotFound`, `RegistryFull`
  - [x] Define `SokrComputationId` — opaque 128-bit identifier for a computation unit
  - [x] Define `SokrCapabilityQuery` struct — `{ computation_id, ir_format, ir_data_ptr, ir_data_len }`
  - [x] Define `SokrCapabilityResponse` struct — `{ result, substrate_id, estimated_latency_ns }`
  - [x] Define `SokrDispatchRequest` struct — `{ computation_id, substrate_id, ir_data_ptr, ir_data_len, params_ptr, params_len }`
  - [x] Define `SokrDispatchResponse` struct — `{ result, completion_token }`
  - [x] Define `SokrCompletionToken` — opaque 64-bit handle
  - [x] Define `SokrCompletionQuery` struct — `{ completion_token, timeout_ns }`
  - [x] Define `SokrCompletionSignal` enum — `Pending`, `Complete`, `Failed`, `TimedOut`
  - [x] Define `SokrSubstratePlugin` vtable — `{ version, capability_fn, dispatch_fn, completion_fn, destroy_fn }`
  - [x] Write `#[repr(C)]` and padding rules for all structs
  - [x] Write null pointer handling rules — all pointer fields documented
  - [x] Write thread safety contract — which functions are safe to call concurrently
  - [x] Write ownership semantics — who allocates, who frees, for each field
- [x] 🔴 Version handshake protocol — specification for plugin compatibility negotiation
  - [x] Define compatibility rules — `major` must match, plugin `minor` must be ≤ core `minor`
  - [x] Define negotiation sequence — core sends version, plugin responds with its version
  - [x] Define rejection behaviour — incompatible plugin returns `VersionMismatch`, never panics
  - [x] Define forward compatibility contract — newer plugin on older core behaviour
  - [x] Document version bump triggers — what constitutes a major vs minor vs patch change
- [ ] 🟡 Plugin interface RFC — open for community comment before v0.2.0 freeze
  - [x] Write RFC document in `docs/rfc/0001-plugin-interface.md`
  - [x] Open GitHub Discussion linking to RFC
  - [x] Set comment period: minimum 4 weeks before freeze
  - [ ] Incorporate feedback or document rationale for rejection

### 0.3 Tooling
- [x] 🔴 `cargo install cargo-audit` — security audit in CI
  - [x] Add `cargo audit` step to CI workflow
  - [x] Add `audit.toml` — ignore list for known false positives
  - [x] Set audit to fail on any `unmaintained` or `vulnerability` advisory
- [x] 🔴 GitHub Actions CI
  - [x] Create `.github/workflows/ci.yml`
    - [x] Trigger: `push` to `main`, all `pull_request` events
    - [x] Job: `check` — `cargo check --all-targets`
    - [x] Job: `test` — `cargo test --all-features`
    - [x] Job: `clippy` — `cargo clippy -- -D warnings`
    - [x] Job: `fmt` — `cargo fmt --check`
    - [x] Job: `audit` — `cargo audit`
    - [x] Job: `no_std` — build with `--target thumbv7m-none-eabi`
    - [x] Matrix: test on `stable`, `beta`, `nightly`
    - [x] Cache: `~/.cargo/registry`, `~/.cargo/git`, `target/`
  - [x] Add CI status badge to `README.md`
- [x] 🔴 `.github/ISSUE_TEMPLATE/`
  - [x] `bug_report.md` — reproduction steps, expected vs actual, SOKR version, OS, hardware
  - [x] `feature_request.md` — problem statement, proposed solution, alternatives considered
  - [x] `plugin_proposal.md` — substrate/IR/binding name, target hardware, maintainer commitment
  - [x] `config.yml` — disable blank issues, link to Discussions for questions
- [x] 🟡 `deny.toml` — license and dependency policy via `cargo-deny`
  - [x] Configure `[licenses]` — allow: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib
  - [x] Configure `[advisories]` — deny yanked crates
  - [x] Configure `[sources]` — only crates.io, deny git dependencies in core
- [x] 🟡 Dependabot configuration
  - [x] Create `.github/dependabot.yml`
  - [x] Configure `cargo` ecosystem — weekly updates
  - [x] Configure `github-actions` ecosystem — weekly updates
  - [x] Set PR limit: max 5 open Dependabot PRs at once

---

## Phase 1 — Core Skeleton `v0.2.0`
> The immutable core exists. One substrate works. Nothing is final.

### 1.1 Workspace Setup
- [x] 🔴 Initialize Cargo workspace at repo root
  - [x] Create root `Cargo.toml` with `[workspace]` members list
  - [x] Add `resolver = "2"`
  - [x] Add `[workspace.dependencies]` — pin shared dependency versions
  - [x] Add `[workspace.lints]` — shared clippy and rustc lint config
  - [x] Verify `cargo check --workspace` passes clean
- [x] 🔴 Workspace crate layout
  - [x] `crates/sokr-core/` — the immutable C ABI core
  - [x] `crates/sokr-cpu/` — CPU substrate plugin
  - [x] `crates/sokr-dispatch-first/` — first-capable dispatch policy
  - [x] `examples/` — integration examples
  - [x] `benches/` — benchmark harness
  - [x] `docs/` — specs, RFCs, design notes

### 1.2 Core ABI (`sokr-core`)
- [x] 🔴 Scaffold `crates/sokr-core/`
  - [x] `Cargo.toml` — `no_std`, no dependencies, `crate-type = ["rlib"]`¹
  - [x] `src/lib.rs` — `#![no_std]`, `#![forbid(unsafe_code)]` outside FFI boundary
  - [x] `src/types.rs` — all C ABI struct and enum definitions
  - [x] `src/registry.rs` — plugin registry implementation
  - [x] `src/ffi.rs` — `#[no_mangle] extern "C"` function exports
  - [x] `cbindgen.toml` — configuration for header generation

¹ `staticlib`/`cdylib` built on-demand: `cargo build --crate-type staticlib,cdylib`
- [x] 🔴 Implement `SokrVersion`
  - [x] Define `#[repr(C)] struct SokrVersion { major: u32, minor: u32, patch: u32 }`
  - [x] Implement `SokrVersion::CURRENT` — compiled-in version constant
  - [x] Implement `SokrVersion::check_compatible()` — compatibility check logic
  - [x] Unit test: compatible versions pass
  - [x] Unit test: major mismatch fails
  - [x] Unit test: plugin too new fails
  - [x] Unit test: patch difference does not affect compatibility
- [x] 🔴 Implement `SokrResult`
  - [x] Define `#[repr(C)] enum SokrResult` with all 10 variants
  - [x] Implement `is_ok()` and `is_err()` methods
  - [x] Unit test: `is_ok()` and `is_err()` behavior
- [x] 🔴 Implement `SokrCapabilityQuery` and `SokrCapabilityResponse`
  - [x] Define structs per ABI spec
  - [x] Validate pointer fields are non-null before use
  - [x] Unit test: null pointer returns `SokrResult::InvalidInput`
  - [x] Unit test: zero-length IR data returns `InvalidInput`
- [ ] 🔴 Implement `SokrDispatchRequest` and `SokrDispatchResponse`
  - [ ] Define structs per ABI spec
  - [ ] Define `SokrCompletionToken` — opaque 64-bit handle
  - [ ] Unit test: dispatch request round-trips through registry
- [ ] 🔴 Implement `SokrCompletionQuery` and `SokrCompletionSignal`
  - [ ] Define structs per ABI spec
  - [ ] Implement timeout logic — `timeout_ns = 0` means non-blocking poll
  - [ ] Unit test: completion signal all variants represented
- [ ] 🔴 Implement `SokrSubstratePlugin` vtable
  - [ ] Define `#[repr(C)] struct SokrSubstratePlugin` with function pointers
  - [ ] Validate all function pointers non-null at registration
  - [ ] Unit test: null vtable entry rejected at registration
- [ ] 🔴 Implement `sokr_capability()`
  - [ ] `#[no_mangle] pub extern "C" fn sokr_capability()`
  - [ ] Route to registered substrate plugin matching `substrate_id`
  - [ ] Return `CapabilityDenied` if no matching substrate registered
  - [ ] Unit test: routes to correct plugin
  - [ ] Unit test: unknown substrate returns `CapabilityDenied`
- [ ] 🔴 Implement `sokr_dispatch()`
  - [ ] `#[no_mangle] pub extern "C" fn sokr_dispatch()`
  - [ ] Validate all dispatch request fields before routing
  - [ ] Route to substrate plugin
  - [ ] Return `completion_token` on success
  - [ ] Unit test: dispatch to registered plugin succeeds
  - [ ] Unit test: dispatch to unregistered plugin fails explicitly
- [ ] 🔴 Implement `sokr_completion()`
  - [ ] `#[no_mangle] pub extern "C" fn sokr_completion()`
  - [ ] Look up completion token in active dispatch table
  - [ ] Return `Pending`, `Complete`, or `Failed`
  - [ ] Unit test: completion after dispatch returns `Complete`
  - [ ] Unit test: unknown token returns `Failed`
- [ ] 🔴 `cbindgen` header generation
  - [ ] Configure `cbindgen.toml` — language: C, style: C, include guards
  - [ ] Add `cargo xtask generate-headers` command
  - [ ] Verify `sokr.h` compiles cleanly with `gcc -Wall -Wextra`
  - [ ] Verify `sokr.h` compiles cleanly with `clang -Wall -Wextra`
  - [ ] Commit generated `include/sokr.h` to repo
- [ ] 🔴 `no_std` enforcement
  - [ ] Add `#![no_std]` to `sokr-core/src/lib.rs`
  - [ ] Add CI job: build with `--target thumbv7m-none-eabi`
  - [ ] Verify no `std` sneaks in via transitive dependency

### 1.3 Plugin Registry
- [ ] 🔴 Plugin registration API
  - [ ] `sokr_register_substrate(plugin: *const SokrSubstratePlugin) -> SokrResult`
  - [ ] Validate plugin version compatibility on registration
  - [ ] Assign unique `substrate_id` to each registered plugin
  - [ ] Store in fixed-size static array — no heap allocation in core
  - [ ] Unit test: register one plugin succeeds, returns assigned id
  - [ ] Unit test: register beyond capacity returns `RegistryFull`
  - [ ] Unit test: register incompatible version returns `VersionMismatch`
  - [ ] Unit test: register with null pointer returns `InvalidInput`
- [ ] 🔴 Plugin deregistration API
  - [ ] `sokr_deregister_substrate(substrate_id: u32) -> SokrResult`
  - [ ] Call plugin's `destroy_fn` before removal
  - [ ] Mark slot as available for reuse
  - [ ] Unit test: deregister existing plugin succeeds
  - [ ] Unit test: deregister unknown id returns `NotFound`
  - [ ] Unit test: deregister then re-register in same slot works
- [ ] 🔴 Plugin version negotiation
  - [ ] Call `plugin.version_fn()` during registration
  - [ ] Compare against `SokrVersion::current()`
  - [ ] Reject if `major` differs
  - [ ] Accept if plugin `minor` ≤ core `minor`
- [ ] 🟡 Plugin registry introspection
  - [ ] `sokr_list_substrates(out: *mut u32, capacity: usize, count: *mut usize) -> SokrResult`
  - [ ] `sokr_describe_substrate(substrate_id: u32, out: *mut SokrSubstrateInfo) -> SokrResult`
  - [ ] Unit test: list returns all registered substrate IDs
  - [ ] Unit test: describe returns correct info for registered substrate

### 1.4 CPU Substrate Plugin (`sokr-cpu`)
- [ ] 🔴 Scaffold `crates/sokr-cpu/`
  - [ ] `Cargo.toml` — depends on `sokr-core`, `crate-type = ["staticlib", "cdylib"]`
  - [ ] `src/lib.rs` — implements `SokrSubstratePlugin` vtable
  - [ ] `src/capability.rs` — capability implementation
  - [ ] `src/dispatch.rs` — dispatch implementation
  - [ ] `src/completion.rs` — completion implementation
- [ ] 🔴 Implement `Capability`
  - [ ] Always return `SokrResult::Ok` — CPU can always attempt any computation
  - [ ] Set `estimated_latency_ns` to 0 (immediate)
  - [ ] Unit test: any query returns capable
  - [ ] Unit test: null query pointer returns `InvalidInput`
- [ ] 🔴 Implement `Dispatch`
  - [ ] Accept raw byte payload as computation unit
  - [ ] Execute synchronously on calling thread for v0.2.0
  - [ ] Store result in completion table keyed by `completion_token`
  - [ ] Generate unique `completion_token` per dispatch
  - [ ] Unit test: dispatch stores result retrievable via completion
  - [ ] Unit test: two concurrent dispatches get distinct tokens
- [ ] 🔴 Implement `Completion`
  - [ ] Look up `completion_token` in result table
  - [ ] Return `Complete` immediately — synchronous dispatch
  - [ ] Free result slot after `Complete` returned
  - [ ] Unit test: completion returns `Complete` after dispatch
  - [ ] Unit test: completion returns `Failed` for unknown token
  - [ ] Unit test: double-poll after `Complete` returns `Failed` (token consumed)
- [ ] 🔴 Integration test — full round-trip
  - [ ] Register `sokr-cpu` plugin with core
  - [ ] Query capability — assert `Ok`
  - [ ] Dispatch computation — assert `Ok` with valid token
  - [ ] Query completion — assert `Complete`
  - [ ] Deregister plugin — assert `Ok`
  - [ ] Verify no memory leak after deregistration
- [ ] 🟡 Benchmark baseline
  - [ ] Measure round-trip latency: register → capability → dispatch → completion
  - [ ] Record as baseline in `benches/RESULTS.md`

### 1.5 First Dispatch Policy Plugin (`sokr-dispatch-first`)
- [ ] 🔴 Scaffold `crates/sokr-dispatch-first/`
  - [ ] `Cargo.toml` — depends on `sokr-core`
  - [ ] `src/lib.rs` — dispatch policy implementation
- [ ] 🔴 Implement first-capable strategy
  - [ ] Iterate registered substrates in registration order
  - [ ] Call `sokr_capability()` on each
  - [ ] Dispatch to first substrate returning `Ok`
  - [ ] Unit test: single substrate — dispatches to it
  - [ ] Unit test: multiple substrates — dispatches to first capable
  - [ ] Unit test: second substrate capable, first not — dispatches to second
- [ ] 🔴 Explicit failure — never silent
  - [ ] Return `SokrResult::NoCapableSubstrate` — distinct from all other errors
  - [ ] Unit test: zero registered substrates returns `NoCapableSubstrate`
  - [ ] Unit test: all substrates deny capability returns `NoCapableSubstrate`

### 1.6 Tests
- [ ] 🔴 Unit tests for version handshake
  - [ ] `test_version_compatible_exact` — same version passes
  - [ ] `test_version_compatible_minor_older_plugin` — plugin minor < core minor passes
  - [ ] `test_version_incompatible_major_higher` — plugin major > core major fails
  - [ ] `test_version_incompatible_major_lower` — plugin major < core major fails
  - [ ] `test_version_compatible_future_minor` — plugin minor > core minor passes
  - [ ] `test_version_patch_irrelevant` — patch difference does not affect compatibility
- [ ] 🔴 Unit tests for plugin registration
  - [ ] `test_register_valid_plugin` — succeeds, returns assigned id
  - [ ] `test_register_null_vtable` — returns `InvalidInput`
  - [ ] `test_register_null_function_pointer` — returns `InvalidInput`
  - [ ] `test_register_incompatible_version` — returns `VersionMismatch`
  - [ ] `test_register_at_capacity` — returns `RegistryFull`
  - [ ] `test_deregister_valid` — succeeds
  - [ ] `test_deregister_invalid_id` — returns `NotFound`
  - [ ] `test_register_after_deregister` — slot reuse works
- [ ] 🔴 Integration test — CPU substrate end-to-end
  - [ ] Full round-trip as described in 1.4
- [ ] 🟡 Compile tests
  - [ ] `compiletest`: `no_std` — verify core does not compile with `std`
  - [ ] `compiletest`: unsafe — verify `#![forbid(unsafe_code)]` blocks unsafe in core
- [ ] 🟡 Miri run
  - [ ] Run `cargo miri test` on core ABI types
  - [ ] Verify no undefined behaviour in pointer handling
  - [ ] Add Miri job to CI — nightly only, allowed to fail

---

## Phase 2 — First Real Substrate `v0.3.0`
> SOKR runs real GPU workloads. The plugin model is proven.

### 2.1 SPIR-V IR Plugin (`sokr-spirv`)
- [ ] 🔴 Scaffold `crates/sokr-spirv/`
  - [ ] `Cargo.toml` — depends on `sokr-core`, `spirv-tools` for validation
  - [ ] `src/lib.rs` — IR plugin implementation
  - [ ] `src/validate.rs` — SPIR-V binary validation
  - [ ] `src/reflect.rs` — workgroup size, binding, entry point extraction
- [ ] 🔴 Accept SPIR-V binary
  - [ ] Register IR format identifier: `SOKR_IR_SPIRV = 0x53505256`
  - [ ] Validate magic number `0x07230203` at capability query time
  - [ ] Return `SokrResult::InvalidIR` if magic number absent
  - [ ] Unit test: valid SPIR-V binary accepted
  - [ ] Unit test: invalid magic number rejected
  - [ ] Unit test: empty payload rejected
- [ ] 🔴 Validate SPIR-V at capability query time
  - [ ] Run `spirv-val` validation pass
  - [ ] Return `InvalidIR` with description if validation fails
  - [ ] Unit test: valid compute shader passes validation
  - [ ] Unit test: invalid shader returns `InvalidIR`
- [ ] 🟡 SPIR-V reflection
  - [ ] Extract `LocalSize` execution mode — workgroup dimensions
  - [ ] Extract descriptor bindings — set, binding, type
  - [ ] Extract entry point names
  - [ ] Expose via `sokr_spirv_reflect()` C function
  - [ ] Unit test: reflection matches known shader metadata

### 2.2 Vulkan Substrate Plugin (`sokr-vulkan`)
- [ ] 🔴 Scaffold `crates/sokr-vulkan/`
  - [ ] `Cargo.toml` — depends on `sokr-core`, `ash`, `gpu-allocator`
  - [ ] `src/lib.rs` — substrate plugin entry point
  - [ ] `src/device.rs` — Vulkan device enumeration and selection
  - [ ] `src/pipeline.rs` — compute pipeline creation and caching
  - [ ] `src/memory.rs` — buffer allocation and data transfer
  - [ ] `src/dispatch.rs` — command buffer recording and submission
  - [ ] `src/completion.rs` — fence and semaphore management
- [ ] 🔴 Implement `Capability`
  - [ ] Enumerate Vulkan physical devices via `vkEnumeratePhysicalDevices`
  - [ ] Check `VK_QUEUE_COMPUTE_BIT` on at least one queue family
  - [ ] Check `VkPhysicalDeviceFeatures` for required features
  - [ ] Return `Ok` if capable, `CapabilityDenied` if not
  - [ ] Unit test: mock device with compute queue returns capable
  - [ ] Unit test: mock device without compute queue returns denied
- [ ] 🔴 Implement `Dispatch`
  - [ ] Create `VkShaderModule` from SPIR-V binary
  - [ ] Create `VkPipelineLayout` and `VkComputePipeline`
  - [ ] Allocate descriptor sets for input/output buffers
  - [ ] Record `vkCmdDispatch` in command buffer
  - [ ] Submit to compute queue
  - [ ] Return `completion_token` mapped to submitted fence
  - [ ] Unit test: dispatch of valid SPIR-V succeeds
  - [ ] Unit test: dispatch of invalid SPIR-V returns `DispatchFailed`
- [ ] 🔴 Implement `Completion`
  - [ ] Poll `vkGetFenceStatus` for `completion_token`
  - [ ] Respect `timeout_ns` — use `vkWaitForFences` with timeout
  - [ ] Return `Pending`, `Complete`, or `TimedOut`
  - [ ] Cleanup fence and pipeline after `Complete`
  - [ ] Unit test: completion after dispatch returns `Complete`
  - [ ] Unit test: completion with zero timeout returns `Pending` or `Complete`
- [ ] 🔴 Multi-device support
  - [ ] Register each physical device as separate substrate plugin instance
  - [ ] Include device name and vendor ID in capability response
  - [ ] Unit test: two physical devices register as two substrate IDs
- [ ] 🟡 Memory management
  - [ ] Host-visible staging buffer for input data upload
  - [ ] Device-local buffer for compute
  - [ ] Readback buffer for result download
  - [ ] Use `gpu-allocator` for sub-allocation
  - [ ] Unit test: data survives upload → compute → readback round-trip
- [ ] 🟡 Pipeline caching
  - [ ] Create `VkPipelineCache` at plugin init
  - [ ] Reuse cached pipeline if same SPIR-V hash seen before
  - [ ] Benchmark: pipeline cache hit vs miss latency

### 2.3 Rust Language Binding
- [ ] 🔴 `ComputeContext`
  - [ ] `pub struct ComputeContext` — safe Rust wrapper around plugin registry
  - [ ] `ComputeContext::new()` — initialise core, register CPU fallback by default
  - [ ] `ComputeContext::register_substrate()` — safe wrapper around `sokr_register_substrate`
  - [ ] `ComputeContext::deregister_substrate()` — safe wrapper
  - [ ] `impl Drop for ComputeContext` — deregister all substrates on drop
  - [ ] Unit test: context drops cleanly with registered plugins
  - [ ] Unit test: double-drop does not panic
- [ ] 🔴 `Kernel`
  - [ ] `pub struct Kernel` — wraps a computation unit (IR bytes + metadata)
  - [ ] `Kernel::from_spirv(bytes: &[u8]) -> Result<Kernel, SokrError>`
  - [ ] `Kernel::dispatch(&self, ctx: &ComputeContext) -> Result<CompletionHandle, SokrError>`
  - [ ] Unit test: kernel from valid SPIR-V succeeds
  - [ ] Unit test: kernel from invalid bytes returns error
  - [ ] Unit test: dispatch returns valid handle
- [ ] 🔴 `CompletionHandle`
  - [ ] `pub struct CompletionHandle` — wraps `SokrCompletionToken`
  - [ ] `CompletionHandle::wait(timeout: Option<Duration>) -> Result<(), SokrError>`
  - [ ] `CompletionHandle::poll() -> CompletionStatus`
  - [ ] Unit test: wait on CPU substrate completes immediately
  - [ ] Unit test: poll before dispatch returns `Pending`
- [ ] 🟡 Builder pattern
  - [ ] `KernelBuilder` — configure workgroup size, push constants, buffer bindings
  - [ ] `DispatchConfig` — override substrate selection, set timeout
  - [ ] Unit test: builder produces equivalent kernel to direct construction

### 2.4 Benchmarks
- [ ] 🔴 Benchmark harness setup
  - [ ] Add `criterion` to workspace dev-dependencies
  - [ ] Create `benches/dispatch_latency.rs`
  - [ ] Create `benches/throughput.rs`
  - [ ] Create `benches/RESULTS.md` — record methodology and results
- [ ] 🔴 CPU vs Vulkan comparison
  - [ ] Benchmark: array addition — 1M elements, CPU vs Vulkan
  - [ ] Benchmark: matrix multiply — 512×512, CPU vs Vulkan
  - [ ] Record baseline results in `benches/RESULTS.md`
- [ ] 🟡 SOKR overhead vs raw dispatch
  - [ ] Benchmark: raw `ash` Vulkan dispatch vs SOKR-Vulkan dispatch
  - [ ] Target: SOKR overhead < 5% over raw `ash`
  - [ ] Document methodology in `benches/RESULTS.md`

---

## Phase 3 — Ecosystem `v0.4.0 – v0.9.0`
> Multiple substrates. Python bindings. Performance-aware dispatch.

### 3.1 CUDA Substrate Plugin (`sokr-cuda`)
- [ ] 🟡 PTX IR plugin (`sokr-ptx`)
  - [ ] Register IR format identifier: `SOKR_IR_PTX = 0x50545800`
  - [ ] Validate PTX magic string `.version` at query time
  - [ ] Unit test: valid PTX accepted, invalid rejected
- [ ] 🟡 CUDA substrate via `cust` crate
  - [ ] Scaffold `crates/sokr-cuda/`
  - [ ] Enumerate CUDA devices via `cuDeviceGetCount`
  - [ ] Implement `Capability` — check compute capability version
  - [ ] Implement `Dispatch` — `cuModuleLoadData` + `cuLaunchKernel`
  - [ ] Implement `Completion` — `cuStreamSynchronize` with timeout
  - [ ] Unit test: PTX vector addition runs on CUDA device
- [ ] 🟡 NVIDIA GPU enumeration
  - [ ] Register each CUDA device as separate substrate
  - [ ] Include device name, VRAM, compute capability in capability response
- [ ] 🟡 Async dispatch
  - [ ] Use CUDA streams for non-blocking dispatch
  - [ ] Map stream event to `completion_token`
  - [ ] Unit test: async dispatch + poll completion works

### 3.2 Metal Substrate Plugin (`sokr-metal`)
- [ ] 🟡 Scaffold `crates/sokr-metal/` — `cfg(target_os = "macos")` only
  - [ ] `Cargo.toml` — depends on `metal-rs`
  - [ ] Implement `Capability` — check Metal device supports compute
  - [ ] Implement `Dispatch` — `MTLComputeCommandEncoder` + `MTLLibrary`
  - [ ] Implement `Completion` — `MTLCommandBuffer` completion handler
  - [ ] Unit test: Metal compute dispatch on Apple Silicon
- [ ] 🟡 Unified memory path
  - [ ] Detect Apple Silicon shared memory architecture
  - [ ] Skip staging buffer when CPU and GPU share memory
  - [ ] Benchmark: unified memory vs staged transfer on M-series

### 3.3 Python Bindings (`sokr-python`)
- [ ] 🟡 Scaffold `crates/sokr-python/`
  - [ ] `Cargo.toml` — depends on `sokr-core`, `pyo3`, feature `extension-module`
  - [ ] `src/lib.rs` — PyO3 module definition
  - [ ] `src/context.rs` — Python `ComputeContext` class
  - [ ] `src/kernel.rs` — Python `Kernel` class
- [ ] 🟡 `sokr.register(plugin_path: str)`
  - [ ] Load substrate plugin from shared library path
  - [ ] Register with core
  - [ ] Raise `SokrError` on failure
- [ ] 🟡 `sokr.dispatch(kernel: Kernel) -> CompletionHandle`
  - [ ] Accept `bytes` or `numpy.ndarray` as kernel payload
  - [ ] Return Python `CompletionHandle` object
- [ ] 🟡 `handle.wait(timeout_ms: int = 0)`
  - [ ] Block until complete or timeout
  - [ ] Raise `SokrTimeoutError` on timeout
- [ ] 🟡 PyPI publish pipeline
  - [ ] `maturin` build configuration
  - [ ] GitHub Actions: build wheels for Linux, macOS, Windows
  - [ ] Publish to PyPI on tag push

### 3.4 WebGPU Substrate Plugin (`sokr-webgpu`)
- [ ] 🟡 Scaffold `crates/sokr-webgpu/` — WASM-compatible
  - [ ] `Cargo.toml` — depends on `wgpu`
  - [ ] Implement `Capability` — query `wgpu::Adapter` for compute support
  - [ ] Implement `Dispatch` — `wgpu::ComputePass` submission
  - [ ] Implement `Completion` — map `wgpu::BufferAsyncError` callback
- [ ] 🟡 WASM compilation target
  - [ ] Add `wasm32-unknown-unknown` to CI matrix
  - [ ] Verify `sokr-webgpu` compiles to WASM
  - [ ] `wasm-pack build` produces valid npm package
- [ ] 🟡 JavaScript API via `wasm-bindgen`
  - [ ] `sokr.init()` — async, initialises WebGPU adapter
  - [ ] `sokr.dispatch(spirv: Uint8Array) -> Promise<Uint8Array>`
  - [ ] Publish to npm as `@sokr/webgpu`

### 3.5 Performance Dispatch Policy (`sokr-dispatch-perf`)
- [ ] 🟡 Scaffold `crates/sokr-dispatch-perf/`
- [ ] 🟡 Per-substrate latency profile
  - [ ] Record actual dispatch + completion latency per substrate per IR format
  - [ ] Store in fixed-size ring buffer — no heap allocation
- [ ] 🟡 Profile-aware routing
  - [ ] Route to substrate with lowest historical latency for this IR format
  - [ ] Fall back to first-capable if no profile data exists
  - [ ] Unit test: routes to faster substrate after profiling
- [ ] 🟡 Profile persistence
  - [ ] Serialize profiles to flat binary file
  - [ ] Load on plugin init, save on plugin destroy
  - [ ] Unit test: profiles survive plugin restart

### 3.6 C Headers
- [ ] 🟡 Finalise `sokr.h`
  - [ ] Run `cbindgen` — verify no drift from hand-spec
  - [ ] Add doxygen-style comments to all exported types and functions
  - [ ] Verify with `gcc -Wall -Wextra -Werror`
  - [ ] Verify with `clang -Wall -Wextra -Werror`
- [ ] 🟡 C example
  - [ ] `examples/c/hello_compute.c` — register CPU plugin, dispatch, completion
  - [ ] `examples/c/Makefile` — build with `gcc` and `clang`
  - [ ] Add to CI: build and run C example
- [ ] 🟡 C++ RAII wrapper
  - [ ] `include/sokr.hpp` — `SokrContext`, `SokrKernel`, `SokrFuture` RAII classes
  - [ ] `examples/cpp/hello_compute.cpp`
  - [ ] Verify with C++17 and C++20

---

## Phase 4 — Future Substrates `v1.x`
> QPU, Neuromorphic, Photonic. The horizon.

### 4.1 QPU Substrate Plugin (`sokr-qpu`)
- [ ] 🟢 OpenQASM 3 IR plugin (`sokr-openqasm`)
  - [ ] Register IR format identifier: `SOKR_IR_OPENQASM3 = 0x4F51334D`
  - [ ] Parse OpenQASM 3 header for version validation
  - [ ] Unit test: valid OpenQASM 3 program accepted
  - [ ] Unit test: OpenQASM 2 program rejected with clear error
- [ ] 🟢 IBM Quantum backend
  - [ ] Authenticate via Qiskit Runtime REST API
  - [ ] Implement `Capability` — query available backends, qubit count, gate set
  - [ ] Implement `Dispatch` — submit job via REST, return job ID as token
  - [ ] Implement `Completion` — poll job status endpoint until terminal state
  - [ ] Unit test: mock REST backend round-trip
- [ ] 🟢 Capability metadata
  - [ ] Qubit count, T1/T2 coherence times, gate error rates in capability response
  - [ ] Unit test: capability response carries hardware metadata

### 4.2 Neuromorphic Substrate Plugin (`sokr-neuro`)
- [ ] 🟢 Spike graph IR
  - [ ] Define spike graph binary format — nodes, synapses, timing constraints
  - [ ] Register IR format identifier: `SOKR_IR_SPIKE = 0x53504B45`
  - [ ] Validator: check node count, synapse count, timing constraints
  - [ ] Unit test: valid spike graph accepted, over-limit graph rejected
- [ ] 🟢 Intel Loihi backend via LAVA bridge
  - [ ] Implement `Capability` — check LAVA SDK available, Loihi device present
  - [ ] Implement `Dispatch` — submit spike graph to LAVA runtime
  - [ ] Implement `Completion` — convergence signal from LAVA callback
- [ ] 🟢 Convergence completion model
  - [ ] Define convergence criteria in capability query
  - [ ] Support streaming partial results before full convergence

### 4.3 Photonic Substrate Plugin (`sokr-photon`)
- [ ] 🟢 Optical circuit IR
  - [ ] Define optical circuit binary format
  - [ ] Register IR format identifier: `SOKR_IR_OPTICAL = 0x4F50544C`
  - [ ] Validator: check gate set compatibility with target device
- [ ] 🟢 Lightmatter backend
  - [ ] Pending public SDK/API availability
  - [ ] Stub plugin with `Capability` returning `CapabilityDenied` until SDK ships
- [ ] 🟢 Photon detection completion model
  - [ ] Map photon measurement events to `SokrCompletionSignal`
  - [ ] Handle probabilistic outputs — partial measurement results

### 4.4 Sovereign IR (`sokr-ir`)
- [ ] 🟢 SOKR-native IR specification
  - [ ] Define substrate-agnostic computation graph format
  - [ ] Version IR format independently of SOKR core
  - [ ] Publish specification as `docs/sokr-ir-spec.md`
  - [ ] Open spec for public comment before implementation begins
- [ ] 🟢 Compiler: SOKR-IR → SPIR-V
  - [ ] Graph lowering pass — computation graph to SPIR-V kernel
  - [ ] Type mapping — SOKR-IR types to SPIR-V types
  - [ ] Integration test: SOKR-IR program produces correct SPIR-V output
- [ ] 🟢 Compiler: SOKR-IR → PTX
  - [ ] Graph lowering pass — computation graph to PTX
  - [ ] Integration test: SOKR-IR program produces correct PTX output
- [ ] 🟢 Compiler: SOKR-IR → OpenQASM 3
  - [ ] Map classical compute nodes to quantum gate equivalents
  - [ ] Integration test: SOKR-IR program produces valid OpenQASM 3
- [ ] 🟢 Compiler: SOKR-IR → spike graph
  - [ ] Map tensor operations to spike timing encodings
  - [ ] Integration test: SOKR-IR program produces valid spike graph

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
