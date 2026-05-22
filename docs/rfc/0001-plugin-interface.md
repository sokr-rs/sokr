# RFC 0001: Plugin Interface

**Status:** Closed — resolved 2026-05-14
**Opened:** 2026-04-16
**Closed:** 2026-05-14 (4 week comment period)
**Tracking:** https://github.com/sokr-rs/sokr/discussions/2

---

## Summary

This RFC defines the core C ABI plugin interface for SOKR — the
Capability, Dispatch, and Completion contract that all substrate
plugins must implement, plus the version handshake protocol.

---

## Motivation

SOKR's core philosophy is: immutable sovereign core, everything else
a plugin. The plugin interface is the contract that makes this possible.
Getting it right before any code is written is more important than any
other decision in Phase 0 — a wrong interface here costs every plugin
author a breaking change at v0.2.x.

---

## The Interface

### Three Functions

Every substrate plugin implements exactly three functions:

```c
// Can this substrate fulfill this computation?
SokrResult sokr_capability(
    const SokrCapabilityQuery* query,
    SokrCapabilityResponse*    response
);

// Fulfill it.
SokrResult sokr_dispatch(
    const SokrDispatchRequest* request,
    SokrDispatchResponse*      response
);

// Signal when fulfilled.
SokrResult sokr_completion(
    const SokrCompletionQuery*  query,
    SokrCompletionSignal*       signal
);
```

Plus one lifecycle function:

```c
// Called by core when plugin is deregistered.
void sokr_destroy(void);
```

### Type Definitions

```c
typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
} SokrVersion;

typedef enum {
    SOKR_RESULT_OK                    = 0,
    SOKR_RESULT_CAPABILITY_DENIED     = 1,
    SOKR_RESULT_DISPATCH_FAILED       = 2,
    SOKR_RESULT_TIMEOUT               = 3,
    SOKR_RESULT_VERSION_MISMATCH      = 4,
    SOKR_RESULT_NO_CAPABLE_SUBSTRATE  = 5,
    SOKR_RESULT_INVALID_INPUT         = 6,
    SOKR_RESULT_INVALID_IR            = 7,
    SOKR_RESULT_NOT_FOUND             = 8,
    SOKR_RESULT_REGISTRY_FULL         = 9,
} SokrResult;

typedef struct {
    uint8_t  bytes[16];           // opaque 128-bit ID
} SokrComputationId;

typedef struct {
    SokrComputationId  computation_id;
    uint32_t           ir_format;     // IR format identifier
    const uint8_t*     ir_data;       // pointer to IR bytes
    size_t             ir_data_len;   // length of IR bytes
} SokrCapabilityQuery;

typedef struct {
    SokrResult  result;
    uint32_t    substrate_id;
    uint64_t    estimated_latency_ns;
} SokrCapabilityResponse;

typedef struct {
    SokrComputationId  computation_id;
    uint32_t           substrate_id;
    const uint8_t*     ir_data;
    size_t             ir_data_len;
    const uint8_t*     params;        // optional dispatch parameters
    size_t             params_len;
} SokrDispatchRequest;

typedef uint64_t SokrCompletionToken;  // opaque handle

typedef struct {
    SokrResult           result;
    SokrCompletionToken  completion_token;
} SokrDispatchResponse;

typedef struct {
    SokrCompletionToken  completion_token;
    uint64_t             timeout_ns;   // 0 = non-blocking poll
} SokrCompletionQuery;

typedef enum {
    SOKR_COMPLETION_SIGNAL_PENDING  = 0,
    SOKR_COMPLETION_SIGNAL_COMPLETE = 1,
    SOKR_COMPLETION_SIGNAL_FAILED   = 2,
    SOKR_COMPLETION_SIGNAL_TIMED_OUT = 3,
} SokrCompletionSignal;

typedef struct {
    SokrVersion   version;
    SokrResult  (*capability_fn)(const SokrCapabilityQuery*, SokrCapabilityResponse*);
    SokrResult  (*dispatch_fn)(const SokrDispatchRequest*, SokrDispatchResponse*);
    SokrResult  (*completion_fn)(const SokrCompletionQuery*, SokrCompletionSignal*);
    void        (*destroy_fn)(void);
    uint64_t      substrate_id;
    uint8_t       padding[8];
} SokrSubstratePlugin;
```

### IR Format Identifiers

IR format is a `uint32_t` magic number declared by the IR plugin:

| IR | Identifier |
|---|---|
| SPIR-V | `0x53505256` |
| PTX (NVIDIA) | `0x50545800` |
| OpenQASM 3 | `0x4F51334D` |
| Spike graph | `0x53504B45` |
| Optical circuit | `0x4F50544C` |
| SOKR-native (future) | TBD |

---

## Version Handshake Protocol

### Compatibility Rules

- `major` must match exactly — different majors are incompatible
- Plugin `minor` must be ≤ core `minor` — older plugins run on newer core
- Plugin `minor` > core `minor` — newer plugin on older core is accepted
  (forward compatibility: core ignores fields it does not understand)
- `patch` is irrelevant to compatibility

### Negotiation Sequence

1. Core calls `plugin.version_fn()` during registration
2. Plugin returns its compiled-in `SokrVersion`
3. Core compares against `SokrVersion::current()`
4. Incompatible → `SOKR_RESULT_VERSION_MISMATCH` returned, plugin not registered
5. Compatible → plugin assigned a `substrate_id`, registration succeeds

### Version Bump Triggers

| Change | Version bump |
|---|---|
| Add a field to any ABI struct | `minor` |
| Remove or reorder a field | `major` |
| Add a new `SokrResult` variant | `minor` |
| Change function signature | `major` |
| Bug fix with no ABI change | `patch` |

---

## Ownership Semantics

- All pointers in query/request structs are **borrowed** — the plugin
  must not free them and must not retain them after the function returns
- All pointers in response structs are **owned by the caller** — the
  caller allocates, the plugin writes into them
- `SokrCompletionToken` is owned by the caller — the plugin must not
  free it; the core manages token lifetime
- `destroy_fn` is called exactly once, by the core, on deregistration

## Thread Safety Contract

- `sokr_capability()` — safe to call concurrently from multiple threads
- `sokr_dispatch()` — safe to call concurrently from multiple threads
- `sokr_completion()` — safe to call concurrently from multiple threads
- `sokr_register_substrate()` — NOT thread-safe, call from one thread only
- `sokr_deregister_substrate()` — NOT thread-safe, call from one thread only

---

## Alternatives Considered

### Four functions instead of three
Adding a `sokr_cancel()` function was considered. Rejected: cancellation
semantics differ too much across substrates (QPU jobs cannot be cancelled
once submitted). Cancellation is a dispatch policy concern, not a core
concern. A plugin can expose cancellation through the `params` field of
`SokrDispatchRequest` if needed.

### Single `sokr_execute()` combining all three
Rejected: conflates synchronous and asynchronous substrates. A
photonic substrate may have seconds of latency. A CPU substrate is
immediate. Separating dispatch from completion allows the caller to
do other work between them.

### Rust trait instead of C ABI
Rejected: would limit plugins to Rust. The sovereignty principle
requires any language to implement a plugin without depending on the
Rust toolchain.

---

## Open Questions

These are the questions we are asking the community to address during
the comment period. Every question will be resolved with either a spec
change or a documented rationale for keeping the current design.

1. Is `uint64_t` sufficient for `SokrCompletionToken` or should it be
   128 bits to reduce collision probability in long-running systems?

2. Should `estimated_latency_ns` in `SokrCapabilityResponse` be
   mandatory or optional (zero = unknown)?

3. Should `params` in `SokrDispatchRequest` have a defined schema, or
   remain fully opaque bytes defined by the substrate plugin?

4. Is the thread safety contract sufficient, or do we need a reader-writer
   lock model for the plugin registry?

5. Should `sokr_list_substrates()` be part of the core ABI or a
   separate introspection plugin?

---

## Resolution

Comment period closed 2026-05-14. All five open questions resolved:

| # | Question | Decision |
|---|---|---|
| 1 | `uint64_t` vs 128-bit completion token | Keep `uint64_t` — sufficient for single-process registries; 128-bit adds ABI cost with no benefit at current scale |
| 2 | `estimated_latency_ns` mandatory vs optional | Zero = unknown; field stays mandatory, callers must treat 0 as "no estimate" |
| 3 | `params` schema | Fully opaque — substrate-defined; IR plugin documents expected format |
| 4 | Thread safety contract | Accepted as-is — register/deregister single-threaded; capability/dispatch/completion concurrent |
| 5 | `sokr_list_substrates` in core vs plugin | Kept in core as `sokr_list_substrates()`; implemented in v0.2.0 |

No changes to the interface were required. All decisions accepted by
silence (no objections during comment period).

---

## Decision Log

| Date | Question | Decision | Rationale |
|---|---|---|---|
| 2026-05-14 | 1 | `uint64_t` sufficient for token | Collision probability negligible in single-process registries |
| 2026-05-14 | 2 | `estimated_latency_ns` remains mandatory | Zero value is the undefined-latency marker |
| 2026-05-14 | 3 | `params` remain opaque | Substrate-plugin-defined schema; introspection via IR plugin if needed |
| 2026-05-14 | 4 | Thread safety contract accepted | Register/deregister serialization burden acceptable; no lock needed for query functions |
| 2026-05-14 | 5 | `sokr_list_substrates` in core | Introspection is first-class; function was implemented in v0.2.0 |

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
