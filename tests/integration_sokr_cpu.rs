#![allow(unsafe_code)]

//! Integration test: sokr core with sokr-cpu reference substrate.
//!
//! Tests the full round-trip:
//! - Register → Capability → Dispatch → Completion

use sokr::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrComputationId, SokrDispatchRequest, SokrDispatchResponse, SokrResult,
};

use sokr_cpu::CPU_PLUGIN;

// Macro to assert with std::process::exit fallback (doesn't require unwinding)
macro_rules! assert_eq_exit {
    ($left:expr, $right:expr, $msg:expr) => {
        if $left != $right {
            eprintln!("Assertion failed: {}", $msg);
            eprintln!("  left: {:?}", $left);
            eprintln!("  right: {:?}", $right);
            std::process::exit(1);
        }
    };
}

macro_rules! assert_ne_exit {
    ($left:expr, $right:expr, $msg:expr) => {
        if $left == $right {
            eprintln!("Assertion failed: {}", $msg);
            eprintln!("  value: {:?}", $left);
            std::process::exit(1);
        }
    };
}

#[test]
fn test_register_cpu_plugin() {
    unsafe {
        let mut substrate_id: u64 = 0;
        let result =
            sokr::ffi::sokr_register_substrate(std::ptr::addr_of!(CPU_PLUGIN), &mut substrate_id);

        assert_eq_exit!(result, SokrResult::Ok, "Failed to register CPU plugin");
        assert_ne_exit!(substrate_id, 0, "Assigned substrate ID should be non-zero");

        sokr::ffi::sokr_deregister_substrate(substrate_id);
    }
}

#[test]
fn test_capability_query_with_cpu() {
    unsafe {
        let mut substrate_id: u64 = 0;
        let register_result =
            sokr::ffi::sokr_register_substrate(std::ptr::addr_of!(CPU_PLUGIN), &mut substrate_id);
        assert_eq_exit!(
            register_result,
            SokrResult::Ok,
            "Failed to register CPU plugin"
        );

        let query = SokrCapabilityQuery {
            computation_id: SokrComputationId { high: 1, low: 2 },
            ir_format: b"test\0".as_ptr().cast(),
            ir_data_ptr: b"dummy_ir".as_ptr().cast(),
            ir_data_len: 8,
            padding: [0; 8],
        };

        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = sokr::ffi::sokr_capability(&query, &mut response);
        assert_eq_exit!(result, SokrResult::Ok, "Capability query should succeed");
        assert_eq_exit!(
            response.result,
            SokrResult::Ok,
            "Capability response result should be Ok"
        );
        assert_ne_exit!(
            response.substrate_id,
            0,
            "CPU should accept the computation"
        );

        sokr::ffi::sokr_deregister_substrate(substrate_id);
    }
}

#[test]
fn test_dispatch_with_cpu() {
    unsafe {
        let mut substrate_id: u64 = 0;
        let register_result =
            sokr::ffi::sokr_register_substrate(std::ptr::addr_of!(CPU_PLUGIN), &mut substrate_id);
        assert_eq_exit!(
            register_result,
            SokrResult::Ok,
            "Failed to register CPU plugin"
        );

        let request = SokrDispatchRequest {
            computation_id: SokrComputationId { high: 3, low: 4 },
            substrate_id,
            ir_data_ptr: b"test_ir".as_ptr().cast(),
            ir_data_len: 7,
            params_ptr: std::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };

        let mut response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: sokr::SokrCompletionToken { handle: 0 },
        };

        let result = sokr::ffi::sokr_dispatch(&request, &mut response);
        assert_eq_exit!(result, SokrResult::Ok, "Dispatch should succeed");
        assert_eq_exit!(
            response.result,
            SokrResult::Ok,
            "Dispatch response result should be Ok"
        );
        assert_ne_exit!(
            response.completion_token.handle,
            0,
            "Completion token handle should be non-zero"
        );

        sokr::ffi::sokr_deregister_substrate(substrate_id);
    }
}

#[test]
fn test_completion_query_with_cpu() {
    unsafe {
        let mut substrate_id: u64 = 0;
        let register_result =
            sokr::ffi::sokr_register_substrate(std::ptr::addr_of!(CPU_PLUGIN), &mut substrate_id);
        assert_eq_exit!(
            register_result,
            SokrResult::Ok,
            "Failed to register CPU plugin"
        );

        let dispatch_request = SokrDispatchRequest {
            computation_id: SokrComputationId { high: 5, low: 6 },
            substrate_id,
            ir_data_ptr: b"dispatch_ir".as_ptr().cast(),
            ir_data_len: 11,
            params_ptr: std::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };

        let mut dispatch_response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: sokr::SokrCompletionToken { handle: 0 },
        };

        let dispatch_result = sokr::ffi::sokr_dispatch(&dispatch_request, &mut dispatch_response);
        assert_eq_exit!(dispatch_result, SokrResult::Ok, "Failed to dispatch");

        let token = dispatch_response.completion_token;
        assert_ne_exit!(token.handle, 0, "Token should be valid");

        let completion_query = SokrCompletionQuery {
            completion_token: token,
            timeout_ns: 0,
            padding: [0; 8],
        };

        let mut signal = SokrCompletionSignal::Pending;
        let result = sokr::ffi::sokr_completion(&completion_query, &mut signal);

        assert_eq_exit!(result, SokrResult::Ok, "Completion query should succeed");
        assert_eq_exit!(
            signal,
            SokrCompletionSignal::Complete,
            "CPU should complete immediately"
        );

        sokr::ffi::sokr_deregister_substrate(substrate_id);
    }
}

#[test]
fn test_full_roundtrip_register_capability_dispatch_completion() {
    unsafe {
        let mut substrate_id: u64 = 0;
        let register_result =
            sokr::ffi::sokr_register_substrate(std::ptr::addr_of!(CPU_PLUGIN), &mut substrate_id);
        assert_eq_exit!(
            register_result,
            SokrResult::Ok,
            "Failed to register CPU plugin"
        );
        assert_ne_exit!(substrate_id, 0, "substrate_id should be non-zero");

        let capability_query = SokrCapabilityQuery {
            computation_id: SokrComputationId { high: 7, low: 8 },
            ir_format: b"roundtrip\0".as_ptr().cast(),
            ir_data_ptr: b"capability_ir".as_ptr().cast(),
            ir_data_len: 13,
            padding: [0; 8],
        };

        let mut capability_response = SokrCapabilityResponse {
            result: SokrResult::Ok,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let cap_result = sokr::ffi::sokr_capability(&capability_query, &mut capability_response);
        assert_eq_exit!(
            cap_result,
            SokrResult::Ok,
            "Capability query should succeed"
        );
        assert_eq_exit!(
            capability_response.result,
            SokrResult::Ok,
            "Capability response result should be Ok"
        );

        let dispatch_request = SokrDispatchRequest {
            computation_id: capability_query.computation_id,
            substrate_id,
            ir_data_ptr: capability_query.ir_data_ptr,
            ir_data_len: capability_query.ir_data_len,
            params_ptr: std::ptr::null(),
            params_len: 0,
            padding: [0; 16],
        };

        let mut dispatch_response = SokrDispatchResponse {
            result: SokrResult::Ok,
            padding: 0,
            completion_token: sokr::SokrCompletionToken { handle: 0 },
        };

        let disp_result = sokr::ffi::sokr_dispatch(&dispatch_request, &mut dispatch_response);
        assert_eq_exit!(disp_result, SokrResult::Ok, "Dispatch should succeed");
        assert_eq_exit!(
            dispatch_response.result,
            SokrResult::Ok,
            "Dispatch response result should be Ok"
        );
        assert_ne_exit!(
            dispatch_response.completion_token.handle,
            0,
            "completion_token should be non-zero"
        );

        let completion_query = SokrCompletionQuery {
            completion_token: dispatch_response.completion_token,
            timeout_ns: 0,
            padding: [0; 8],
        };

        let mut signal = SokrCompletionSignal::Pending;
        let comp_result = sokr::ffi::sokr_completion(&completion_query, &mut signal);
        assert_eq_exit!(
            comp_result,
            SokrResult::Ok,
            "Completion query should succeed"
        );
        assert_eq_exit!(
            signal,
            SokrCompletionSignal::Complete,
            "Signal should be Complete"
        );

        sokr::ffi::sokr_deregister_substrate(substrate_id);
    }
}

#[test]
fn test_cpu_plugin_metadata() {
    assert_eq_exit!(
        CPU_PLUGIN.version,
        sokr::SokrVersion::CURRENT,
        "CPU plugin should have current ABI version"
    );
    let cap_fn = CPU_PLUGIN.capability_fn as *const core::ffi::c_void;
    assert_ne_exit!(cap_fn, std::ptr::null(), "capability_fn should not be null");
    let dis_fn = CPU_PLUGIN.dispatch_fn as *const core::ffi::c_void;
    assert_ne_exit!(dis_fn, std::ptr::null(), "dispatch_fn should not be null");
    let com_fn = CPU_PLUGIN.completion_fn as *const core::ffi::c_void;
    assert_ne_exit!(com_fn, std::ptr::null(), "completion_fn should not be null");
    let des_fn = CPU_PLUGIN.destroy_fn as *const core::ffi::c_void;
    assert_ne_exit!(des_fn, std::ptr::null(), "destroy_fn should not be null");
}
