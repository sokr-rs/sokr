# seL4 Formal Verification: Applicability to SOKR

**Date**: June 2026
**Status**: Phase 3 Research (Task 3.1)
**Summary**: Survey of seL4's formal methods approach and recommendations for SOKR's pre-1.0 ABI hardening.

---

## Executive Summary

seL4 is the first OS microkernel with end-to-end formal verification proving memory safety, information flow security, and functional correctness via a three-layer refinement proof stack (C code → Isabelle/HOL abstract spec → user-level properties). The proof effort was substantial: 200k lines of Isabelle/HOL for 8,500 lines of C kernel code, requiring 5+ years and 2–3 proof engineers.

**Key constraint**: seL4 assumes hardware (CPU, MMU) behaves per ISA specification. Multi-core, timing, and device I/O are **not** formally verified.

**For SOKR**: seL4's full refinement stack is **overkill**. SOKR is a stateless library with a fixed-size registry and no hardware coupling—orders of magnitude simpler than a full kernel. Instead, adopt a pragmatic, **TLA+ + Alloy + Miri** approach (1–1.5k lines total, 4–8 weeks, automated model checking, fast iteration).

---

## What seL4 Proves

| Layer | What | Tools | Scope |
|-------|------|-------|-------|
| **C Code** | No buffer overflows, UB, wild pointers, use-after-free | CompCert + C semantics | x86, ARM |
| **Abstract Spec** | Functional correctness: IPC respects capabilities, message passing is atomic | Isabelle/HOL (200k lines) | Untimed, single-threaded |
| **Refinement** | C code correctly implements abstract spec | Isabelle/HOL refinement logic | Bridges code ↔ spec via stuttering equivalence |

**Core invariants**: Information flow security, capability integrity, message passing atomicity, exception correctness, no undefined behavior.

---

## seL4's Capability Model

A capability is an **unforgeable reference** to a kernel object with delegable access rights:

- **Unforgeable**: User code cannot create a capability; kernel alone hands them out
- **Delegable**: Parent → child via Copy (same rights), Mint (add badge), Grant (further delegate)
- **Revocable**: Kernel maintains a **derivation tree**; Revoke atomically destroys descendants

**Key difference from SOKR**: seL4's fine-grained delegation tree is unnecessary for SOKR's static registry model. SOKR plugins register once, version-checked once, and deregister explicitly—no delegation or revocation tree needed.

---

## Proof Techniques

1. **Refinement Mapping**: Each abstract operation maps to C instruction sequences; prove equivalence up to stuttering
2. **Inductive Invariants**: Loop/state invariants proven at each level
3. **Stratification**: Generic properties proven once, ISA-specific properties per architecture
4. **CompCert Guarantee**: C code executes per ISO C semantics (eliminates memory-safety bugs as proof failure source)

**Scale**: 5+ years, 6–12 month maintenance burden per kernel change.

---

## Critical Scope Limitations

**NOT formally verified**:
- **Hardware**: CPU cache, memory ordering, speculative execution (Meltdown/Spectre outside threat model)
- **Timing**: Covert channels, interrupt latency
- **Concurrency**: Multi-core kernel execution (single-core proofs only)
- **Device I/O**: DMA, firmware, device drivers (plugged in at boundary)
- **Privileged mode**: Assumes CPU mode switch (user ↔ kernel) works per ISA

**Bottom line**: seL4 proves *kernel semantics* given correct hardware—not hardware correctness.

---

## Recommended for SOKR (Tier 1)

### 1. Formalize Version Handshake Invariant (TLA+)

**What**: Prove "A plugin with version (major, minor_p) can only execute on core with version (major, minor_c) where minor_p ≤ minor_c"

**Why**: Version mismatch breaks ABI contracts

**Effort**: 1–2 weeks, ~500 lines TLA+

**Tool**: TLA+ Checker (automated model checking, free, fast)

### 2. Formalize Handle Validity Contract (TLA+)

**What**: Prove:
- `handle = 0` is always invalid
- If `sokr_dispatch` returns `NoCapableSubstrate`, handle is 0
- If handle is 0, `sokr_completion` returns `InvalidInput`

**Why**: Handle lifecycle bugs corrupt completion queries

**Effort**: 1 week, ~300 lines TLA+

### 3. Registry Integrity (TLA+)

**What**: Prove:
- Once a plugin is at index i, it remains until deregistered
- `destroy_fn` called exactly once on deregistration
- `substrate_id` is never 0 after registration

**Effort**: 1 week, ~200 lines TLA+

### 4. Pointer Safety Audit (Miri + Tests)

**What**: Audit `src/ffi.rs` for pointer validity (null, alignment, bounds, lifetime)

**Effort**: 1 week

**Actions**:
- Add Miri to CI (catches undefined behavior at test time)
- Property-based tests: invalid pointers, misaligned IR, oversized IR
- Document pointer lifetime contracts in function comments

**Total Phase 1**: 3–4 weeks, ~1000 lines of formal specs + test suite, minute-scale model checking iteration.

---

## NOT Recommended for SOKR (Tier 3)

### 1. Full Refinement Proof Stack (Isabelle/HOL)

**Cost**: 6–12 months, 2–3 proof engineers

**Benefit**: **Marginal** because:
- SOKR is a library, not a kernel (no hardware, interrupts, privilege mode)
- Registry has zero hardware coupling
- Fixed-size array eliminates most memory-safety bugs upfront

**Only pursue if** aerospace/medical/defense deployment + contractual requirement.

### 2. Capability Delegation Tree

**Why skip**: SOKR has no delegation. Modeling seL4's tree over-engineers the proof.

### 3. Multi-Core Concurrency Proofs

**Why skip**: SOKR core is stateless per dispatch. Concurrency delegated to plugins. Plugin thread-safety is *their* responsibility.

---

## Recommended 3-Phase Action Plan

### Phase 1 (Weeks 1–2): TLA+ Formalization

**Deliverable**: `docs/formal-spec.tla` with model checker results

**Time**: 8–10 person-days

**Tools**: TLA+ Toolbox (free)

### Phase 2 (Weeks 3–4): Pointer Safety Audit

**Deliverable**: FFI safety report + updated code + Miri CI job

**Time**: 5–7 person-days

**Tools**: Miri, loom, cargo-fuzz

### Phase 3 (Weeks 5–7, Optional): Alloy Model of Dispatch

**Deliverable**: `docs/formal-spec.als` with SAT solver results

**Time**: 10–14 person-days

**Tools**: Alloy Analyzer (free, academic)

---

## Comparison: seL4 vs. SOKR

| Property | seL4 | SOKR (Recommended) |
|----------|------|-------------------|
| **Threat Model** | Kernel (privilege, IPC, resources) | Library (dispatch, registry, version) |
| **Verification** | C + spec + refinement proofs | Invariants + pointer contracts (TLA+/Alloy) |
| **Concurrency** | Single-core, ints disabled | Stateless; plugins own concurrency |
| **Capability** | Dynamic tree (delegation/revocation) | Static registry (no delegation) |
| **Proof Scale** | 200k lines Isabelle/HOL | 1–1.5k lines TLA+/Alloy |
| **Tools** | Interactive (Isabelle/HOL) | Automated (TLA+, Alloy, Miri) |
| **Hardware Assumptions** | CPU per ISA spec | None (library) |
| **Maintenance** | High (re-prove per change) | Low (fast model checking) |
| **Time to Deploy** | 6–12 months | 4–8 weeks |

---

## Key Takeaways

1. **seL4's proof stack is 10–20x overkill for SOKR**. SOKR is a stateless library; seL4 is a full kernel with hardware interaction.

2. **TLA+ and Alloy are right-sized**. Automated model checkers find edge cases fast. 1–1.5k lines total for core invariants. Re-check after code change in minutes, not months.

3. **Pointer safety is SOKR's primary boundary risk**. Unsafe code already confined to `src/ffi.rs` (good practice). Miri + property tests catch UB early.

4. **Delegation and revocation are not SOKR concerns**. SOKR has no capability tree. Skip seL4's delegation model entirely.

5. **Version handshake is the highest-leverage proof target**. Forward/backward compatibility is critical. TLA+ can prove the compatibility rule is sufficient.

---

## References

1. Klein, et al. (2009). "seL4: Formal Verification of an OS Kernel." *PLDI*. Introduces three-layer proof stack, refinement methodology, CompCert integration.

2. Elphinstone & Heiser (2013). "From L3 to seL4: What Have We Learnt?" *SOSP*. Lessons on verifiability and architecture.

3. seL4 Foundation. "seL4 Reference Manual." https://sel4.systems/. Current specification, capability model, syscall contracts.

4. Lamport, L. (2002). "Specifying Systems: The TLA+ Language and Tools for Hardware and Software Engineers." Addison-Wesley. TLA+ primer; recommended reading for Phase 1.

5. Jackson, D. (2012). "Software Abstractions: Logic, Language, and Analysis." MIT Press. Alloy language and model-checking techniques.

---

## Next Steps

1. **Confirm interest in Phase 1 (TLA+ formalization)**. Timeline preference? (4–8 weeks effort)
2. **Should Miri be added to CI immediately**, or after Phase 1?
3. **Any specific version compatibility edge case** to explore first? (e.g., major=0 handling?)

---

*Research prepared for SOKR Phase 3: Formal Verification Roadmap (v1.0 ABI hardening)*
