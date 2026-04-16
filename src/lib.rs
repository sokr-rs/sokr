//! # SOKR — Sovereign Open Kernel Runtime
//!
//! **This crate is in early design phase. No API is stable.**
//!
//! SOKR is a sovereign compute runtime where the core is immutable
//! and everything else is a plugin: IR, substrate backends, language
//! bindings, and dispatch policy.
//!
//! The core exposes exactly three operations:
//! - **Capability** — can this substrate fulfill this computation?
//! - **Dispatch** — fulfill it
//! - **Completion** — signal when fulfilled
//!
//! No assumption is made about memory model, parallelism, execution
//! time, or computation representation. Any substrate that can answer
//! three questions is a valid SOKR backend, including substrates that
//! do not yet exist.

#![no_std]
