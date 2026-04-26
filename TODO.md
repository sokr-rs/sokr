# SOKR — Development TODO

> Sovereign Open Kernel Runtime — Core Only
> Last Updated: 2026-04-20
> Legend: 🔴 Critical path · 🟡 Important · 🟢 Nice-to-have

---

## Vision

A sovereign compute runtime where the algorithm is the permanent asset
and the substrate is a runtime decision — for hardware that exists today
and hardware that does not yet exist.

**This repo is the core only.** Plugin development lives at
[sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins).

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
- [x] 🔴 GitHub repo made public: `sokr-rs/sokr`
- [x] 🔴 CONTRIBUTING.md complete
- [x] 🔴 Repo restructure: flatten from workspace to single crate
  - [x] Move `crates/sokr-core/src/` → `src/`
  - [x] Move `crates/sokr-core/Cargo.toml` → root `Cargo.toml`, rename to `sokr`
  - [x] Move `crates/sokr-core/cbindgen.toml` → root `cbindgen.toml`
  - [x] Remove `crates/sokr-cpu/`, `crates/sokr-dispatch-first/` — move to `sokr-plugins` repo
  - [x] Remove workspace `[workspace]` section from root `Cargo.toml`
  - [x] Verify `cargo check` passes on flattened structure
  - [x] Verify `cargo test` passes on flattened structure
  - [x] Update CI workflow — remove `--workspace` flags
  - [x] Publish `sokr` v0.1.1 from new structure → https://crates.io/crates/sokr/0.1.1

### 0.2 Design Documents
- [x] 🔴 Core philosophy documented
- [x] 🔴 Three-function interface defined: Capability, Dispatch, Completion
- [x] 🔴 Plugin categories defined: IR, Substrate, Language Binding, Dispatch Policy
- [x] 🔴 IR hybrid strategy documented
- [x] 🔴 Architecture layering documented
- [x] 🔴 C ABI surface specification complete
- [x] 🔴 Version handshake protocol complete
- [ ] 🟡 Plugin interface RFC — open for community comment before v0.2.0 freeze
  - [x] Write RFC document in `docs/rfc/0001-plugin-interface.md`
  - [x] Open GitHub Discussion: https://github.com/sokr-rs/sokr/discussions/2
  - [x] Set comment period: minimum 4 weeks (closes 2026-05-14)
  - [ ] Incorporate feedback or document rationale for rejection

### 0.3 Tooling
- [x] 🔴 GitHub Actions CI — check, test, clippy, fmt, audit, no_std
- [x] 🔴 `.github/ISSUE_TEMPLATE/` — bug, feature, plugin proposal
- [x] 🟡 `deny.toml` — license and dependency policy
- [x] 🟡 Dependabot — weekly cargo and github-actions updates

---

## Phase 1 — Core Skeleton `v0.2.0`
> The immutable core exists. ABI is complete. Version handshake works.

### 1.1 Repo Restructure
- [x] 🔴 Complete single-crate flatten (see 0.1 Repo restructure above)
- [x] 🔴 Verify `sokr-plugins` repo exists and `sokr-cpu` moved there
- [x] 🔴 Update all internal references from `sokr-core` → `sokr`

### 1.2 Core ABI (`src/`)
- [x] 🔴 `src/types.rs` — all C ABI struct and enum definitions
- [x] 🔴 `src/registry.rs` — plugin registry, no heap allocation
- [x] 🔴 `src/ffi.rs` — `#[no_mangle] extern "C"` function stubs
- [x] 🔴 `SokrVersion` — `CURRENT` constant + `check_compatible()`
- [x] 🔴 `SokrResult` — 10 variants + `is_ok()` / `is_err()`
- [x] 🔴 All query/request/response/signal structs defined
- [x] 🔴 `SokrSubstratePlugin` vtable defined
- [x] 🔴 Implement `sokr_capability()` — route to registered substrate
  - [x] Route to all registered substrate plugins; first accepting plugin wins
  - [x] Return `CapabilityDenied` if no matching substrate registered
  - [x] Unit test: routes to correct plugin
  - [x] Unit test: unknown substrate returns `CapabilityDenied`
- [x] 🔴 Implement `sokr_dispatch()` — route to substrate and dispatch
  - [x] Validate all dispatch request fields before routing
  - [x] Route to substrate plugin by `substrate_id`
  - [x] Return `completion_token` on success
  - [x] Unit test: dispatch to registered plugin succeeds
  - [x] Unit test: dispatch to unregistered plugin fails explicitly
- [ ] 🔴 Implement `sokr_completion()` — poll completion token
  - [ ] Look up completion token in active dispatch table
  - [ ] Return `Pending`, `Complete`, or `Failed`
  - [ ] Unit test: completion after dispatch returns `Complete`
  - [ ] Unit test: unknown token returns `Failed`
- [ ] 🔴 `cbindgen` header generation
  - [ ] Add `cargo xtask generate-headers` command
  - [ ] Verify `sokr.h` compiles cleanly with `gcc -Wall -Wextra`
  - [ ] Verify `sokr.h` compiles cleanly with `clang -Wall -Wextra`
  - [ ] Commit generated `include/sokr.h` to repo

### 1.3 Plugin Registry
- [ ] 🔴 `sokr_register_substrate()` — register plugin with version check
  - [ ] Validate plugin version compatibility on registration
  - [ ] Assign unique `substrate_id` to each registered plugin
  - [ ] Store in fixed-size static array — no heap allocation
  - [ ] Unit test: register one plugin succeeds, returns assigned id
  - [ ] Unit test: register beyond capacity returns `RegistryFull`
  - [ ] Unit test: register incompatible version returns `VersionMismatch`
  - [ ] Unit test: register with null pointer returns `InvalidInput`
- [ ] 🔴 `sokr_deregister_substrate()` — deregister and call destroy_fn
  - [ ] Call plugin's `destroy_fn` before removal
  - [ ] Mark slot as available for reuse
  - [ ] Unit test: deregister existing plugin succeeds
  - [ ] Unit test: deregister unknown id returns `NotFound`
  - [ ] Unit test: deregister then re-register in same slot works
- [ ] 🟡 `sokr_list_substrates()` — introspection
  - [ ] Unit test: list returns all registered substrate IDs

### 1.4 Tests
- [ ] 🔴 Unit tests for version handshake
  - [ ] `test_version_compatible_exact`
  - [ ] `test_version_compatible_minor_older_plugin`
  - [ ] `test_version_incompatible_major_higher`
  - [ ] `test_version_incompatible_major_lower`
  - [ ] `test_version_patch_irrelevant`
- [ ] 🔴 Unit tests for plugin registration
  - [ ] `test_register_valid_plugin`
  - [ ] `test_register_null_vtable`
  - [ ] `test_register_incompatible_version`
  - [ ] `test_register_at_capacity`
  - [ ] `test_deregister_valid`
  - [ ] `test_deregister_invalid_id`
  - [ ] `test_register_after_deregister`
- [ ] 🟡 Miri run — undefined behaviour check on ABI types
  - [ ] `cargo miri test` passes clean
  - [ ] Add Miri job to CI — nightly only, allowed to fail

### 1.5 `no_std` Enforcement
- [x] 🔴 `#![cfg_attr(not(test), no_std)]` in `src/lib.rs`
- [ ] 🔴 CI job: build with `--target thumbv7m-none-eabi`
  - [ ] Passes clean with no `std` leaking through

---

## Phase 2 — ABI Stable `v0.3.0`
> Core ABI frozen. `sokr.h` generated and committed.
> Integration tested against `sokr-plugins` reference implementations.

- [ ] 🔴 Integration test against `sokr-cpu` from `sokr-plugins`
  - [ ] Register → Capability → Dispatch → Completion round-trip
  - [ ] Passes against CPU substrate as external dependency
- [ ] 🔴 `sokr.h` C header finalised and committed to `include/`
- [ ] 🟡 C example in `examples/c/hello_compute.c`
- [ ] 🟡 C++ RAII wrapper in `include/sokr.hpp`
- [ ] 🟡 Benchmark: core dispatch overhead < 5% vs raw vtable call

---

## Phase 3 — Formal Verification Roadmap `v1.x`
> Sovereignty claim backed by proof, not just philosophy.

- [ ] 🟢 Survey seL4 capability model for applicable formal methods
- [ ] 🟢 Specify version handshake protocol in TLA+ or Alloy
- [ ] 🟢 Verify ABI memory safety invariants with Miri and KLEE
- [ ] 🟢 Publish formal specification as `docs/formal-spec.md`

---

## SemVer Policy

| Version | Meaning |
|---|---|
| `0.1.x` | Foundation. ABI defined, no routing yet. |
| `0.2.x` | Core ABI complete. Registry + routing implemented. |
| `0.3.x` | ABI frozen. Integrated with sokr-plugins. |
| `1.0.0` | Core ABI stable. Formal spec published. |
| `1.x.x` | Backwards compatible additions only. |
| `2.0.0` | Core ABI breaking change. RFC required. |

---

## Contribution Policy

- All contributions require DCO sign-off (`Signed-off-by:` in commit)
- Core ABI changes require RFC and 4-week community comment period
- Plugin contributions → submit to `sokr-rs/sokr-plugins` instead
- Copyright assigned to **The SOKR Project**
- License: **MIT OR Apache-2.0** — no exceptions

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
