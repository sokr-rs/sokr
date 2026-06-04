# SOKR Plans.md

**Project**: SOKR — Sovereign Open Kernel Runtime
**Current Version**: v0.3.0 (ABI Frozen)
**Created**: 2026-06-04

---

## Phase 0–2: Foundation & ABI Stable ✅ COMPLETE

Phases 0–2 complete as of v0.3.0:
- Phase 0: Identity, design docs, tooling
- Phase 1: Core skeleton, routing, registry, tests
- Phase 2: ABI stable, integration tested, benchmarked

See [TODO.md](TODO.md) for historical completion details.

---

## Phase 3: Formal Verification Roadmap → v1.0

**Goal**: Back sovereignty claim with proof, not just philosophy.

**Parent tracking issue**: [#4 Pre-1.0 ABI hardening](https://github.com/sokr-rs/sokr/issues/4)

| Task | Description | DoD | Depends | Status |
|------|-------------|-----|---------|--------|
| 3.1  | Survey seL4 capability model for applicable formal methods | Research doc in docs/research/: seL4 overview + capability model + proof techniques + 3-phase action plan for SOKR (TLA+, pointer audit, optional Alloy) | - | cc:done |
| 3.2  | Specify version handshake protocol in TLA+ or Alloy | TLA+ spec docs/formal_spec.tla PASSES TLC with committed docs/formal_spec.cfg; TLC output pasted into formal-spec-guide.md | 3.1 | cc:done |
| 3.2a | Fix TLA+ spec so it runs in TLC | NULL as model value; EXTENDS FiniteSets; valid set-builder; no in-Next Stutter; Next over constants with small bounds; no `[]AllInvariants` in Spec; valid module/filename (formal_spec, not formal-spec) | 3.2 | cc:done |
| 3.2b | Add TLC config and run | docs/formal_spec.cfg committed; TLC run (2.19): 709 distinct states, depth 4, 6 invariants hold; negative control confirms non-vacuous | 3.2a | cc:done |
| 3.2c | Reconcile theorem claims | Unprovable THEOREMs removed; checkable decision rule kept as CompatibilityDecisionInvariant; dead state removed (timestamp, version_mismatch_detected, IsValidVersion, CORE_PATCH) | 3.2a | cc:done |
| 3.3  | Verify ABI memory safety invariants with Miri and property-based tests | CI job runs Miri on all FFI code; property-based tests for invalid pointers, misaligned IR, oversized buffers; all checks pass | 3.1 | cc:todo |
| 3.4  | Publish formal specification as `docs/formal-spec.md` | Markdown doc: scope, assumptions, verified invariants, reference to TLA+/Alloy model | 3.2, 3.3 | cc:todo |

---

## Maintenance & Ongoing

| Task | Description | DoD | Depends | Status |
|------|-------------|-----|---------|--------|
| M.1  | Keep dependencies current via Dependabot | Weekly Cargo + GitHub Actions updates; no security warnings in `cargo audit` | - | cc:wip |
| M.2  | Triage and respond to community discussions | Monitor GitHub Discussions; respond to plugin/architecture questions within 48h | - | cc:todo |

---

## Post-v1.0 Backlog

- Phase 4: Advanced substrate research (QPU, neuromorphic, photonic proof-of-concept)
- Phase 5: Language binding ecosystem (Python, Go, C++, others)
- Architecture evolution: inline IR representation, dispatch policy extensions

See [sokr-plugins TODO.md](https://github.com/sokr-rs/sokr-plugins/blob/main/TODO.md) for plugin roadmap.

---

## Review Notes

- **2026-06-04 — Task 3.2 walked back `cc:done` → `cc:wip`.** Review found the
  committed TLA+ spec (`docs/formal-spec.tla`) was never run and will not parse in
  TLC: `NULL` undefined, `Cardinality` used without `EXTENDS FiniteSets`, invalid
  `WHERE` set-builder (line 195), no `.cfg` file, and `Next` hardcodes `1..999999`
  (intractable state space). The four `THEOREM`s are not checked by anything (TLC
  ignores `THEOREM`; none have TLAPS proofs), so "theorems proven" was inaccurate.
  **Credit:** the compatibility logic faithfully mirrors `check_compatible`
  (`src/types.rs:65`) as enforced at registration in `src/ffi.rs:136`. Split into
  3.2a/3.2b/3.2c to make "passes model checker" real.
- **2026-06-04 — Task 3.2 re-completed and verified.** Rewrote spec as
  `docs/formal_spec.tla` (+`formal_spec.cfg`); the old hyphenated `formal-spec.tla`
  could never host a TLA+ module (hyphens illegal; module name must match
  filename). TLC 2.19 passes: 709 distinct states, depth 4, 6 invariants hold. A
  negative-control run (asserting `active = {}`) was correctly flagged with a
  counterexample, confirming the check is non-vacuous.

## Notes

- **No active substrate work in core.** Substrate plugins live in [sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins).
- **DCO sign-off required** on all commits (`git commit -s`).
- **RFC required** for core ABI changes; no RFC needed for plugin or doc changes.
- **License**: MIT OR Apache-2.0. All contributions assigned to The SOKR Project.
