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
//! ## Module Structure
//!
//! - [`types`] — C ABI struct and enum definitions (`#[repr(C)]`)
//! - [`registry`] — Plugin registry for substrate management
//! - [`ffi`] — `#[no_mangle] extern "C"` function exports (unsafe)
//!
//! No assumption is made about memory model, parallelism, execution
//! time, or computation representation. Any substrate that can answer
//! three questions is a valid SOKR backend, including substrates that
//! do not yet exist.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(feature = "ffi"), forbid(unsafe_code))]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(feature = "ffi")]
pub mod ffi;
pub mod registry;
pub mod types;

pub use registry::{Registry, MAX_SUBSTRATES};
pub use types::*;

// Panic handler for no_std environment
#[cfg(not(test))]
#[allow(clippy::missing_const_for_fn)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
