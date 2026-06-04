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
| 3.1  | Survey seL4 capability model for applicable formal methods | Research doc: 1–2 page summary of seL4 techniques & applicability to SOKR | - | cc:done |
| 3.2  | Specify version handshake protocol in TLA+ or Alloy | Formal spec (TLA+ or Alloy) for `check_compatible()` logic; passes model checker | 3.1 | cc:todo |
| 3.3  | Verify ABI memory safety invariants with Miri and KLEE | CI job runs Miri + KLEE; all checks pass; test report in CI logs | - | cc:todo |
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

## Notes

- **No active substrate work in core.** Substrate plugins live in [sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins).
- **DCO sign-off required** on all commits (`git commit -s`).
- **RFC required** for core ABI changes; no RFC needed for plugin or doc changes.
- **License**: MIT OR Apache-2.0. All contributions assigned to The SOKR Project.
