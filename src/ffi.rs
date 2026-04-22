//! C FFI exports for SOKR core.
//!
//! This module contains all `#[no_mangle] extern "C"` functions.
//! It is the only module that uses unsafe code.

#![allow(unsafe_code)]

use crate::types::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrDispatchRequest, SokrDispatchResponse, SokrResult, SokrVersion,
};

#[allow(missing_docs)]
static SOKR_VERSION_STATIC: SokrVersion = SokrVersion::CURRENT;

/// Returns the current SOKR core ABI version.
///
/// # Safety
/// The returned pointer is a static reference valid for the lifetime of the program.
/// Do not free or modify it.
#[no_mangle]
pub extern "C" fn sokr_version() -> *const SokrVersion {
    core::ptr::addr_of!(SOKR_VERSION_STATIC)
}

/// Checks if a plugin version is compatible with this core.
///
/// # Arguments
/// - `plugin`: Pointer to the plugin's version struct
/// - `result`: Pointer to store the result (`1` for compatible, `0` for not)
///
/// # Returns
/// - `SokrResult::Ok` on success
/// - `SokrResult::InvalidInput` if either pointer is null
///
/// # Safety
/// Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
#[no_mangle]
pub unsafe extern "C" fn sokr_check_version(
    plugin: *const SokrVersion,
    result: *mut i32,
) -> SokrResult {
    if plugin.is_null() || result.is_null() {
        return SokrResult::InvalidInput;
    }

    let plugin_version = unsafe { *plugin };
    let compatibility = plugin_version.check_compatible(SokrVersion::CURRENT);
    unsafe {
        *result = if compatibility.is_ok() { 1 } else { 0 };
    }

    compatibility
}

/// Queries a substrate's capability to fulfill a computation.
///
/// # Arguments
/// - `query`: Pointer to capability query descriptor
/// - `response`: Pointer to store the response
///
/// # Returns
/// - `SokrResult::Ok` on success
/// - `SokrResult::InvalidInput` if either pointer is null or IR data length is zero
///
/// # Safety
/// Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
#[no_mangle]
pub unsafe extern "C" fn sokr_capability(
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult {
    if query.is_null() || response.is_null() {
        return SokrResult::InvalidInput;
    }

    let query_ref = unsafe { &*query };

    // Validate IR data pointer is non-null if length is non-zero
    if query_ref.ir_data_len == 0 {
        return SokrResult::InvalidInput;
    }
    if query_ref.ir_format.is_null() || query_ref.ir_data_ptr.is_null() {
        return SokrResult::InvalidInput;
    }

    // TODO: Route to registered substrates (Phase 1.3)
    // For now, return NotFound since no substrates are registered yet
    let result = SokrResult::NoCapableSubstrate;
    unsafe {
        (*response).result = result;
    }
    result
}

/// Dispatches a computation to a substrate for execution.
///
/// # Arguments
/// - `request`: Pointer to dispatch request descriptor
/// - `response`: Pointer to store the response (contains completion token)
///
/// # Returns
/// - `SokrResult::Ok` on successful dispatch
/// - `SokrResult::InvalidInput` if either pointer is null or IR data length is zero
///
/// # Safety
/// Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
#[no_mangle]
pub unsafe extern "C" fn sokr_dispatch(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult {
    if request.is_null() || response.is_null() {
        return SokrResult::InvalidInput;
    }

    let request_ref = unsafe { &*request };

    // Validate IR data pointer is non-null if length is non-zero
    if request_ref.ir_data_len == 0 {
        return SokrResult::InvalidInput;
    }
    if request_ref.ir_data_ptr.is_null() {
        return SokrResult::InvalidInput;
    }

    // Validate params pointer is non-null if length is non-zero
    if request_ref.params_len > 0 && request_ref.params_ptr.is_null() {
        return SokrResult::InvalidInput;
    }

    // TODO: Route to registered substrates and dispatch (Phase 1.3)
    // For now, return NotFound since no substrates are registered yet
    let result = SokrResult::NoCapableSubstrate;
    unsafe {
        (*response).result = result;
        (*response).padding = 0;
        (*response).completion_token.handle = 0;
    }
    result
}

/// Queries the completion status of a dispatched computation.
///
/// # Arguments
/// - `query`: Pointer to completion query descriptor
/// - `signal`: Pointer to store the completion signal
///
/// # Returns
/// - `SokrResult::Ok` on success (when routed to a substrate)
/// - `SokrResult::NoCapableSubstrate` if no substrate is registered (current stub behavior)
/// - `SokrResult::InvalidInput` if either pointer is null or token handle is 0 (invalid)
///
/// # Timeout Behavior
/// - `timeout_ns = 0`: Non-blocking poll, returns immediately with current status
/// - `timeout_ns > 0`: Blocks up to specified nanoseconds (TODO: implement in Phase 1.3)
///
/// # Safety
/// Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
#[no_mangle]
pub unsafe extern "C" fn sokr_completion(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult {
    if query.is_null() || signal.is_null() {
        return SokrResult::InvalidInput;
    }

    let query_ref = unsafe { &*query };

    // Validate token handle is non-zero (invalid/unset sentinel)
    if query_ref.completion_token.handle == 0 {
        return SokrResult::InvalidInput;
    }

    // TODO: Route to registered substrates and check completion (Phase 1.3)
    // For now, return NotFound since no substrates are registered yet
    let result = SokrResult::NoCapableSubstrate;
    unsafe {
        (*signal) = SokrCompletionSignal::Pending;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sokr_version_returns_valid() {
        let ptr = sokr_version();
        assert!(!ptr.is_null());
        unsafe {
            let version = *ptr;
            assert_eq!(version.major, 0);
            assert_eq!(version.minor, 1);
        }
    }

    #[test]
    fn check_version_null_pointers() {
        let current = SokrVersion::CURRENT;
        let result = unsafe { sokr_check_version(core::ptr::null(), &mut 0) };
        assert_eq!(result, SokrResult::InvalidInput);

        let result = unsafe { sokr_check_version(&current, core::ptr::null_mut()) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn check_version_compatible() {
        let current = SokrVersion::CURRENT;
        let mut result = 0;
        let status = unsafe { sokr_check_version(&current, &mut result) };
        assert_eq!(status, SokrResult::Ok);
        assert_eq!(result, 1);
    }

    #[test]
    fn check_version_incompatible_returns_mismatch() {
        let newer = SokrVersion {
            major: 99,
            minor: 0,
            patch: 0,
        };
        let mut result = 0;
        let status = unsafe { sokr_check_version(&newer, &mut result) };
        assert_eq!(status, SokrResult::VersionMismatch);
        assert_eq!(result, 0);
    }

    #[test]
    fn capability_null_pointers_returns_invalid_input() {
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        // Null query pointer
        let result = unsafe { sokr_capability(core::ptr::null(), &mut response) };
        assert_eq!(result, SokrResult::InvalidInput);

        // Null response pointer
        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: [0].as_ptr().cast(),
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 1,
            padding: [0; 8],
        };
        let result = unsafe { sokr_capability(&query, core::ptr::null_mut()) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn capability_zero_length_ir_returns_invalid_input() {
        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: [0].as_ptr().cast(),
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 0,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::InvalidInput);
        // Early returns don't populate response - this is acceptable for InvalidInput
    }

    #[test]
    fn capability_populates_response_result() {
        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: [0].as_ptr().cast(),
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 1,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::NoCapableSubstrate);
        assert_eq!(response.result, SokrResult::NoCapableSubstrate);
    }

    #[test]
    fn capability_null_ir_format_returns_invalid_input() {
        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: core::ptr::null(),
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 1,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn capability_null_ir_data_returns_invalid_input() {
        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: [0].as_ptr().cast(),
            ir_data_ptr: core::ptr::null(),
            ir_data_len: 1,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn dispatch_null_pointers_returns_invalid_input() {
        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: crate::types::SokrCompletionToken { handle: 0 },
        };

        // Null request pointer
        let result = unsafe { sokr_dispatch(core::ptr::null(), &mut response) };
        assert_eq!(result, SokrResult::InvalidInput);

        // Null response pointer
        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: 0,
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 1,
            params_ptr: core::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };
        let result = unsafe { sokr_dispatch(&request, core::ptr::null_mut()) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn dispatch_zero_length_ir_returns_invalid_input() {
        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: 0,
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 0,
            params_ptr: core::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };
        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: crate::types::SokrCompletionToken { handle: 0 },
        };

        let result = unsafe { sokr_dispatch(&request, &mut response) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn dispatch_populates_response() {
        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: 0,
            ir_data_ptr: &0u8 as *const u8 as *const core::ffi::c_void,
            ir_data_len: 1,
            params_ptr: core::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };
        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            // Use sentinel to prove the function actually writes to handle
            completion_token: crate::types::SokrCompletionToken {
                handle: 0xDEAD_BEEF,
            },
        };

        let result = unsafe { sokr_dispatch(&request, &mut response) };
        assert_eq!(result, SokrResult::NoCapableSubstrate);
        assert_eq!(response.result, SokrResult::NoCapableSubstrate);
        // Function overwrites sentinel with 0 (invalid token sentinel)
        assert_eq!(response.completion_token.handle, 0);
    }

    #[test]
    fn completion_null_pointers_returns_invalid_input() {
        let mut signal = SokrCompletionSignal::Pending;

        // Null query pointer
        let result = unsafe { sokr_completion(core::ptr::null(), &mut signal) };
        assert_eq!(result, SokrResult::InvalidInput);

        // Null signal pointer
        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 1 },
            timeout_ns: 0,
            padding: [0; 8],
        };
        let result = unsafe { sokr_completion(&query, core::ptr::null_mut()) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn completion_invalid_token_handle_returns_invalid_input() {
        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 0 }, // Invalid
            timeout_ns: 0,
            padding: [0; 8],
        };
        let mut signal = SokrCompletionSignal::Pending;

        let result = unsafe { sokr_completion(&query, &mut signal) };
        assert_eq!(result, SokrResult::InvalidInput);
    }

    #[test]
    fn completion_populates_signal_pending() {
        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 1 },
            timeout_ns: 0,
            padding: [0; 8],
        };
        // Use sentinel to prove the function actually writes to signal
        let mut signal = SokrCompletionSignal::Complete;

        let result = unsafe { sokr_completion(&query, &mut signal) };
        assert_eq!(result, SokrResult::NoCapableSubstrate);
        assert_eq!(signal, SokrCompletionSignal::Pending);
    }

    #[test]
    fn completion_signal_variants_all_represented() {
        // Ensure all enum variants can be constructed and have correct discriminants
        assert_eq!(SokrCompletionSignal::Pending as u32, 0);
        assert_eq!(SokrCompletionSignal::Complete as u32, 1);
        assert_eq!(SokrCompletionSignal::Failed as u32, 2);
        assert_eq!(SokrCompletionSignal::TimedOut as u32, 3);
    }
}
