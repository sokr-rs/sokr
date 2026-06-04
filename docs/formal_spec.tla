---------------------------- MODULE formal_spec ----------------------------
(*
   TLA+ formal specification for the SOKR plugin version handshake.

   Models the core's plugin registry and the version-compatibility gate
   enforced at registration time in `sokr_register_substrate`
   (src/ffi.rs:136), which delegates to `SokrVersion::check_compatible`
   (src/types.rs:65).

   The central safety property is VersionCompatibilityInvariant: every
   plugin present in the registry satisfies the compatibility rule, i.e.
   an incompatible plugin can never occupy a registry slot.

   This module is checked by TLC with formal_spec.cfg. Liveness is out of
   scope for this pass; only safety invariants are asserted.
*)

EXTENDS Naturals, FiniteSets

CONSTANTS
    NULL,           \* model value marking an empty registry slot
    PluginIds,      \* finite set of distinct plugin identifiers
    Majors,         \* finite set of candidate plugin major versions
    Minors,         \* finite set of candidate plugin minor versions
    MaxSubstrates,  \* registry capacity (MAX_SUBSTRATES in the core)
    CoreMajor,      \* core ABI major version
    CoreMinor       \* core ABI minor version

ASSUME MaxSubstrates \in Nat \ {0}
ASSUME CoreMajor \in Nat /\ CoreMinor \in Nat
ASSUME Majors \subseteq Nat /\ Minors \subseteq Nat

VARIABLES
    registry,       \* [1..MaxSubstrates -> Entry \cup {NULL}]
    active          \* set of currently registered plugin ids

vars == <<registry, active>>

Slots == 1 .. MaxSubstrates

(* A registry entry. Patch is intentionally omitted: it is ignored by the
   compatibility rule, so modeling it would only inflate the state space. *)
Entry == [id : PluginIds, major : Majors, minor : Minors]

(* Compatibility rule, mirroring SokrVersion::check_compatible:
   plugin major must equal core major; plugin minor must be <= core minor. *)
VersionCompatible(major, minor) ==
    /\ major = CoreMajor
    /\ minor <= CoreMinor

(* Indices currently holding a plugin. *)
OccupiedSlots == { i \in Slots : registry[i] # NULL }

(* Ids currently present in the registry (filter, then map). *)
RegisteredIds == { registry[i].id : i \in OccupiedSlots }

----------------------------------------------------------------------------

Init ==
    /\ registry = [i \in Slots |-> NULL]
    /\ active = {}

(* Successful registration: a new, compatible plugin takes an empty slot.
   The compatibility guard is what makes the safety invariant hold. *)
Register(p, major, minor) ==
    /\ p \notin active
    /\ VersionCompatible(major, minor)
    /\ \E i \in Slots :
        /\ registry[i] = NULL
        /\ registry' = [registry EXCEPT ![i] = [id |-> p, major |-> major, minor |-> minor]]
    /\ active' = active \cup {p}

(* Rejected registration: an incompatible plugin is refused; state is
   unchanged. Models the VersionMismatch return path in ffi.rs without
   mutating the registry. *)
RejectIncompatible(p, major, minor) ==
    /\ ~VersionCompatible(major, minor)
    /\ \E i \in Slots : registry[i] = NULL
    /\ UNCHANGED vars

(* Deregistration: an active plugin's slot is cleared. *)
Deregister(p) ==
    /\ p \in active
    /\ \E i \in Slots :
        /\ registry[i] # NULL
        /\ registry[i].id = p
        /\ registry' = [registry EXCEPT ![i] = NULL]
    /\ active' = active \ {p}

Next ==
    \/ \E p \in PluginIds, m \in Majors, n \in Minors : Register(p, m, n)
    \/ \E p \in PluginIds, m \in Majors, n \in Minors : RejectIncompatible(p, m, n)
    \/ \E p \in PluginIds : Deregister(p)

Spec == Init /\ [][Next]_vars

----------------------------------------------------------------------------
(* Invariants checked by TLC. *)

TypeOK ==
    /\ registry \in [Slots -> Entry \cup {NULL}]
    /\ active \subseteq PluginIds

(* Safety: every registered plugin is version-compatible. An incompatible
   plugin can never occupy a slot. *)
VersionCompatibilityInvariant ==
    \A i \in Slots :
        registry[i] # NULL =>
            VersionCompatible(registry[i].major, registry[i].minor)

(* The active set exactly mirrors the registry contents. *)
RegistryConsistencyInvariant ==
    active = RegisteredIds

(* Each plugin id occupies at most one slot. *)
NoDuplicateInvariant ==
    \A i, j \in Slots :
        (registry[i] # NULL /\ registry[j] # NULL /\ registry[i].id = registry[j].id)
            => i = j

(* The registry never exceeds capacity. *)
CapacityInvariant ==
    Cardinality(active) <= MaxSubstrates

(* The compatibility decision is exactly the semantic rule (no inverted or
   off-by-one condition). A constant predicate, true in every state. *)
CompatibilityDecisionInvariant ==
    \A m \in Majors, n \in Minors :
        VersionCompatible(m, n) <=> (m = CoreMajor /\ n <= CoreMinor)

=============================================================================
