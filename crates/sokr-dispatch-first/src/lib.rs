//! SOKR dispatch policy - first capable substrate wins
//!
//! Simple dispatch policy: iterate registered substrates,
//! dispatch to first capable one. Fallback to error if none capable.

#![cfg_attr(not(test), no_std)]
#![allow(missing_docs)] // Placeholder - Phase 1.4 implementation
