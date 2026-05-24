//! Benchmark: SOKR dispatch overhead vs raw vtable call
//!
//! Measures the actual cost of routing a dispatch through `sokr_dispatch`
//! (null-pointer validation + registry lookup + plugin fn call) compared
//! to calling the substrate's `dispatch_fn` directly. The delta is the
//! tax the core ABI imposes on every dispatch.
//!
//! Run with: cargo bench --bench dispatch_overhead

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::hint::black_box;
use std::ptr;
use std::time::Instant;

use sokr::ffi::{sokr_dispatch, sokr_register_substrate};
use sokr::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrCompletionToken, SokrComputationId, SokrDispatchRequest, SokrDispatchResponse, SokrResult,
    SokrSubstratePlugin, SokrVersion,
};

extern "C" fn bench_capability(
    _v: *const SokrVersion,
    _q: *const SokrCapabilityQuery,
    _r: *mut SokrCapabilityResponse,
) -> SokrResult {
    SokrResult::Ok
}

extern "C" fn bench_dispatch(
    _request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult {
    unsafe {
        (*response).result = SokrResult::Ok;
        (*response).padding = 0;
        (*response).completion_token.handle = 42;
    }
    SokrResult::Ok
}

extern "C" fn bench_completion(
    _q: *const SokrCompletionQuery,
    _s: *mut SokrCompletionSignal,
) -> SokrResult {
    SokrResult::Ok
}

extern "C" fn bench_destroy() {}

fn main() {
    println!("SOKR Dispatch Overhead Benchmark");
    println!("================================\n");

    let plugin = SokrSubstratePlugin {
        version: SokrVersion::CURRENT,
        capability_fn: bench_capability,
        dispatch_fn: bench_dispatch,
        completion_fn: bench_completion,
        destroy_fn: bench_destroy,
        substrate_id: 0,
        padding: [0; 8],
    };
    let mut substrate_id: u64 = 0;
    let reg_result = unsafe { sokr_register_substrate(&plugin, &mut substrate_id) };
    assert_eq!(
        reg_result,
        SokrResult::Ok,
        "Failed to register benchmark plugin"
    );

    let ir_data = b"test_ir_data";
    let request = SokrDispatchRequest {
        computation_id: SokrComputationId {
            high: 0x0123_4567_89AB_CDEF,
            low: 0xFEDC_BA98_7654_3210,
        },
        substrate_id,
        ir_data_ptr: ir_data.as_ptr().cast::<c_void>(),
        ir_data_len: ir_data.len(),
        params_ptr: ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };

    let iterations: u64 = 100_000_000;

    println!("Benchmark 1: Raw vtable call (bench_dispatch directly)");
    let mut response = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };
    let start = Instant::now();
    for _ in 0..iterations {
        let r = bench_dispatch(black_box(&request), black_box(&mut response));
        black_box(r);
        black_box(&response);
    }
    let raw_duration = start.elapsed();
    let raw_per_call = raw_duration.as_nanos() as f64 / iterations as f64;
    println!("  Total: {raw_duration:?} ({raw_per_call:.2} ns/call)");
    println!(
        "  Response token: {} (to prevent optimization)\n",
        response.completion_token.handle
    );

    println!("Benchmark 2: SOKR dispatch path (sokr_dispatch)");
    let mut response = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };
    let start = Instant::now();
    for _ in 0..iterations {
        let r = unsafe { sokr_dispatch(black_box(&request), black_box(&mut response)) };
        black_box(r);
        black_box(&response);
    }
    let sokr_duration = start.elapsed();
    let sokr_per_call = sokr_duration.as_nanos() as f64 / iterations as f64;
    println!("  Total: {sokr_duration:?} ({sokr_per_call:.2} ns/call)");
    println!(
        "  Response token: {} (to prevent optimization)\n",
        response.completion_token.handle
    );

    let overhead_ns = sokr_per_call - raw_per_call;
    let overhead_ratio = if raw_per_call > 0.0 {
        overhead_ns / raw_per_call
    } else {
        0.0
    };
    let overhead_percent = overhead_ratio * 100.0;

    println!("Performance Analysis");
    println!("====================");
    println!("Raw vtable call:     {raw_per_call:.2} ns/call");
    println!("sokr_dispatch:       {sokr_per_call:.2} ns/call");
    println!("Overhead:            {overhead_ns:.2} ns/call ({overhead_percent:.2}%)");
    println!();

    // Use absolute-nanosecond threshold rather than a percentage: the baseline
    // is an empty function call, so any percentage figure mostly reflects how
    // little the baseline does. SOKR's FFI tax in absolute time is what
    // actually matters when composed with real substrate workloads
    // (microseconds for CPU, milliseconds+ for GPU/QPU).
    const OVERHEAD_BUDGET_NS: f64 = 50.0;
    let success = overhead_ns < OVERHEAD_BUDGET_NS;
    println!(
        "Requirement: overhead < {OVERHEAD_BUDGET_NS:.1} ns/call ... {}",
        if success { "PASS" } else { "FAIL" }
    );

    if success {
        println!("\nBenchmark result: PASS");
        std::process::exit(0);
    } else {
        println!(
            "\nBenchmark result: FAIL - overhead {overhead_ns:.2} ns exceeds {OVERHEAD_BUDGET_NS:.1} ns budget"
        );
        std::process::exit(1);
    }
}
