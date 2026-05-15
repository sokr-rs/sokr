//! C FFI exports for SOKR core.
//!
//! This module contains all `#[no_mangle] extern "C"` functions.
//! It is the only module that uses unsafe code.

#![allow(unsafe_code)]

use core::cell::UnsafeCell;

use crate::registry::Registry;
use crate::types::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrDispatchRequest, SokrDispatchResponse, SokrResult, SokrSubstratePlugin, SokrVersion,
};

/// Wrapper to allow `Sync` on `UnsafeCell<Registry>`.
///
/// # Safety
/// `Sync` is sound only because we guarantee single-threaded access
/// during Phase 1.2. Phase 1.3 will replace this with a proper mutex.
struct SyncRegistry(UnsafeCell<Registry>);

// SAFETY: Single-threaded access invariant for Phase 1.2.
// All FFI entry points document their `unsafe` contract.
unsafe impl Sync for SyncRegistry {}

impl SyncRegistry {
    const fn new() -> Self {
        Self(UnsafeCell::new(Registry::new()))
    }

    const fn get(&self) -> *mut Registry {
        self.0.get()
    }
}

static REGISTRY: SyncRegistry = SyncRegistry::new();

static SOKR_VERSION_STATIC: SokrVersion = SokrVersion::CURRENT;

/// Returns the current SOKR core ABI version.
///
/// # Returns
/// Pointer to a `static` `SokrVersion` valid for the lifetime of the program.
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
        *result = i32::from(compatibility.is_ok());
    }

    compatibility
}

/// Registers a substrate plugin with the core registry.
///
/// # Arguments
/// - `plugin`: Pointer to plugin vtable
/// - `substrate_id_out`: Output pointer for assigned non-zero substrate ID
///
/// # Returns
/// - `SokrResult::Ok` on success
/// - `SokrResult::InvalidInput` if pointers are null
/// - `SokrResult::VersionMismatch` if plugin ABI is incompatible
/// - `SokrResult::RegistryFull` if registry has no free slots
///
/// # Safety
/// Non-null pointers must be valid and properly aligned.
#[no_mangle]
pub unsafe extern "C" fn sokr_register_substrate(
    plugin: *const SokrSubstratePlugin,
    substrate_id_out: *mut u64,
) -> SokrResult {
    if plugin.is_null() || substrate_id_out.is_null() {
        return SokrResult::InvalidInput;
    }

    let plugin_value = unsafe { core::ptr::read(plugin) };
    let compatibility = plugin_value.version.check_compatible(SokrVersion::CURRENT);
    if compatibility != SokrResult::Ok {
        return SokrResult::VersionMismatch;
    }

    // SAFETY: Single-threaded access invariant for Phase 1.2.
    let registry = unsafe { &mut *REGISTRY.get() };
    match registry.register_with_id(plugin_value) {
        Ok(id) => {
            unsafe {
                *substrate_id_out = id;
            }
            SokrResult::Ok
        }
        Err(err) => err,
    }
}

/// Deregisters a substrate plugin by ID.
///
/// # Returns
/// - `SokrResult::Ok` if deregistered
/// - `SokrResult::InvalidInput` if `substrate_id` is 0
/// - `SokrResult::NotFound` if ID is unknown
#[no_mangle]
pub extern "C" fn sokr_deregister_substrate(substrate_id: u64) -> SokrResult {
    if substrate_id == 0 {
        return SokrResult::InvalidInput;
    }

    // SAFETY: Single-threaded access invariant for Phase 1.2.
    unsafe { (&mut *REGISTRY.get()).deregister(substrate_id) }
}

/// Lists currently registered substrate IDs.
///
/// # Arguments
/// - `substrate_ids_out`: Output buffer for substrate IDs (may be null only when `capacity == 0`)
/// - `capacity`: Number of `u64` entries available in `substrate_ids_out`
/// - `count_out`: Output pointer for total number of registered substrates
///
/// # Returns
/// - `SokrResult::Ok` if all IDs were written
/// - `SokrResult::RegistryFull` if `capacity` is smaller than the number of registered substrates
/// - `SokrResult::InvalidInput` if pointers are invalid
///
/// # Safety
/// - `count_out` must be non-null and valid for writes
/// - If `capacity > 0`, `substrate_ids_out` must be non-null and valid for `capacity` writes
#[no_mangle]
pub unsafe extern "C" fn sokr_list_substrates(
    substrate_ids_out: *mut u64,
    capacity: usize,
    count_out: *mut usize,
) -> SokrResult {
    if count_out.is_null() {
        return SokrResult::InvalidInput;
    }
    if capacity > 0 && substrate_ids_out.is_null() {
        return SokrResult::InvalidInput;
    }

    // SAFETY: Single-threaded access invariant for Phase 1.2.
    let registry = unsafe { &*REGISTRY.get() };
    let total = registry.len();
    unsafe {
        *count_out = total;
    }

    for (written, plugin) in registry.iter().enumerate() {
        if written >= capacity {
            return SokrResult::RegistryFull;
        }
        unsafe {
            *substrate_ids_out.add(written) = plugin.substrate_id;
        }
    }

    SokrResult::Ok
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
/// - `SokrResult::CapabilityDenied` if no registered substrate accepts the computation
///
/// # Response Contract (Failure Path)
/// On `InvalidInput` or `CapabilityDenied`, the response is fully zeroed
/// (`result`, `padding`, `substrate_id`, `estimated_latency_ns`).
/// Callers must not read uninitialized fields.
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

    // Route to registered substrates; first accepting plugin wins.
    // SAFETY: We hold exclusive access assumption (Phase 1.2, single-threaded).
    for plugin in unsafe { &*REGISTRY.get() }.iter() {
        let plugin_result =
            (plugin.capability_fn)(core::ptr::addr_of!(SOKR_VERSION_STATIC), query, response);
        match plugin_result {
            SokrResult::Ok => return SokrResult::Ok,
            // CapabilityDenied = plugin disclaims this computation; try next.
            SokrResult::CapabilityDenied => {}
            // Any other error means the plugin owns this request but failed.
            other => {
                unsafe {
                    (*response).result = other;
                    (*response).padding = 0;
                    (*response).substrate_id = 0;
                    (*response).estimated_latency_ns = 0;
                }
                return other;
            }
        }
        // Defensively zero between disclaim attempts so a later plugin cannot
        // observe or accidentally propagate a prior plugin's partial write.
        unsafe {
            (*response).substrate_id = 0;
            (*response).estimated_latency_ns = 0;
        }
    }

    let result = SokrResult::CapabilityDenied;
    unsafe {
        (*response).result = result;
        (*response).padding = 0;
        (*response).substrate_id = 0;
        (*response).estimated_latency_ns = 0;
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
/// - `SokrResult::NoCapableSubstrate` if no substrate can fulfill the computation
///
/// # Response Contract (Failure Path)
/// On any non-`Ok` result, the response is fully zeroed (`result`, `padding`,
/// `completion_token.handle = 0`). Callers must not read uninitialized fields.
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

    // Route to registered substrate by substrate_id.
    // SAFETY: Single-threaded access invariant for Phase 1.2.
    let registry = unsafe { &*REGISTRY.get() };
    if let Some(plugin) = registry.find_by_substrate_id(request_ref.substrate_id) {
        let dispatch_result = (plugin.dispatch_fn)(request, response);
        if dispatch_result == SokrResult::Ok {
            return SokrResult::Ok;
        }
        // Plugin owns this substrate_id but failed — propagate its exact error.
        let result = dispatch_result;
        unsafe {
            (*response).result = result;
            (*response).padding = 0;
            (*response).completion_token.handle = 0;
        }
        return result;
    }

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
/// - `SokrResult::Ok` on success (a substrate recognized the token)
/// - `SokrResult::NotFound` if no registered substrate recognizes the token
/// - `SokrResult::InvalidInput` if either pointer is null or token handle is 0 (invalid)
///
/// # Signal Contract (Failure Path)
/// On any non-`Ok` result where `signal` is non-null, `signal` is set to
/// `SokrCompletionSignal::Failed`. Exception: when `signal` itself is null,
/// no write occurs. Callers must not treat a failed query as `Pending`.
///
/// # Timeout Behavior
/// - `timeout_ns = 0`: Non-blocking poll, returns immediately with current status
/// - `timeout_ns > 0`: Blocks up to the specified nanoseconds per plugin attempt
///   (TODO: implement in Phase 1.3). With N registered plugins, the worst-case
///   wall time before returning is `N × timeout_ns`.
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
        unsafe {
            (*signal) = SokrCompletionSignal::Failed;
        }
        return SokrResult::InvalidInput;
    }

    // Route to registered substrates; first plugin recognizing the token wins.
    // SAFETY: Single-threaded access invariant for Phase 1.2.
    for plugin in unsafe { &*REGISTRY.get() }.iter() {
        let plugin_result = (plugin.completion_fn)(query, signal);
        match plugin_result {
            SokrResult::Ok => return SokrResult::Ok,
            // NotFound = plugin disclaims this token; try next.
            SokrResult::NotFound => {}
            // Any other error means the plugin owns this token but failed.
            other => {
                unsafe {
                    (*signal) = SokrCompletionSignal::Failed;
                }
                return other;
            }
        }
        // Defensively reset signal between disclaim attempts so a later plugin
        // cannot observe or accidentally propagate a prior plugin's partial write.
        unsafe {
            (*signal) = SokrCompletionSignal::Failed;
        }
    }

    let result = SokrResult::NotFound;
    unsafe {
        (*signal) = SokrCompletionSignal::Failed;
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
    fn capability_empty_registry_returns_capability_denied() {
        reset_registry();
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
        assert_eq!(result, SokrResult::CapabilityDenied);
        assert_eq!(response.result, SokrResult::CapabilityDenied);
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
    fn completion_empty_registry_returns_not_found() {
        reset_registry();
        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 1 },
            timeout_ns: 0,
            padding: [0; 8],
        };
        // Use sentinel to prove the function actually writes to signal
        let mut signal = SokrCompletionSignal::Complete;

        let result = unsafe { sokr_completion(&query, &mut signal) };
        assert_eq!(result, SokrResult::NotFound);
        // Failure path writes Failed, not Pending — Pending implies the
        // computation was accepted and is still running.
        assert_eq!(signal, SokrCompletionSignal::Failed);
    }

    #[test]
    fn completion_signal_variants_all_represented() {
        // Ensure all enum variants can be constructed and have correct discriminants
        assert_eq!(SokrCompletionSignal::Pending as u32, 0);
        assert_eq!(SokrCompletionSignal::Complete as u32, 1);
        assert_eq!(SokrCompletionSignal::Failed as u32, 2);
        assert_eq!(SokrCompletionSignal::TimedOut as u32, 3);
    }

    // ── Completion routing tests ─────────────────────────────

    extern "C" fn test_completion(
        query: *const SokrCompletionQuery,
        signal: *mut SokrCompletionSignal,
    ) -> SokrResult {
        unsafe {
            if (*query).completion_token.handle == 123 {
                (*signal) = SokrCompletionSignal::Complete;
                SokrResult::Ok
            } else {
                SokrResult::NotFound
            }
        }
    }

    fn completion_plugin(id: u64) -> crate::types::SokrSubstratePlugin {
        extern "C" fn dummy_capability(
            _version: *const SokrVersion,
            _query: *const SokrCapabilityQuery,
            _response: *mut SokrCapabilityResponse,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_dispatch(
            _request: *const SokrDispatchRequest,
            _response: *mut SokrDispatchResponse,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_destroy() {}

        crate::types::SokrSubstratePlugin {
            version: SokrVersion::CURRENT,
            capability_fn: dummy_capability,
            dispatch_fn: dummy_dispatch,
            completion_fn: test_completion,
            destroy_fn: dummy_destroy,
            substrate_id: id,
            padding: [0; 8],
        }
    }

    #[test]
    fn completion_routes_to_plugin_recognizing_token() {
        reset_registry();
        unsafe {
            (*REGISTRY.get()).register(completion_plugin(1));
        }

        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 123 },
            timeout_ns: 0,
            padding: [0; 8],
        };
        let mut signal = SokrCompletionSignal::Pending;

        let result = unsafe { sokr_completion(&query, &mut signal) };
        assert_eq!(result, SokrResult::Ok);
        assert_eq!(signal, SokrCompletionSignal::Complete);
    }

    #[test]
    fn completion_unknown_token_returns_not_found() {
        reset_registry();
        unsafe {
            (*REGISTRY.get()).register(completion_plugin(1));
        }

        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 999 },
            timeout_ns: 0,
            padding: [0; 8],
        };
        let mut signal = SokrCompletionSignal::Pending;

        let result = unsafe { sokr_completion(&query, &mut signal) };
        assert_eq!(result, SokrResult::NotFound);
        assert_eq!(signal, SokrCompletionSignal::Failed);
    }

    extern "C" fn disclaiming_completion(
        _query: *const SokrCompletionQuery,
        _signal: *mut SokrCompletionSignal,
    ) -> SokrResult {
        SokrResult::NotFound
    }

    fn disclaiming_completion_plugin(id: u64) -> crate::types::SokrSubstratePlugin {
        extern "C" fn dummy_capability(
            _version: *const SokrVersion,
            _query: *const SokrCapabilityQuery,
            _response: *mut SokrCapabilityResponse,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_dispatch(
            _request: *const SokrDispatchRequest,
            _response: *mut SokrDispatchResponse,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_destroy() {}

        crate::types::SokrSubstratePlugin {
            version: SokrVersion::CURRENT,
            capability_fn: dummy_capability,
            dispatch_fn: dummy_dispatch,
            completion_fn: disclaiming_completion,
            destroy_fn: dummy_destroy,
            substrate_id: id,
            padding: [0; 8],
        }
    }

    #[test]
    fn completion_routes_past_disclaiming_plugin() {
        reset_registry();
        unsafe {
            (*REGISTRY.get()).register(disclaiming_completion_plugin(1));
            (*REGISTRY.get()).register(completion_plugin(2));
        }

        let query = SokrCompletionQuery {
            completion_token: crate::types::SokrCompletionToken { handle: 123 },
            timeout_ns: 0,
            padding: [0; 8],
        };
        let mut signal = SokrCompletionSignal::Pending;

        let result = unsafe { sokr_completion(&query, &mut signal) };
        assert_eq!(result, SokrResult::Ok);
        assert_eq!(signal, SokrCompletionSignal::Complete);
    }

    #[test]
    fn dispatch_registered_plugin_error_propagates() {
        reset_registry();
        extern "C" fn failing_dispatch(
            _request: *const SokrDispatchRequest,
            _response: *mut SokrDispatchResponse,
        ) -> SokrResult {
            SokrResult::DispatchFailed
        }
        let assigned_id = unsafe {
            (*REGISTRY.get())
                .register_with_id(dispatch_plugin(7, failing_dispatch))
                .expect("registration should succeed")
        };

        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: assigned_id,
            ir_data_ptr: b"ir".as_ptr().cast(),
            ir_data_len: 2,
            params_ptr: core::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };
        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: crate::types::SokrCompletionToken { handle: 99 },
        };

        let result = unsafe { sokr_dispatch(&request, &mut response) };
        assert_eq!(result, SokrResult::DispatchFailed);
        assert_eq!(response.completion_token.handle, 0);
    }

    // ── Capability routing tests ───────────────────────────────

    fn reset_registry() {
        // SAFETY: Tests are run single-threaded (or hold implicit lock).
        unsafe { *REGISTRY.get() = Registry::new() };
    }

    extern "C" fn accepting_capability(
        _version: *const SokrVersion,
        _query: *const SokrCapabilityQuery,
        response: *mut SokrCapabilityResponse,
    ) -> SokrResult {
        unsafe {
            (*response).result = SokrResult::Ok;
            (*response).padding = 0;
            (*response).substrate_id = 42;
            (*response).estimated_latency_ns = 1_000;
        }
        SokrResult::Ok
    }

    extern "C" fn rejecting_capability(
        _version: *const SokrVersion,
        _query: *const SokrCapabilityQuery,
        _response: *mut SokrCapabilityResponse,
    ) -> SokrResult {
        SokrResult::CapabilityDenied
    }

    fn capability_plugin(
        cap_fn: crate::types::SokrCapabilityFn,
    ) -> crate::types::SokrSubstratePlugin {
        extern "C" fn dummy_dispatch(
            _request: *const SokrDispatchRequest,
            _response: *mut SokrDispatchResponse,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_completion(
            _query: *const SokrCompletionQuery,
            _signal: *mut SokrCompletionSignal,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_destroy() {}

        crate::types::SokrSubstratePlugin {
            version: SokrVersion::CURRENT,
            capability_fn: cap_fn,
            dispatch_fn: dummy_dispatch,
            completion_fn: dummy_completion,
            destroy_fn: dummy_destroy,
            substrate_id: 42,
            padding: [0; 8],
        }
    }

    #[test]
    fn capability_empty_registry_denies() {
        reset_registry();
        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: b"spirv\0".as_ptr().cast(),
            ir_data_ptr: b"data".as_ptr().cast(),
            ir_data_len: 4,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 99,
            estimated_latency_ns: 99,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::CapabilityDenied);
        assert_eq!(response.substrate_id, 0);
        assert_eq!(response.estimated_latency_ns, 0);
    }

    #[test]
    fn capability_routes_to_accepting_plugin() {
        reset_registry();
        unsafe {
            (*REGISTRY.get()).register(capability_plugin(accepting_capability));
        }

        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: b"spirv\0".as_ptr().cast(),
            ir_data_ptr: b"data".as_ptr().cast(),
            ir_data_len: 4,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::Ok);
        assert_eq!(response.substrate_id, 42);
        assert_eq!(response.estimated_latency_ns, 1_000);
    }

    #[test]
    fn capability_skips_rejecting_plugin() {
        reset_registry();
        unsafe {
            (*REGISTRY.get()).register(capability_plugin(rejecting_capability));
        }

        let query = SokrCapabilityQuery {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            ir_format: b"spirv\0".as_ptr().cast(),
            ir_data_ptr: b"data".as_ptr().cast(),
            ir_data_len: 4,
            padding: [0; 8],
        };
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 77,
            estimated_latency_ns: 77,
        };

        let result = unsafe { sokr_capability(&query, &mut response) };
        assert_eq!(result, SokrResult::CapabilityDenied);
        assert_eq!(response.substrate_id, 0);
        assert_eq!(response.estimated_latency_ns, 0);
    }

    // ── Dispatch routing tests ─────────────────────────────────

    extern "C" fn accepting_dispatch(
        _request: *const SokrDispatchRequest,
        response: *mut SokrDispatchResponse,
    ) -> SokrResult {
        unsafe {
            (*response).result = SokrResult::Ok;
            (*response).padding = 0;
            (*response).completion_token.handle = 123;
        }
        SokrResult::Ok
    }

    extern "C" fn rejecting_dispatch(
        _request: *const SokrDispatchRequest,
        _response: *mut SokrDispatchResponse,
    ) -> SokrResult {
        SokrResult::NoCapableSubstrate
    }

    fn dispatch_plugin(
        id: u64,
        dispatch_fn: crate::types::SokrDispatchFn,
    ) -> crate::types::SokrSubstratePlugin {
        extern "C" fn dummy_capability(
            _version: *const SokrVersion,
            _query: *const SokrCapabilityQuery,
            _response: *mut SokrCapabilityResponse,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_completion(
            _query: *const SokrCompletionQuery,
            _signal: *mut SokrCompletionSignal,
        ) -> SokrResult {
            SokrResult::Ok
        }
        extern "C" fn dummy_destroy() {}

        crate::types::SokrSubstratePlugin {
            version: SokrVersion::CURRENT,
            capability_fn: dummy_capability,
            dispatch_fn,
            completion_fn: dummy_completion,
            destroy_fn: dummy_destroy,
            substrate_id: id,
            padding: [0; 8],
        }
    }

    #[test]
    fn dispatch_routes_to_registered_plugin() {
        reset_registry();
        let assigned_id = unsafe {
            (*REGISTRY.get())
                .register_with_id(dispatch_plugin(7, accepting_dispatch))
                .expect("registration should succeed")
        };

        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: assigned_id,
            ir_data_ptr: b"ir".as_ptr().cast(),
            ir_data_len: 2,
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
        assert_eq!(result, SokrResult::Ok);
        assert_eq!(response.completion_token.handle, 123);
    }

    #[test]
    fn dispatch_unregistered_plugin_fails() {
        reset_registry();
        unsafe {
            (*REGISTRY.get()).register(dispatch_plugin(7, accepting_dispatch));
        }

        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: 99, // not registered
            ir_data_ptr: b"ir".as_ptr().cast(),
            ir_data_len: 2,
            params_ptr: core::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };
        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: crate::types::SokrCompletionToken { handle: 77 },
        };

        let result = unsafe { sokr_dispatch(&request, &mut response) };
        assert_eq!(result, SokrResult::NoCapableSubstrate);
        assert_eq!(response.completion_token.handle, 0);
    }

    #[test]
    fn dispatch_rejecting_plugin_returns_no_capable() {
        reset_registry();
        let assigned_id = unsafe {
            (*REGISTRY.get())
                .register_with_id(dispatch_plugin(7, rejecting_dispatch))
                .expect("registration should succeed")
        };

        let request = SokrDispatchRequest {
            computation_id: crate::types::SokrComputationId { high: 1, low: 2 },
            substrate_id: assigned_id,
            ir_data_ptr: b"ir".as_ptr().cast(),
            ir_data_len: 2,
            params_ptr: core::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };
        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: crate::types::SokrCompletionToken { handle: 88 },
        };

        let result = unsafe { sokr_dispatch(&request, &mut response) };
        assert_eq!(result, SokrResult::NoCapableSubstrate);
        assert_eq!(response.completion_token.handle, 0);
    }

    use core::sync::atomic::{AtomicUsize, Ordering};

    static FFI_DESTROY_CALLS: AtomicUsize = AtomicUsize::new(0);

    extern "C" fn ffi_test_capability(
        _version: *const SokrVersion,
        _query: *const SokrCapabilityQuery,
        _response: *mut SokrCapabilityResponse,
    ) -> SokrResult {
        SokrResult::CapabilityDenied
    }

    extern "C" fn ffi_test_dispatch(
        _request: *const SokrDispatchRequest,
        _response: *mut SokrDispatchResponse,
    ) -> SokrResult {
        SokrResult::NoCapableSubstrate
    }

    extern "C" fn ffi_test_completion(
        _query: *const SokrCompletionQuery,
        _signal: *mut SokrCompletionSignal,
    ) -> SokrResult {
        SokrResult::NotFound
    }

    extern "C" fn ffi_test_destroy() {
        FFI_DESTROY_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn ffi_test_plugin(version: SokrVersion) -> crate::types::SokrSubstratePlugin {
        crate::types::SokrSubstratePlugin {
            version,
            capability_fn: ffi_test_capability,
            dispatch_fn: ffi_test_dispatch,
            completion_fn: ffi_test_completion,
            destroy_fn: ffi_test_destroy,
            substrate_id: 0,
            padding: [0; 8],
        }
    }

    #[test]
    fn register_one_plugin_succeeds_and_returns_id() {
        reset_registry();
        let plugin = ffi_test_plugin(SokrVersion::CURRENT);
        let mut assigned = 0;

        let result = unsafe { sokr_register_substrate(&plugin, &mut assigned) };
        assert_eq!(result, SokrResult::Ok);
        assert_ne!(assigned, 0);
    }

    #[test]
    fn register_beyond_capacity_returns_registry_full() {
        reset_registry();

        for _ in 0..crate::registry::MAX_SUBSTRATES {
            let plugin = ffi_test_plugin(SokrVersion::CURRENT);
            let mut assigned = 0;
            assert_eq!(
                unsafe { sokr_register_substrate(&plugin, &mut assigned) },
                SokrResult::Ok
            );
            assert_ne!(assigned, 0);
        }

        let plugin = ffi_test_plugin(SokrVersion::CURRENT);
        let mut assigned = 0;
        assert_eq!(
            unsafe { sokr_register_substrate(&plugin, &mut assigned) },
            SokrResult::RegistryFull
        );
    }

    #[test]
    fn register_incompatible_version_returns_version_mismatch() {
        reset_registry();
        let plugin = ffi_test_plugin(SokrVersion {
            major: 99,
            minor: 0,
            patch: 0,
        });
        let mut assigned = 0;

        let result = unsafe { sokr_register_substrate(&plugin, &mut assigned) };
        assert_eq!(result, SokrResult::VersionMismatch);
        assert_eq!(assigned, 0);
    }

    #[test]
    fn register_null_pointer_returns_invalid_input() {
        reset_registry();
        let plugin = ffi_test_plugin(SokrVersion::CURRENT);
        let mut assigned = 0;

        assert_eq!(
            unsafe { sokr_register_substrate(core::ptr::null(), &mut assigned) },
            SokrResult::InvalidInput
        );
        assert_eq!(
            unsafe { sokr_register_substrate(&plugin, core::ptr::null_mut()) },
            SokrResult::InvalidInput
        );
    }

    #[test]
    fn deregister_existing_plugin_calls_destroy_once() {
        reset_registry();
        FFI_DESTROY_CALLS.store(0, Ordering::SeqCst);

        let plugin = ffi_test_plugin(SokrVersion::CURRENT);
        let mut assigned = 0;
        assert_eq!(
            unsafe { sokr_register_substrate(&plugin, &mut assigned) },
            SokrResult::Ok
        );

        assert_eq!(sokr_deregister_substrate(assigned), SokrResult::Ok);
        assert_eq!(FFI_DESTROY_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn deregister_unknown_returns_not_found() {
        reset_registry();
        assert_eq!(sokr_deregister_substrate(777), SokrResult::NotFound);
    }

    #[test]
    fn deregister_then_reregister_works() {
        reset_registry();

        let plugin_a = ffi_test_plugin(SokrVersion::CURRENT);
        let mut id_a = 0;
        assert_eq!(
            unsafe { sokr_register_substrate(&plugin_a, &mut id_a) },
            SokrResult::Ok
        );
        assert_eq!(sokr_deregister_substrate(id_a), SokrResult::Ok);

        let plugin_b = ffi_test_plugin(SokrVersion::CURRENT);
        let mut id_b = 0;
        assert_eq!(
            unsafe { sokr_register_substrate(&plugin_b, &mut id_b) },
            SokrResult::Ok
        );
        assert_ne!(id_b, 0);
    }

    #[test]
    fn list_substrates_returns_registered_ids() {
        reset_registry();

        let plugin_a = ffi_test_plugin(SokrVersion::CURRENT);
        let plugin_b = ffi_test_plugin(SokrVersion::CURRENT);
        let mut id_a = 0;
        let mut id_b = 0;

        assert_eq!(
            unsafe { sokr_register_substrate(&plugin_a, &mut id_a) },
            SokrResult::Ok
        );
        assert_eq!(
            unsafe { sokr_register_substrate(&plugin_b, &mut id_b) },
            SokrResult::Ok
        );

        let mut ids = [0u64; 2];
        let mut count = 0usize;
        let result = unsafe { sokr_list_substrates(ids.as_mut_ptr(), ids.len(), &mut count) };

        assert_eq!(result, SokrResult::Ok);
        assert_eq!(count, 2);
        assert!(ids.contains(&id_a));
        assert!(ids.contains(&id_b));
    }
}
