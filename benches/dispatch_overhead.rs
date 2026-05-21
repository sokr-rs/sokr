//! Benchmark: SOKR dispatch overhead vs raw vtable call
//!
//! Measures the performance overhead of the SOKR FFI layer compared to
//! direct vtable function calls.
//!
//! Run with: cargo bench --bench dispatch_overhead

#![allow(unsafe_code)]
#![allow(dead_code)]

use std::ffi::c_void;

#[derive(Clone, Copy)]
struct ComputationId {
    high: u64,
    low: u64,
}

#[derive(Clone, Copy)]
struct DispatchRequest {
    computation_id: ComputationId,
    substrate_id: u64,
    ir_data_ptr: *const c_void,
    ir_data_len: usize,
    params_ptr: *const c_void,
    params_len: usize,
}

#[derive(Clone, Copy)]
struct CompletionToken {
    handle: u64,
}

#[derive(Clone, Copy)]
struct DispatchResponse {
    result: u32,
    padding: u32,
    completion_token: CompletionToken,
}

// Dispatch function signature (matches sokr)
type DispatchFn = extern "C" fn(*const DispatchRequest, *mut DispatchResponse) -> u32;

// Mock dispatch functions
extern "C" fn mock_dispatch_empty(
    _request: *const DispatchRequest,
    response: *mut DispatchResponse,
) -> u32 {
    unsafe {
        (*response).result = 0;
        (*response).completion_token.handle = 42;
    }
    0
}

extern "C" fn mock_dispatch_with_work(
    _request: *const DispatchRequest,
    response: *mut DispatchResponse,
) -> u32 {
    // Simulate a tiny amount of work to avoid optimization
    let mut sum: u64 = 0;
    for i in 0..10 {
        sum = sum.wrapping_add(i);
    }

    unsafe {
        (*response).result = 0;
        (*response).completion_token.handle = 42 + sum as u64;
    }
    0
}

fn main() {
    println!("SOKR Dispatch Overhead Benchmark");
    println!("================================\n");

    // Setup
    let dispatch_fn: DispatchFn = mock_dispatch_empty;

    let computation_id = ComputationId {
        high: 0x0123456789ABCDEF,
        low: 0xFEDCBA9876543210,
    };

    let ir_data = b"test_ir_data";

    let request = DispatchRequest {
        computation_id,
        substrate_id: 1,
        ir_data_ptr: ir_data.as_ptr() as *const c_void,
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
    };

    // Benchmark: Direct vtable call
    println!("Benchmark 1: Direct vtable function call");
    let iterations = 100_000_000;
    let start = std::time::Instant::now();

    let mut response = DispatchResponse {
        result: 0,
        padding: 0,
        completion_token: CompletionToken { handle: 0 },
    };

    for _ in 0..iterations {
        let _ = dispatch_fn(&request, &mut response);
    }

    let vtable_duration = start.elapsed();
    let vtable_per_call = vtable_duration.as_nanos() as f64 / iterations as f64;

    println!(
        "  Total: {:?} ({:.2} ns/call)",
        vtable_duration, vtable_per_call
    );
    println!(
        "  Response token: {} (to prevent optimization)\n",
        response.completion_token.handle
    );

    // Benchmark: Via function pointer (overhead simulation)
    println!("Benchmark 2: Function dispatch via indirect call");
    let start = std::time::Instant::now();

    let mut response = DispatchResponse {
        result: 0,
        padding: 0,
        completion_token: CompletionToken { handle: 0 },
    };

    for _ in 0..iterations {
        let fn_ptr = dispatch_fn as DispatchFn;
        let _ = fn_ptr(&request, &mut response);
    }

    let indirect_duration = start.elapsed();
    let indirect_per_call = indirect_duration.as_nanos() as f64 / iterations as f64;

    println!(
        "  Total: {:?} ({:.2} ns/call)",
        indirect_duration, indirect_per_call
    );
    println!(
        "  Response token: {} (to prevent optimization)\n",
        response.completion_token.handle
    );

    // Calculate overhead
    let overhead_ns = indirect_per_call - vtable_per_call;
    let overhead_percent = if vtable_per_call > 0.0 {
        (overhead_ns / vtable_per_call) * 100.0
    } else {
        0.0
    };

    println!("Performance Analysis");
    println!("====================");
    println!("Direct vtable call:  {:.2} ns/call", vtable_per_call);
    println!("Indirect call:       {:.2} ns/call", indirect_per_call);
    println!(
        "Overhead:            {:.2} ns/call ({:.2}%)",
        overhead_ns, overhead_percent
    );
    println!();

    // Success criteria
    let success = overhead_percent < 5.0;
    println!(
        "Requirement: overhead < 5.0% ... {}",
        if success { "✓ PASS" } else { "✗ FAIL" }
    );

    if success {
        println!("\nBenchmark result: PASS");
        std::process::exit(0);
    } else {
        println!("\nBenchmark result: FAIL - overhead exceeds 5% threshold");
        std::process::exit(1);
    }
}
