//! C ABI type definitions for SOKR core.
//!
//! All types are `#[repr(C)]` for stable C ABI compatibility.

use core::ffi::{c_char, c_void};

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
    /// Current SOKR core ABI version (0.1.1).
    pub const CURRENT: Self = Self {
        major: 0,
        minor: 1,
        patch: 1,
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

/// Static export of current SOKR version for C FFI.
///
/// This is exported as `extern const` (valid at file scope in C),
/// unlike the `SokrVersion_CURRENT` macro which uses compound literals.
#[allow(unsafe_code)]
#[no_mangle]
pub static SOKR_VERSION_CURRENT: SokrVersion = SokrVersion {
    major: 0,
    minor: 1,
    patch: 1,
};

/// Result codes for SOKR operations.
#[repr(u32)]
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

impl SokrResult {
    /// Returns true if the result indicates success.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Returns true if the result indicates an error.
    #[must_use]
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }
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
    /// Reserved padding for ABI alignment.
    pub padding: [u8; 8],
}

/// Response from a capability query.
#[repr(C)]
pub struct SokrCapabilityResponse {
    /// Result of the capability query.
    pub result: SokrResult,
    /// Reserved padding for ABI alignment.
    pub padding: u32,
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
    /// Reserved padding for ABI alignment and future extension.
    pub padding: [u8; 16],
}

/// Response from a dispatch request.
#[repr(C)]
pub struct SokrDispatchResponse {
    /// Result of the dispatch request.
    pub result: SokrResult,
    /// Reserved padding for ABI alignment.
    pub padding: u32,
    /// Token to query completion status.
    pub completion_token: SokrCompletionToken,
}

/// Opaque 64-bit completion handle.
///
/// ## Valid Token Contract
/// - `handle = 0` is reserved as the "invalid / unset" sentinel
/// - Valid tokens are always non-zero (assigned by substrate on successful dispatch)
/// - Callers receiving `handle = 0` on error should not use it for completion queries
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SokrCompletionToken {
    /// Opaque handle identifying this completion.
    /// Value of 0 indicates an invalid or unset token.
    pub handle: u64,
}

/// Query for completion status.
#[repr(C)]
pub struct SokrCompletionQuery {
    /// Completion token to query.
    pub completion_token: SokrCompletionToken,
    /// Timeout in nanoseconds (0 for no timeout).
    pub timeout_ns: u64,
    /// Reserved padding for ABI alignment.
    pub padding: [u8; 8],
}

/// Completion status signal.
#[repr(u32)]
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

/// Capability query function pointer type.
///
/// # Pointer contract
/// All pointers are valid and non-null for the duration of the call.
/// Implementations MUST NOT retain any pointer past return.
/// Return `SokrResult::CapabilityDenied` to disclaim the computation
/// without claiming ownership; any other non-`Ok` result is propagated
/// to the caller as a hard failure.
pub type SokrCapabilityFn = extern "C" fn(
    version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult;

/// Dispatch function pointer type.
///
/// # Pointer contract
/// All pointers are valid and non-null for the duration of the call.
/// Implementations MUST NOT retain any pointer past return.
/// On non-`Ok` return the core zeroes `response`; any error is propagated
/// verbatim to the caller.
pub type SokrDispatchFn = extern "C" fn(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult;

/// Completion query function pointer type.
///
/// # Pointer contract
/// All pointers are valid and non-null for the duration of the call.
/// Implementations MUST NOT retain any pointer past return.
/// Return `SokrResult::NotFound` to disclaim the token without claiming
/// ownership; any other non-`Ok` result is propagated to the caller as
/// a hard failure and `signal` is set to `SokrCompletionSignal::Failed`.
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
    /// Unique identifier for this substrate plugin.
    pub substrate_id: u64,
    /// Reserved padding for ABI alignment.
    pub padding: [u8; 8],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compatible_same() {
        let v = SokrVersion::CURRENT;
        assert!(v.check_compatible(v).is_ok());
    }

    #[test]
    fn version_incompatible_major_mismatch() {
        let plugin = SokrVersion {
            major: 1,
            minor: 0,
            patch: 0,
        };
        let core = SokrVersion {
            major: 0,
            minor: 1,
            patch: 0,
        };
        assert!(plugin.check_compatible(core).is_err());
    }

    #[test]
    fn version_incompatible_plugin_too_new() {
        let plugin = SokrVersion {
            major: 0,
            minor: 5,
            patch: 0,
        };
        let core = SokrVersion {
            major: 0,
            minor: 1,
            patch: 0,
        };
        assert!(plugin.check_compatible(core).is_err());
    }

    #[test]
    fn version_compatible_older_plugin() {
        let plugin = SokrVersion {
            major: 0,
            minor: 0,
            patch: 5,
        };
        let core = SokrVersion {
            major: 0,
            minor: 1,
            patch: 0,
        };
        assert!(plugin.check_compatible(core).is_ok());
    }

    #[test]
    fn version_compatible_patch_ignored() {
        let plugin = SokrVersion {
            major: 0,
            minor: 1,
            patch: 99,
        };
        let core = SokrVersion {
            major: 0,
            minor: 1,
            patch: 0,
        };
        assert!(plugin.check_compatible(core).is_ok());
    }

    #[test]
    fn result_is_ok() {
        assert!(SokrResult::Ok.is_ok());
        assert!(!SokrResult::Ok.is_err());
    }

    #[test]
    fn result_is_err() {
        assert!(SokrResult::VersionMismatch.is_err());
        assert!(!SokrResult::VersionMismatch.is_ok());
    }

    #[test]
    fn computation_id_equality() {
        let id1 = SokrComputationId { high: 1, low: 2 };
        let id2 = SokrComputationId { high: 1, low: 2 };
        let id3 = SokrComputationId { high: 2, low: 1 };
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn current_version_matches_cargo() {
        // Prevent drift between SokrVersion::CURRENT and Cargo.toml version.
        let current = SokrVersion::CURRENT;
        let cargo = env!("CARGO_PKG_VERSION");
        let expected = format!("{}.{}.{}", current.major, current.minor, current.patch);
        assert_eq!(
            expected, cargo,
            "SokrVersion::CURRENT ({expected}) must match CARGO_PKG_VERSION ({cargo}). \
             Bump both together in Cargo.toml and types.rs."
        );
    }
}
