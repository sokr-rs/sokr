# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-17

### Added

- **Foundation Phase Complete**: Project initialization and core infrastructure
- **C ABI Surface Specification**: Full type system for plugin interface
  - `SokrVersion` - version handshake struct with `major.minor.patch`
  - `SokrResult` - comprehensive error codes (Ok, CapabilityDenied, DispatchFailed, Timeout, VersionMismatch, etc.)
  - `SokrComputationId` - opaque 128-bit computation identifier
  - `SokrCapabilityQuery/Response` - capability negotiation interface
  - `SokrDispatchRequest/Response` - dispatch interface
  - `SokrCompletionToken/Query/Signal` - completion polling interface
  - `SokrSubstratePlugin` - vtable for plugin registration
- **Version Handshake Protocol**: Plugin compatibility negotiation
  - `check_compatible()` method enforcing: major versions must match, plugin minor ≤ core minor
  - Forward/backward compatibility rules documented
  - `VersionMismatch` error for graceful rejection
- **Thread Safety Contract**: Comprehensive documentation on concurrent usage
- **Ownership Semantics**: Pointer lifetime rules and allocation contracts
- **Documentation**:
  - `README.md` - project overview with comparison table
  - `ARCHITECTURE.md` - design philosophy and plugin categories
  - `TODO.md` - development roadmap through Phase 4
  - `CONTRIBUTING.md` - contribution guidelines with DCO rules
  - `RFC 0001` - plugin interface specification
- **CI/CD Pipeline**:
  - GitHub Actions workflow with 6 jobs (check, test matrix on stable/beta/nightly, fmt, clippy, audit, no_std build)
  - `cargo-audit` integration with `audit.toml` configuration
  - `cargo-deny` license checking with `deny.toml`
  - Dependabot configuration for weekly updates
- **Developer Tooling**:
  - Pre-commit hooks with standard checks and Rust-specific validation
  - Issue templates (bug report, feature request, plugin proposal)
  - `.gitignore` with comprehensive patterns
- **Legal**: Dual MIT OR Apache-2.0 licensing with proper headers

### Notes

This is the **Foundation Release** - the crate reserves the name on crates.io and establishes the architecture, but contains no runnable code. The core ABI types are defined and documented, ready for Phase 1 implementation.

[Unreleased]: https://github.com/sokr-rs/sokr/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sokr-rs/sokr/releases/tag/v0.1.0
