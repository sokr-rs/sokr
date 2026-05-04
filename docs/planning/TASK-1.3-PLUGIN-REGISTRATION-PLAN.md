# Problem Framing & Planning: SOKR Task 1.3 (Plugin Registration)

**Status:** Draft for Human Review  
**Version:** 0.1.0  
**Date:** 2026-04-20  
**Task ID:** SOKR-1.3  
**Crate Version:** sokr v0.1.2  

---

## 1. Problem Statement

Task 1.3 requires implementing two coupled operations—`sokr_register_substrate()` and `sokr_deregister_substrate()`—that manage the lifecycle of substrate plugins in the SOKR core registry. The core decision points are:

- **Who assigns `substrate_id`** (plugin-provided vs. core-assigned auto-increment)
- **When version validation occurs** (registration-time only or per-dispatch)
- **Thread-safety guarantees** (none now, deferred to a later phase)
- **Which invariants must hold** for `substrate_id` across the plugin lifecycle

---

## 2. Falsifying Outcome

The implementation is **wrong** if any of the following occur:

1. Two plugins registered in the same session receive the same `substrate_id`.
2. An incompatible plugin is registered and incompatibility is detected only at dispatch time (or never).
3. A valid `substrate_id` becomes invalid during plugin lifetime without explicit deregistration.
4. The registry reports success when full (capacity underflow).
5. The plugin `destroy_fn` is not called on deregistration, or is called multiple times.
6. Heap allocation (`Vec`, `Box`, etc.) is used despite the no-heap constraint.
7. Concurrent registration/deregistration causes races and corrupt state.
8. A plugin returning `VersionMismatch` during registration is still stored in the registry.

Any of these outcomes falsify the design and must be caught by tests or CI.

---

## 3. Scope

### In Scope (Task 1.3)

- Implement `sokr_register_substrate()`:
  - Validate non-null pointers.
  - Perform version compatibility check via `SokrVersion::check_compatible`.
  - Assign a unique, non-zero `substrate_id`.
  - Store plugin in fixed-size static array (`MAX_SUBSTRATES = 16`).
  - Return appropriate `SokrResult` codes (`Ok`, `InvalidInput`, `VersionMismatch`, `RegistryFull`).

- Implement `sokr_deregister_substrate()`:
  - Validate input.
  - Locate plugin by `substrate_id`.
  - Call `destroy_fn` exactly once.
  - Free slot for reuse.
  - Return `Ok` or `NotFound`/`InvalidInput`.

- Add unit tests covering:
  - Successful registration and ID assignment.
  - Capacity overflow (`RegistryFull`).
  - Version incompatibility (`VersionMismatch`).
  - Null-pointer handling (`InvalidInput`).
  - Deregistration success and slot reuse.
  - Deregister unknown ID (`NotFound`).

- Ensure no heap allocation and no_std compatibility.

### Out of Scope (Deferred)

- `sokr_list_substrates()` (marked 🟡 in TODO).
- Thread-safety synchronization (`Mutex`/`RwLock`) in internals.
- Plugin hot-reload and substrate migration.
- C header regeneration beyond existing `cbindgen` workflow.

---

## 4. Constraint Inventory

1. **MAX_SUBSTRATES = 16** — fixed-size array, no heap.
2. **`substrate_id != 0`** — zero reserved as invalid/unset sentinel.
3. **`substrate_id` must be unique** within active registry entries.
4. **`substrate_id` must be stable** for plugin lifetime.
5. **Version check mandatory** at registration.
6. **Version handshake rule**: `plugin.major == core.major && plugin.minor <= core.minor`.
7. **`destroy_fn` called exactly once** on successful deregistration.
8. **All FFI pointers non-null** (`InvalidInput` on violation).
9. **Unknown deregistration ID returns `NotFound`** (no panic).
10. **Slot reuse allowed** after deregistration.
11. **FFI functions must not panic**; errors represented by `SokrResult`.
12. **Thread-safety not guaranteed in Phase 1.3** and must be documented.

---

## 5. Position Statement

**Chosen position:** Core-assigned auto-incremented `substrate_id`, registration-time version validation only, thread-safety deferred, and no per-dispatch version revalidation.

**Rationale:**

- Core assignment guarantees uniqueness by construction.
- Registration-time check fails fast and keeps dispatch path minimal.
- Deferring synchronization avoids premature complexity and ABI risk.
- No per-dispatch version checks avoids overhead on the hot path.

**Trade-off accepted:** ABI/version changes require deregister/register rather than in-place mutation.

---

## 6. Alternatives and Trade-Offs

### Alternative A — Plugin-Provided ID + Registration-Time Validation (Rejected)

Plugins provide `substrate_id`; core checks uniqueness and version at registration.

### Alternative B — Core-Assigned Auto-Increment + Registration-Time Validation (Chosen)

Core assigns IDs, validates version once, stores plugin.

### Alternative C — Core-Assigned ID + Per-Dispatch Version Revalidation (Rejected)

Core assigns IDs and re-checks version on each dispatch/completion path.

| Dimension | A | B (Chosen) | C |
|---|---|---|---|
| Uniqueness guarantee | Conditional (core must reject collisions) | Strong by construction | Strong by construction |
| Plugin burden | Higher | Lower | Lower |
| Dispatch overhead | Low | Low | Higher |
| Complexity | Medium | Low | High |
| Failure timing | Registration | Registration | Mixed |
| Test surface | Medium | Low-medium | High |

**Why B:** Best fit with current constraints (`no_std`, fixed capacity, low complexity, predictable behavior).

---

## 7. Reversibility Tag

**One-Way Door** (API/ABI behavior): once external users rely on ID semantics and registration contract, changing assignment strategy becomes expensive and potentially breaking.

**Cost of reversal:** high (plugin ecosystem updates, docs/spec updates, integration breakage).

---

## 8. Draft ADR Skeleton

### ADR-1.3.1 — Substrate ID Assignment

- **Status:** Proposed  
- **Context:** Registry must assign stable unique IDs under fixed capacity and no heap.
- **Decision:** Core assigns IDs (non-zero, unique), independent from plugin-provided value.
- **Consequences:** Simplifies plugin integration; codifies one-way behavior for identity semantics.

### ADR-1.3.2 — Version Validation Timing

- **Status:** Proposed  
- **Context:** Need ABI compatibility assurance with minimal runtime overhead.
- **Decision:** Validate only at registration via `SokrVersion::check_compatible`.
- **Consequences:** Faster dispatch/completion path; incompatible plugins fail fast before use.

### ADR-1.3.3 — Thread-Safety Deferral

- **Status:** Proposed  
- **Context:** Internal synchronization strategy not finalized in this phase.
- **Decision:** Mark register/deregister as not thread-safe for now; document caller serialization requirement.
- **Consequences:** Lower implementation complexity now; future hardening planned.

---

## 9. Spec Sketch (C ABI-Oriented)

### `sokr_register_substrate()`

- Inputs:
  - pointer to plugin vtable/descriptor
  - pointer to `substrate_id_out`
- Behavior:
  - validate pointers
  - validate version compatibility
  - allocate free slot
  - assign unique non-zero ID
  - write assigned ID to output
- Errors:
  - `InvalidInput`
  - `VersionMismatch`
  - `RegistryFull`

**Invariant set:**

1. Assigned ID is unique among active entries.
2. Assigned ID is non-zero.
3. Slot state transitions are atomic with respect to function outcome.
4. No heap allocation.

### `sokr_deregister_substrate()`

- Inputs:
  - `substrate_id`
- Behavior:
  - find active slot by ID
  - invoke `destroy_fn` once
  - clear slot
- Errors:
  - `InvalidInput` (if request pointer invalid in FFI wrapper)
  - `NotFound`

---

## 10. Acceptance Criteria (WRITE Handoff)

1. Register one valid plugin succeeds and returns assigned non-zero ID.
2. IDs are unique across multiple registrations.
3. Register beyond capacity returns `RegistryFull`.
4. Register incompatible version returns `VersionMismatch`.
5. Register with null pointer returns `InvalidInput`.
6. Deregister existing plugin succeeds.
7. Deregister unknown ID returns `NotFound`.
8. Deregister calls `destroy_fn` exactly once.
9. Deregistered slot can be reused by subsequent registration.
10. No heap allocation in registry path.
11. Tests added for all above outcomes.
12. Existing capability/dispatch/completion paths remain compatible with registry behavior.

---

## 11. Open Questions (Human Review)

1. Should ID assignment be monotonic forever or slot-index-derived with generation counter?
2. If `destroy_fn` misbehaves, should deregistration still clear the slot?
3. Should duplicate plugin registration (same function pointers/version) be allowed as separate entries?
4. Should list/introspection API return stable ordering or insertion order?
5. At what exact milestone do we require thread-safe register/deregister internals?
6. Should registration expose richer diagnostics (beyond `SokrResult`) for operations tooling?
7. Should TODO 1.3 and 1.4 tests be unified into a single registry test module?

---

## 12. Summary / Action Items

1. Implement registration with pointer checks, version checks, unique ID assignment, and fixed-array insertion.
2. Implement deregistration with lookup, `destroy_fn` call, and slot reuse.
3. Add required unit tests for success/failure matrix.
4. Keep behavior aligned with no-heap/no_std constraints.
5. Document thread-safety limitation explicitly in API docs.
6. Record final decisions in ADR(s) before v0.2.0 freeze.

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
