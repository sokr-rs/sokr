# RFC 0001: Plugin Interface Specification

> Status: **Draft**  
> Target Version: 0.2.0  
> Comment Period: 14 days from publication

---

## Summary

This RFC specifies the SOKR plugin interface — the contract between the immutable core and swappable substrate plugins. The goal is to stabilize the C ABI surface for Phase 1 (v0.2.0) implementation.

---

## Motivation

SOKR's core value proposition is substrate independence. For this to work, the plugin interface must be:

1. **Stable** — once released, breaking changes require major version bump
2. **Minimal** — only three operations cross the boundary
3. **Safe** — no undefined behavior from valid plugin code
4. **Testable** — plugins can be validated without real hardware

---

## Detailed Design

### 1. VTable Registration

Plugins register via a VTable struct passed to the core:

```rust
#[repr(C)]
pub struct SokrSubstratePlugin {
    pub version: SokrVersion,
    pub capability_fn: SokrCapabilityFn,
    pub dispatch_fn: SokrDispatchFn,
    pub completion_fn: SokrCompletionFn,
    pub destroy_fn: SokrDestroyFn,
    _padding: [u8; 16],
}
```

**Invariants:**
- All function pointers must be non-null
- `version` must pass `check_compatible()` against core version
- `destroy_fn` is only called after all operations complete

### 2. Capability Query

The core asks: "Can this substrate fulfill this computation?"

```rust
pub type SokrCapabilityFn = extern "C" fn(
    version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult;
```

**Contract:**
- `version` is the core's version for handshake
- Plugin returns `Ok` if capable, `CapabilityDenied` if not
- Plugin may return `VersionMismatch` for incompatible ABI
- `estimated_latency_ns` is advisory; 0 means "unknown"

### 3. Dispatch

The core says: "Fulfill this computation."

```rust
pub type SokrDispatchFn = extern "C" fn(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult;
```

**Contract:**
- Must return immediately with a completion token (async model)
- Synchronous substrates should still return token, then immediately complete
- Token must be unique per dispatch within substrate
- `DispatchFailed` for runtime errors (OOM, device lost, etc.)

### 4. Completion

The core asks: "What is the status of this dispatch?"

```rust
pub type SokrCompletionFn = extern "C" fn(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult;
```

**Contract:**
- May be called multiple times per token
- Returns `Pending` until work is done
- Returns `Complete` or `Failed` once, then token is invalid
- `TimedOut` if query timeout expires before completion

### 5. Cleanup

```rust
pub type SokrDestroyFn = extern "C" fn();
```

**Contract:**
- Called exactly once per plugin
- Only called after all dispatches complete
- Plugin releases all resources

---

## Thread Safety

All functions must be thread-safe:

| Function | Concurrent Calls | Notes |
|----------|-----------------|-------|
| `capability_fn` | ✅ Yes | Synchronize internal state access |
| `dispatch_fn` | ✅ Yes | Each call gets new token |
| `completion_fn` | ✅ Yes | Multiple queries per token allowed |
| `destroy_fn` | ❌ No | Only after all operations complete |

---

## Error Handling

Plugins must handle all error cases gracefully:

| Error | When Returned | Recovery |
|-------|--------------|----------|
| `Ok` | Success | Proceed |
| `CapabilityDenied` | Substrate cannot fulfill | Try another substrate |
| `DispatchFailed` | Runtime failure | Retry or escalate |
| `Timeout` | Operation timed out | Retry with longer timeout |
| `VersionMismatch` | Incompatible ABI | Plugin incompatible |
| `InvalidInput` | Null pointer or bad alignment | Bug in caller |
| `InvalidIR` | IR format rejected | Try different IR plugin |

---

## Version Handshake

Detailed version compatibility is documented in `SokrVersion::check_compatible()`.
Key points:

- Major versions must match exactly
- Plugin minor ≤ Core minor
- Patch is informational only
- Incompatible plugins return `VersionMismatch`, never panic

---

## Drawbacks

1. **C ABI constraints** — No rich types, manual memory layout management
2. **No async/await** — Manual polling via completion queries
3. **Function pointer overhead** — Small but measurable vs direct calls
4. **Plugin complexity** — Substrate authors must implement all four functions

---

## Alternatives Considered

### Dynamic Linking with dlopen
- Rejected: Requires OS support, conflicts with `no_std` goals

### COM-style Interface Query
- Rejected: Adds complexity for marginal benefit; three functions is minimal

### Pure Rust Traits
- Rejected: Limits plugin authors to Rust; C ABI enables any language

---

## Unresolved Questions

1. Should `capability_fn` include a "warmup" hint for JIT compilation?
2. Should we add a "batch dispatch" API for submitting multiple computations atomically?
3. How should plugins report progress for long-running computations?

---

## Implementation Timeline

| Phase | Target | Deliverable |
|-------|--------|-------------|
| RFC Comment | Now | This document |
| Freeze Decision | +14 days | Accept/modify/reject |
| Implementation | 0.2.0 | Working CPU plugin |
| Stabilization | 0.3.0 | No changes unless bugfix |

---

## Comment Period

Please leave feedback as comments on this GitHub Discussion:
- **Link**: TBD (will be created when merged)
- **Duration**: 14 days
- **Requested input**: Thread safety edge cases, error code coverage, naming

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
