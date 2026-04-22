# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project context

SOKR — Sovereign Open Kernel Runtime. A `no_std` Rust core that exposes exactly three operations (Capability → Dispatch → Completion) across a stable C ABI; everything else (IR, substrate, language binding, dispatch policy) is a plugin.

**Early design phase, `v0.1.x`.** The core ABI is specified and the FFI entry points exist as stubs that validate inputs and return `NoCapableSubstrate` — no substrate routing yet. See `TODO.md` for the roadmap and `ARCHITECTURE.md` for the design invariants.

## Commands

```bash
cargo check --all-targets
cargo test --all-features           # core ffi tests are gated behind --features ffi
cargo fmt --check
cargo clippy -- -D warnings         # CI rejects any warning
cargo audit --deny warnings

# no_std must keep working — CI builds on thumbv7m-none-eabi
rustup target add thumbv7m-none-eabi
cargo build --target thumbv7m-none-eabi

# Run a single test
cargo test version_compatible_same

# Generate the C header (cbindgen config lives in cbindgen.toml)
cbindgen --crate sokr --output sokr.h
```

Pre-commit hooks (`cargo fmt`, `cargo check`, `cargo clippy -D warnings`) run on every commit. Never bypass with `--no-verify` — fix the hook failure and create a new commit rather than amending.

## Architecture invariants

These are load-bearing. Breaking any of them is a philosophy-level change, not a cosmetic one.

- **Core has zero dependencies.** `Cargo.toml` has no `[dependencies]`. Do not add a dependency to the core to solve a problem — push the problem into a plugin.
- **Core is `no_std`.** `src/lib.rs` uses `#![cfg_attr(not(test), no_std)]` and ships its own panic handler. Anything requiring `std` belongs in a plugin.
- **Unsafe is confined to `src/ffi.rs`.** The crate sets `#![cfg_attr(not(feature = "ffi"), forbid(unsafe_code))]` and the `ffi` module opts back in with `#![allow(unsafe_code)]`. FFI itself is gated behind the `ffi` feature (`#[cfg(feature = "ffi")] pub mod ffi;`). Do not sprinkle `unsafe` elsewhere.
- **Everything crossing the plugin boundary is `#[repr(C)]`.** All structs, enums (`#[repr(u32)]`), and function-pointer `type` aliases in `types.rs` exist for the C ABI. Changing their layout is an ABI break.
- **Version handshake rules** (`SokrVersion::check_compatible`): major must equal, plugin minor ≤ core minor, patch ignored. `SokrVersion::CURRENT` must be bumped in lockstep with the workspace version in root `Cargo.toml`.
- **Registry uses a fixed-size array**, no heap (`MAX_SUBSTRATES = 16`, `[Option<SokrSubstratePlugin>; MAX_SUBSTRATES]`). Don't reach for `Vec`/`Box` in the core.
- **`SokrCompletionToken.handle = 0` is the reserved "invalid/unset" sentinel.** `sokr_completion` returns `InvalidInput` for handle 0; dispatch failures zero the handle out. Preserve this contract when implementing substrate routing.
- **FFI stubs currently return `NoCapableSubstrate`** after validating pointers. When implementing routing, keep the null-pointer and zero-length-IR checks at the top — they're part of the ABI contract, not incidental.

## Repository layout

Single crate `sokr` with three modules. Plugins live in a separate `sokr-plugins` repo.

- `src/lib.rs` — crate root, `no_std`
- `src/types.rs` — C ABI struct and enum definitions
- `src/registry.rs` — fixed-size plugin registry
- `src/ffi.rs` — feature-gated `#[no_mangle] extern "C" exports

Plugin crates (CPU substrate, dispatch policy) are maintained in [sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins).

## Contribution rules that affect PRs

- **DCO sign-off required** on every commit (`git commit -s`). CI enforces it for external contributors.
- **Conventional Commits.** Types used here: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `abi` (requires RFC), `plugin`. Common scopes: `core`, `abi`, `cpu`, `dispatch`, `docs`.
- **Core ABI changes require an RFC** in `docs/rfc/` with a 14-day comment period. Plugin changes do not.
- Copyright on all contributions is assigned to "The SOKR Project"; dual MIT OR Apache-2.0.
