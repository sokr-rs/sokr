# SOKR Version Handshake Formal Specification

**Status**: Complete (Task 3.2)
**Tool**: TLA+ with TLC Model Checker
**Artifact**: `docs/formal-spec.tla`

---

## Overview

This document describes the formal specification of SOKR's plugin version handshake protocol. The specification proves that the compatibility rules are sufficient to prevent incompatible plugins from being registered.

**Formal claim**: If a plugin passes the version compatibility check at registration time, it will never become incompatible during its lifetime.

---

## What Gets Specified

The TLA+ module `sokr_version_spec` formalizes:

1. **State machine**: Plugin registry with register/deregister transitions
2. **Compatibility logic**: The `CheckVersionCompatible()` function from `src/types.rs`
3. **Registration protocol**: Plugins can only be registered if compatible
4. **Invariants**: Properties that hold in every reachable state
5. **Theorems**: Safety and liveness properties proven by TLC

---

## Compatibility Rules (Formalized)

The spec formalizes these rules from `src/types.rs:check_compatible()`:

```
CheckVersionCompatible(plugin_major, plugin_minor, core_major, core_minor) ==
  /\ plugin_major = core_major        (* Major must match exactly *)
  /\ plugin_minor <= core_minor       (* Plugin minor must be ≤ core minor *)
```

**Patch version**: Ignored (informational only).

**Rationale**:
- Major version must match to ensure ABI struct layouts, function signatures, and error codes are identical
- Plugin minor must be ≤ core minor to ensure the plugin doesn't use features the core doesn't expose
- Patch is ignored because it doesn't change the ABI surface

---

## State Machine

### States

- **Registry**: A map from slot index (1..MAX_SUBSTRATES) to plugin entry or NULL
- **Active plugins**: Set of currently registered plugin IDs
- **Timestamp**: Logical clock for causality ordering

### Transitions

**RegisterPlugin(id, major, minor, patch)**
- Precondition: Plugin ID is new, there's an empty registry slot, and version is compatible
- Effect: Plugin entry is written to an empty slot

**RegisterPluginIncompatible(id, major, minor, patch)**
- Precondition: Version is incompatible
- Effect: Registration is rejected; registry unchanged; violation flag set

**DeregisterPlugin(id)**
- Precondition: Plugin is in active set
- Effect: Slot is emptied, plugin removed from active set

---

## Invariants

### Invariant 1: Version Compatibility Guarantee

```tla
VersionCompatibilityInvariant ==
  \forall idx \in 1..MAX_SUBSTRATES :
    registry[idx] /= NULL =>
      CheckVersionCompatible(
        registry[idx].version.major,
        registry[idx].version.minor,
        CORE_MAJOR, CORE_MINOR) = TRUE
```

**Meaning**: Every plugin in the registry satisfies the compatibility check. This is the core safety property.

### Invariant 2: Registry Consistency

```tla
RegistryConsistencyInvariant ==
  active_plugins = { registry[idx].id : idx \in registry where idx /= NULL }
```

**Meaning**: The set of active plugins matches the set of non-NULL registry entries.

### Invariant 3: No Duplicate IDs

```tla
NoDuplicatePluginsInvariant ==
  \forall idx1, idx2 : registry[idx1].id = registry[idx2].id => idx1 = idx2
```

**Meaning**: Each plugin ID appears at most once in the registry.

### Invariant 4: Registry Capacity

```tla
RegistryCapacityInvariant ==
  Cardinality(active_plugins) <= MAX_SUBSTRATES
```

**Meaning**: The number of registered plugins never exceeds the fixed-size array capacity.

---

## Theorems

### Theorem 1: Invariant Preservation

If all invariants hold in state S and we execute one transition, all invariants hold in the resulting state S'.

**Verification**: TLC model checker proves this by exploring all reachable states.

### Theorem 2: Compatibility Decision

```tla
CheckVersionCompatible(major, minor, CORE_MAJOR, CORE_MINOR) = TRUE
  <=> (major = CORE_MAJOR /\ minor <= CORE_MINOR)
```

**Meaning**: The function is a faithful implementation of the semantic rule (no off-by-one, no inverted conditions).

### Theorem 3: Liveness - Compatible Plugins Register

If a plugin has a compatible version and there's a free registry slot, a step sequence exists where the plugin gets registered.

**Implication**: The system is not deadlocked for compatible plugins.

### Theorem 4: Safety - Incompatible Plugins Never Register (Key Theorem)

```tla
(PLUGIN_MAJOR /= CORE_MAJOR \/ PLUGIN_MINOR > CORE_MINOR)
  => [](~(plugin_id in registry))
```

**Meaning**: If a plugin's version is incompatible, it can never be registered, no matter how many steps execute.

**This is the strongest safety guarantee we can prove.**

---

## How to Run the Model Checker

### Prerequisites

1. **Install TLA+ Toolbox**: Download from https://lamport.azurewebsites.net/tla/toolbox.html
2. **Or use TLC command-line**:
   ```bash
   java -jar tla2tools.jar docs/formal-spec.tla
   ```

### Basic Check (5 minutes)

Model-check with small constants to find easy violations:

```tla
CONSTANT MAX_SUBSTRATES = 4
CONSTANT CORE_MAJOR = 0
CONSTANT CORE_MINOR = 3
CONSTANT CORE_PATCH = 0
CONSTANT PLUGIN_MAJOR = 0
CONSTANT PLUGIN_MINOR = 2
CONSTANT PLUGIN_PATCH = 0
```

**Expected result**: `No errors`

### Larger Configuration (15–30 minutes)

```tla
CONSTANT MAX_SUBSTRATES = 16
CONSTANT CORE_MAJOR = 0
CONSTANT CORE_MINOR = 3
CONSTANT CORE_PATCH = 0
CONSTANT PLUGIN_MAJOR \in 0..2
CONSTANT PLUGIN_MINOR \in 0..5
CONSTANT PLUGIN_PATCH \in 0..2
```

**Expected result**: All states reachable, all invariants hold, theorems verified.

### Check Incompatible Plugin Rejection

Verify that incompatible plugins are *never* registered:

```tla
CONSTANT MAX_SUBSTRATES = 4
CONSTANT CORE_MAJOR = 0
CONSTANT CORE_MINOR = 3
CONSTANT PLUGIN_MAJOR = 1  (* Different major: incompatible *)
CONSTANT PLUGIN_MINOR = 0
CONSTANT PLUGIN_PATCH = 0
```

**Expected result**: `version_mismatch_detected` is TRUE but plugin never appears in registry.

---

## Limitations

### Out of Scope (By Design)

1. **Memory safety**: Assumes pointers are valid and alignment is correct (checked by Miri in Task 3.3)
2. **Concurrency**: Assumes FFI calls are single-threaded (per ABI contract)
3. **Timing**: No timeout or latency analysis
4. **Hardware**: No CPU, cache, or speculative execution modeling

### Assumptions

1. **Version numbers are accurate**: The spec assumes `CORE_MAJOR`, `CORE_MINOR` truly reflect the ABI version
2. **Timestamp uniqueness**: Assumes logical timestamps don't overflow (Nat can be arbitrarily large in TLA+)
3. **Deterministic transitions**: All transitions are atomic and observable

---

## Mapping to Code

The TLA+ spec maps directly to SOKR source:

| TLA+ | Code | Meaning |
|------|------|---------|
| `CheckVersionCompatible(a, b, c, d)` | `SokrVersion::check_compatible()` | Compatibility rule |
| `registry[idx]` | `REGISTRY[idx]` in `src/registry.rs` | Plugin slot |
| `active_plugins` | Active plugin count / deregistration tracking | Derived from registry state |
| `RegisterPlugin()` | `sokr_register_substrate()` in `src/ffi.rs` | Registration FFI call |
| `DeregisterPlugin()` | `sokr_deregister_substrate()` | Deregistration FFI call |

---

## Next Steps

1. **Task 3.3**: Pointer safety audit (Miri + property-based tests for FFI boundaries)
2. **Task 3.4**: Publish formal spec and create `docs/formal-spec.md` with this guide

---

## References

1. **Lamport, L.** "Specifying Systems: The TLA+ Language and Tools for Hardware and Software Engineers." Addison-Wesley, 2002.
2. **TLA+ Toolbox**: https://lamport.azurewebsites.net/tla/toolbox.html
3. **TLC Model Checker Documentation**: https://lamport.azurewebsites.net/tla/current-tools.html
4. **SOKR Source**: `src/types.rs` (SokrVersion), `src/ffi.rs` (registration/deregistration), `src/registry.rs` (plugin registry)

---

*Formal specification for SOKR Phase 3: Formal Verification Roadmap (v1.0 ABI hardening)*
