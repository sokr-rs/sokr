# SOKR Version Handshake Formal Specification

**Status**: Verified — passes TLC (Task 3.2)
**Tool**: TLA+ / TLC model checker 2.19
**Artifacts**: `docs/formal_spec.tla`, `docs/formal_spec.cfg`

---

## Overview

This document describes the formal specification of SOKR's plugin version
handshake and the result of model-checking it. The specification models the
core's plugin registry and the version-compatibility gate enforced at
registration time, and proves (by exhaustive state exploration) that an
incompatible plugin can never occupy a registry slot.

**Verified claim**: every plugin present in the registry satisfies the
compatibility rule. The model checker explored all reachable states and found
no violation of this or any other asserted invariant.

---

## What the spec models

The TLA+ module `formal_spec` captures:

1. **State**: a fixed-size registry (`[1..MaxSubstrates -> Entry ∪ {NULL}]`) and
   the set of active plugin ids.
2. **Compatibility rule**: `VersionCompatible(major, minor)`, mirroring
   `SokrVersion::check_compatible` (`src/types.rs:65`).
3. **Transitions**: `Register` (compatible plugin takes an empty slot),
   `RejectIncompatible` (incompatible attempt leaves state unchanged), and
   `Deregister` (clears a slot).
4. **Invariants**: six safety properties checked by TLC.

Patch version is intentionally omitted from the model: the compatibility rule
ignores it, so modeling it would only inflate the state space.

---

## Where the rule actually lives (code mapping)

The compatibility gate is enforced at the FFI boundary, not in the registry
itself. `Registry::register_with_id` (`src/registry.rs`) only checks capacity;
the version check is in the FFI entry point:

```rust
// src/ffi.rs:136
let compatibility = plugin_value.version.check_compatible(SokrVersion::CURRENT);
if compatibility != SokrResult::Ok {
    return SokrResult::VersionMismatch;
}
```

| TLA+ | Code |
|------|------|
| `VersionCompatible(m, n)` | `SokrVersion::check_compatible` (`src/types.rs:65`) |
| `Register(p, m, n)` guard | version gate in `sokr_register_substrate` (`src/ffi.rs:136`) |
| `registry` / `Slots` | `Registry.substrates: [Option<…>; MAX_SUBSTRATES]` (`src/registry.rs`) |
| `Deregister(p)` | `sokr_deregister_substrate` (`src/ffi.rs:161`) |
| `MaxSubstrates` | `MAX_SUBSTRATES = 16` (`src/registry.rs:13`) |

---

## Invariants checked

| Invariant | Meaning |
|-----------|---------|
| `TypeOK` | State stays well-typed (registry entries are valid records or `NULL`). |
| `VersionCompatibilityInvariant` | **Core safety.** Every registered plugin satisfies the compatibility rule — incompatible plugins never occupy a slot. |
| `RegistryConsistencyInvariant` | The `active` set exactly mirrors the registry contents. |
| `NoDuplicateInvariant` | Each plugin id occupies at most one slot. |
| `CapacityInvariant` | The registry never exceeds `MaxSubstrates`. |
| `CompatibilityDecisionInvariant` | The decision is exactly the semantic rule `m = CoreMajor ∧ n ≤ CoreMinor` (no inverted/off-by-one condition). |

Liveness (e.g. "a compatible plugin can always eventually register") is out of
scope for this pass; only safety invariants are asserted.

---

## How to run TLC

Download `tla2tools.jar` (TLC 2.19+) from
<https://github.com/tlaplus/tlaplus/releases/latest> and run:

```bash
cd docs
java -cp /path/to/tla2tools.jar tlc2.TLC -config formal_spec.cfg formal_spec.tla
```

The committed `formal_spec.cfg` fixes the core version at 0.3 (matching
`SokrVersion::CURRENT`) and sweeps plugin versions over `Majors = {0,1}`,
`Minors = {0,1,2,3,4}` so every rejection path is exercised: wrong major (1)
and minor-too-high (4 > 3). `MaxSubstrates = 3` and three plugin ids exercise
capacity and duplicate-id boundaries.

### Verification result

```
TLC2 Version 2.19 of 08 August 2024
Model checking completed. No error has been found.
10063 states generated, 709 distinct states found, 0 states left on queue.
The depth of the complete state graph search is 4.
```

All six invariants hold across all 709 reachable states.

### Negative control

To confirm the check is not vacuous, a throwaway run asserted the deliberately
false invariant `active = {}` ("nothing is ever registered"). TLC correctly
reported a violation and produced a counterexample reaching the `Register`
action:

```
Error: Invariant NeverRegistersInvariant is violated.
Error: The behavior up to this point is:
State 2: <Register ... of module formal_spec_neg>
```

This demonstrates TLC genuinely explores the registration path, so the clean
pass on the real invariants is meaningful.

---

## Limitations

Out of scope by design:

1. **Memory safety** — pointer validity/alignment is verified separately by Miri
   (Task 3.3), not by this model.
2. **Concurrency** — the ABI contract is single-threaded for registration; the
   model assumes atomic transitions.
3. **Timing / hardware** — no latency, cache, or speculative-execution modeling.

Assumptions:

1. `CoreMajor`/`CoreMinor` faithfully reflect the ABI version.
2. The bounded version ranges in the `.cfg` are representative — they cover
   equal/lower/higher major and minor relative to the core, which are the only
   classes the rule distinguishes.

---

## Next steps

- **Task 3.3**: pointer safety audit (Miri + property-based tests for FFI boundaries).
- **Task 3.4**: fold this spec and its result into the published `docs/formal-spec.md`.

---

## References

1. Lamport, L. (2002). *Specifying Systems: The TLA+ Language and Tools for
   Hardware and Software Engineers.* Addison-Wesley.
2. TLA+ tools: <https://github.com/tlaplus/tlaplus>
3. SOKR source: `src/types.rs` (`SokrVersion`), `src/ffi.rs` (registration),
   `src/registry.rs` (registry).

---

*Formal specification for SOKR Phase 3: Formal Verification Roadmap (v1.0 ABI hardening)*
