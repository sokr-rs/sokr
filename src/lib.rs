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

/// ## Thread Safety Contract
///
/// All SOKR C ABI functions are **thread-safe** and may be called concurrently
/// from any thread. Implementations must provide their own synchronization:
///
/// | Function | Thread Safety Requirement |
/// |----------|---------------------------|
/// | `capability_fn` | Must be safe to call concurrently. Substrate implementation must synchronize internal state access. |
/// | `dispatch_fn` | Must be safe to call concurrently. May be called while other dispatches are in-flight. |
/// | `completion_fn` | Must be safe to call concurrently from any thread. Must not block indefinitely. |
/// | `destroy_fn` | **Must only be called once**, after all other operations have completed. Caller must ensure no concurrent calls are in-flight. |
///
/// ### Concurrency Guarantees
///
/// - The core may call `capability_fn` concurrently from multiple threads
/// - The core may call `dispatch_fn` while `completion_fn` is being called for other tokens
/// - The core guarantees `destroy_fn` is only called after all pending operations complete
/// - Plugin implementations must not assume any particular calling thread
///
/// ## Ownership Semantics
///
/// ### Pointer Fields
///
/// All pointer fields in query/request structs follow these rules:
///
/// | Field | Ownership | Validity | Null Allowed |
/// |-------|-----------|----------|--------------|
/// | `ir_format` | Borrowed by callee | Valid for duration of call | No - must point to null-terminated C string |
/// | `ir_data_ptr` | Borrowed by callee | Valid for duration of call | Yes - if `ir_data_len` is 0 |
/// | `params_ptr` | Borrowed by callee | Valid for duration of call | Yes - if `params_len` is 0 |
/// | `response` | Borrowed by callee for write | Valid, non-null, properly aligned | No - must be valid writable pointer |
/// | `signal` | Borrowed by callee for write | Valid, non-null, properly aligned | No - must be valid writable pointer |
///
/// ### Allocation Contract
///
/// - **Core allocates**: Query/request structs on stack before calling plugin
/// - **Plugin writes**: Response data to provided output pointers
/// - **No heap allocation in core**: The core never allocates heap memory for plugin use
/// - **Plugin manages its own heap**: Substrate plugins may allocate internally, but must not expose allocation details through ABI
/// - **No transfer of ownership**: Pointers in structs are always borrowed, never transferred
///
/// ### Lifetime Rules
///
/// 1. Input pointers are valid only for the duration of the function call
/// 2. Output pointers must remain valid until the function returns
/// 3. The plugin must not retain references to input data after returning
/// 4. The core must not modify data through input pointers after calling
///
/// ### Error Handling
///
/// - All functions return `SokrResult` - check this before reading output values
/// - Output struct contents are undefined if result is not `Ok`
/// - `InvalidInput` is returned for null required pointers or invalid alignment

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
