# Contributing to SOKR

> Sovereign Open Kernel Runtime

Thank you for your interest in contributing to SOKR. This document outlines the requirements and guidelines for all contributions.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [DCO Sign-Off Requirement](#dco-sign-off-requirement)
3. [Contribution Paths](#contribution-paths)
4. [Code Style](#code-style)
5. [Commit Message Convention](#commit-message-convention)
6. [Copyright Assignment](#copyright-assignment)

---

## Getting Started

### Development Environment Setup

1. **Install Rust** via [rustup](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup component add rustfmt clippy
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/sokr-rs/sokr.git
   cd sokr
   ```

3. **Verify your setup**:
   ```bash
   cargo check
   cargo test
   cargo fmt --check
   cargo clippy
   ```

4. **Install additional tools** (optional but recommended):
   ```bash
   cargo install cargo-audit cargo-deny
   ```

---

## DCO Sign-Off Requirement

**Every commit must include a DCO (Developer Certificate of Origin) sign-off.**

This certifies that you have the right to submit the code under the project's license. The sign-off is a simple line at the end of your commit message:

```
Signed-off-by: Your Name <your.email@example.com>
```

### How to Sign Off

**Option 1: Configure git to always sign off**
```bash
git config --global format.signoff true
```

**Option 2: Add sign-off manually per commit**
```bash
git commit -s -m "Your commit message"
```

**Option 3: Amend an existing commit**
```bash
git commit --amend --signoff --no-edit
```

By contributing, you agree to the following:

> By making a contribution to this project, I certify that:
> (a) The contribution was created in whole or in part by me and I have the right to submit it under the open source license indicated in the file; or
> (b) The contribution is based upon previous work that, to the best of my knowledge, is covered under an appropriate open source license and I have the right under that license to submit that work with modifications, whether created in whole or in part by me, under the same open source license (unless I am permitted to submit under a different license), as indicated in the file; or
> (c) The contribution was provided directly to me by some other person who certified (a), (b) or (c) and I have not modified it.
> (d) I understand and agree that this project and the contribution are public and that a record of the contribution (including all personal information I submit with it, including my sign-off) is maintained indefinitely and may be redistributed consistent with this project or the open source license(s) involved.

---

## Contribution Paths

SOKR has two distinct contribution paths:

### 1. Plugin Contributions

**No RFC required.** Plugins are sovereign.

- Create your plugin in a new repository
- Follow the plugin interface specification in `ARCHITECTURE.md`
- Publish to crates.io with the `sokr-` prefix (optional but recommended)
- Open an issue in `sokr-rs/sokr` to be listed in the ecosystem registry

### 2. Core ABI Changes

**RFC required with community comment period.**

The core ABI is intentionally minimal and stable. Changes must go through:

1. **Open an RFC issue** in `sokr-rs/sokr` with the `rfc` label
2. **Describe the problem** — what limitation does this solve?
3. **Describe the solution** — what changes to the C ABI surface?
4. **Community comment period** — minimum 14 days
5. **Maintainer decision** — approve, request changes, or reject

Examples of core ABI changes:
- Adding/removing functions from the C ABI surface
- Changing struct layouts or version handshake protocol
- Modifying the plugin registration API

---

## Code Style

All contributions must pass the following checks:

### Rustfmt
```bash
cargo fmt --check
```
All Rust code must be formatted with `rustfmt`. CI will reject PRs that fail this check.

### Clippy
```bash
cargo clippy -- -D warnings
```
No clippy warnings allowed. If you believe a warning is a false positive, suppress it with an explicit comment explaining why.

### no_std Compliance
The SOKR core must remain `no_std` compatible:
```rust
#![no_std]
#![forbid(unsafe_code)]
```

If your code requires the standard library, it belongs in a plugin crate, not the core.

### Documentation
All public items must have doc comments:
```rust
/// Brief description of what this does.
///
/// Longer description if needed, with examples when appropriate.
pub fn my_function() { }
```

---

## Commit Message Convention

We use **Conventional Commits** format:

```
<type>(<scope>): <short description>

[optional body]

Signed-off-by: Your Name <your.email@example.com>
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only changes |
| `style` | Code style (formatting, no logic change) |
| `refactor` | Code refactoring |
| `test` | Adding or updating tests |
| `chore` | Build process, dependencies, tooling |
| `abi` | Core ABI changes (requires RFC) |
| `plugin` | Plugin-related changes |

### Scopes

Common scopes: `core`, `abi`, `cpu`, `vulkan`, `spirv`, `dispatch`, `docs`

### Examples

```
feat(core): add plugin version handshake

Implements version negotiation at plugin load time.
Prevents silent ABI breaks.

Signed-off-by: Alice Developer <alice@example.com>
```

```
docs(abi): document C ABI surface types

Add detailed documentation for SokrCapabilityQuery,
SokrDispatchRequest, and SokrCompletionSignal.

Signed-off-by: Bob Contributor <bob@example.com>
```

---

## Copyright Assignment

By contributing to SOKR, you agree that:

1. **All contributions are dual-licensed** under MIT OR Apache-2.0
2. **Copyright is assigned to "The SOKR Project"**
3. **No additional terms or conditions** are imposed

This ensures the project remains uniformly licensed and cannot be encumbered by scattered copyright holders.

---

## Questions?

- Open an issue in `sokr-rs/sokr`
- Check existing issues and RFCs before proposing changes
- For security issues, see [SECURITY.md](SECURITY.md)

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
