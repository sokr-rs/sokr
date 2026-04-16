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

use core::ffi::{c_void, c_char};

// ============================================================================
// C ABI Surface Specification
// ============================================================================

/// Version handshake struct for plugin compatibility negotiation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SokrVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SokrVersion {
    pub const CURRENT: Self = Self {
        major: 0,
        minor: 1,
        patch: 0,
    };
}

/// Result codes for SOKR operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SokrResult {
    Ok = 0,
    CapabilityDenied = 1,
    DispatchFailed = 2,
    Timeout = 3,
    VersionMismatch = 4,
    NoCapableSubstrate = 5,
    InvalidInput = 6,
    InvalidIR = 7,
    NotFound = 8,
    RegistryFull = 9,
}

/// Opaque 128-bit identifier for a computation unit.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SokrComputationId {
    pub high: u64,
    pub low: u64,
}

/// Computation descriptor for capability queries.
#[repr(C)]
pub struct SokrCapabilityQuery {
    pub computation_id: SokrComputationId,
    pub ir_format: *const c_char,
    pub ir_data_ptr: *const c_void,
    pub ir_data_len: usize,
    _padding: [u8; 8],
}

/// Response from a capability query.
#[repr(C)]
pub struct SokrCapabilityResponse {
    pub result: SokrResult,
    pub substrate_id: u64,
    pub estimated_latency_ns: u64,
}

/// Dispatch payload struct.
#[repr(C)]
pub struct SokrDispatchRequest {
    pub computation_id: SokrComputationId,
    pub substrate_id: u64,
    pub ir_data_ptr: *const c_void,
    pub ir_data_len: usize,
    pub params_ptr: *const c_void,
    pub params_len: usize,
}

/// Response from a dispatch request.
#[repr(C)]
pub struct SokrDispatchResponse {
    pub result: SokrResult,
    pub completion_token: SokrCompletionToken,
}

/// Opaque 64-bit completion handle.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SokrCompletionToken {
    pub handle: u64,
}

/// Query for completion status.
#[repr(C)]
pub struct SokrCompletionQuery {
    pub completion_token: SokrCompletionToken,
    pub timeout_ns: u64,
    _padding: [u8; 8],
}

/// Completion status signal.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SokrCompletionSignal {
    Pending = 0,
    Complete = 1,
    Failed = 2,
    TimedOut = 3,
}

// ============================================================================
// Plugin VTable
// ============================================================================

/// Function pointer types for substrate plugin operations.
pub type SokrCapabilityFn = extern "C" fn(
    version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult;

pub type SokrDispatchFn = extern "C" fn(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult;

pub type SokrCompletionFn = extern "C" fn(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult;

pub type SokrDestroyFn = extern "C" fn();

/// VTable struct for substrate plugins.
#[repr(C)]
pub struct SokrSubstratePlugin {
    pub version: SokrVersion,
    pub capability_fn: SokrCapabilityFn,
    pub dispatch_fn: SokrDispatchFn,
    pub completion_fn: SokrCompletionFn,
    pub destroy_fn: SokrDestroyFn,
    _padding: [u8; 16],
}
