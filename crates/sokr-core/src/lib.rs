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

#![cfg_attr(not(test), no_std)]

use core::ffi::{c_char, c_void};

#[cfg(not(test))]
use core::panic::PanicInfo;

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
///
/// Version handshake struct for plugin compatibility negotiation.
///
/// ## Version Compatibility Rules
///
/// | Core Version | Plugin Version | Compatible? | Reason |
/// |--------------|----------------|-------------|--------|
/// | 1.2.3 | 1.1.0 | ✅ Yes | Same major, plugin minor ≤ core minor |
/// | 1.2.3 | 1.2.0 | ✅ Yes | Same major, same minor |
/// | 1.2.3 | 1.3.0 | ❌ No | Plugin minor > core minor |
/// | 1.2.3 | 2.0.0 | ❌ No | Major version mismatch |
/// | 1.2.3 | 0.9.0 | ❌ No | Major version mismatch |
///
/// ## Negotiation Sequence
///
/// 1. Core sends its version pointer as first argument to `capability_fn`
/// 2. Plugin inspects core version and determines compatibility
/// 3. Plugin returns `VersionMismatch` if incompatible (never panics)
/// 4. On success, plugin fills response and returns `Ok`
///
/// ## Forward Compatibility
///
/// - Newer plugin on older core: Plugin must check and return `VersionMismatch`
/// - Older plugin on newer core: Allowed if major matches and plugin minor ≤ core minor
///
/// ## Version Bump Triggers
///
/// - **Major**: Any breaking change to C ABI (struct layout, function signatures)
/// - **Minor**: New features, new result codes, new optional fields (backwards compatible)
/// - **Patch**: Documentation fixes, implementation corrections (no ABI change)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SokrVersion {
    /// Major version - must match between core and plugin.
    pub major: u32,
    /// Minor version - plugin must be ≤ core.
    pub minor: u32,
    /// Patch version - informational only.
    pub patch: u32,
}

impl SokrVersion {
    /// Current SOKR core ABI version (0.1.0).
    pub const CURRENT: Self = Self {
        major: 0,
        minor: 1,
        patch: 0,
    };

    /// Check if this plugin version is compatible with the given core version.
    ///
    /// Returns `SokrResult::Ok` if compatible, `SokrResult::VersionMismatch` otherwise.
    /// This function never panics - incompatible versions are handled gracefully.
    ///
    /// # Compatibility Rules
    /// - `plugin.major` must equal `core.major`
    /// - `plugin.minor` must be ≤ `core.minor`
    /// - `patch` is ignored for compatibility (informational only)
    #[must_use]
    pub const fn check_compatible(self, core: Self) -> SokrResult {
        if self.major != core.major {
            return SokrResult::VersionMismatch;
        }
        if self.minor > core.minor {
            return SokrResult::VersionMismatch;
        }
        SokrResult::Ok
    }
}

/// Result codes for SOKR operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SokrResult {
    /// Operation succeeded.
    Ok = 0,
    /// Substrate cannot fulfill this computation.
    CapabilityDenied = 1,
    /// Dispatch failed at runtime.
    DispatchFailed = 2,
    /// Operation timed out.
    Timeout = 3,
    /// Plugin ABI version incompatible with core.
    VersionMismatch = 4,
    /// No registered substrate can fulfill this computation.
    NoCapableSubstrate = 5,
    /// Invalid input parameters.
    InvalidInput = 6,
    /// Invalid IR format.
    InvalidIR = 7,
    /// Resource not found.
    NotFound = 8,
    /// Plugin registry is full.
    RegistryFull = 9,
}

/// Opaque 128-bit identifier for a computation unit.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SokrComputationId {
    /// High 64 bits of the identifier.
    pub high: u64,
    /// Low 64 bits of the identifier.
    pub low: u64,
}

/// Computation descriptor for capability queries.
#[repr(C)]
pub struct SokrCapabilityQuery {
    /// Computation to query capability for.
    pub computation_id: SokrComputationId,
    /// IR format identifier (null-terminated C string).
    pub ir_format: *const c_char,
    /// Pointer to IR data.
    pub ir_data_ptr: *const c_void,
    /// Length of IR data in bytes.
    pub ir_data_len: usize,
    _padding: [u8; 8],
}

/// Response from a capability query.
#[repr(C)]
pub struct SokrCapabilityResponse {
    /// Result of the capability query.
    pub result: SokrResult,
    /// Substrate that can fulfill this computation (if capable).
    pub substrate_id: u64,
    /// Estimated latency in nanoseconds (0 if unknown).
    pub estimated_latency_ns: u64,
}

/// Dispatch payload struct.
#[repr(C)]
pub struct SokrDispatchRequest {
    /// Computation to dispatch.
    pub computation_id: SokrComputationId,
    /// Substrate to dispatch to.
    pub substrate_id: u64,
    /// Pointer to IR data.
    pub ir_data_ptr: *const c_void,
    /// Length of IR data in bytes.
    pub ir_data_len: usize,
    /// Pointer to dispatch parameters.
    pub params_ptr: *const c_void,
    /// Length of parameters in bytes.
    pub params_len: usize,
}

/// Response from a dispatch request.
#[repr(C)]
pub struct SokrDispatchResponse {
    /// Result of the dispatch request.
    pub result: SokrResult,
    _padding: u32,
    /// Token to query completion status.
    pub completion_token: SokrCompletionToken,
}

/// Opaque 64-bit completion handle.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SokrCompletionToken {
    /// Opaque handle identifying this completion.
    pub handle: u64,
}

/// Query for completion status.
#[repr(C)]
pub struct SokrCompletionQuery {
    /// Completion token to query.
    pub completion_token: SokrCompletionToken,
    /// Timeout in nanoseconds (0 for no timeout).
    pub timeout_ns: u64,
    _padding: [u8; 8],
}

/// Completion status signal.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SokrCompletionSignal {
    /// Operation is still pending.
    Pending = 0,
    /// Operation completed successfully.
    Complete = 1,
    /// Operation failed.
    Failed = 2,
    /// Operation timed out.
    TimedOut = 3,
}

// ============================================================================
// Plugin VTable
// ============================================================================

/// Capability query function pointer type.
pub type SokrCapabilityFn = extern "C" fn(
    version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult;

/// Dispatch function pointer type.
pub type SokrDispatchFn = extern "C" fn(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult;

/// Completion query function pointer type.
pub type SokrCompletionFn = extern "C" fn(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult;

/// Cleanup function called when a plugin is deregistered.
pub type SokrDestroyFn = extern "C" fn();

/// `VTable` struct for substrate plugins.
#[repr(C)]
pub struct SokrSubstratePlugin {
    /// Plugin ABI version for compatibility check.
    pub version: SokrVersion,
    /// Capability query function pointer.
    pub capability_fn: SokrCapabilityFn,
    /// Dispatch function pointer.
    pub dispatch_fn: SokrDispatchFn,
    /// Completion query function pointer.
    pub completion_fn: SokrCompletionFn,
    /// Cleanup function called on deregistration.
    pub destroy_fn: SokrDestroyFn,
    _padding: [u8; 16],
}

// ============================================================================
// Panic Handler (required for no_std)
// ============================================================================

/// Panic handler for `no_std` environment.
/// In a real implementation, this would log or signal the panic appropriately.
#[cfg(not(test))]
#[allow(clippy::missing_const_for_fn)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
