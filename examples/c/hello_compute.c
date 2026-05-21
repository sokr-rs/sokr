/**
 * SOKR C API Example: Hello Compute
 *
 * Demonstrates the complete Register → Capability → Dispatch → Completion
 * workflow using the SOKR C FFI interface.
 *
 * This example:
 * 1. Registers a substrate plugin (sokr-cpu reference implementation)
 * 2. Queries the substrate's capability to accept a computation
 * 3. Dispatches a computation for execution
 * 4. Checks the completion status
 *
 * Build:
 *   gcc -o hello_compute hello_compute.c -I../../include -L../../target/debug \
 *       -lsokr_cpu -lsokr
 */

#include <sokr.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Declare the CPU plugin from sokr-cpu library */
extern const struct SokrSubstratePlugin CPU_PLUGIN;

/* Helper to print SokrResult status */
static const char *sokr_result_str(enum SokrResult result) {
  switch (result) {
  case SOKR_RESULT_OK:
    return "Ok";
  case SOKR_RESULT_CAPABILITY_DENIED:
    return "CapabilityDenied";
  case SOKR_RESULT_DISPATCH_FAILED:
    return "DispatchFailed";
  case SOKR_RESULT_TIMEOUT:
    return "Timeout";
  case SOKR_RESULT_VERSION_MISMATCH:
    return "VersionMismatch";
  case SOKR_RESULT_NO_CAPABLE_SUBSTRATE:
    return "NoCapableSubstrate";
  case SOKR_RESULT_INVALID_INPUT:
    return "InvalidInput";
  case SOKR_RESULT_INVALID_IR:
    return "InvalidIR";
  case SOKR_RESULT_NOT_FOUND:
    return "NotFound";
  case SOKR_RESULT_REGISTRY_FULL:
    return "RegistryFull";
  }
  return "Unknown";
}

/* Helper to print completion signal */
static const char *sokr_signal_str(enum SokrCompletionSignal signal) {
  switch (signal) {
  case SOKR_COMPLETION_SIGNAL_PENDING:
    return "Pending";
  case SOKR_COMPLETION_SIGNAL_COMPLETE:
    return "Complete";
  case SOKR_COMPLETION_SIGNAL_FAILED:
    return "Failed";
  case SOKR_COMPLETION_SIGNAL_TIMED_OUT:
    return "TimedOut";
  }
  return "Unknown";
}

int main(void) {
  printf("SOKR C API Example: Hello Compute\n");
  printf("==================================\n\n");

  /* ──────────────────────────────────────────────────────────────
   * Step 1: Register the substrate plugin
   * ────────────────────────────────────────────────────────────── */

  printf("Step 1: Register substrate plugin\n");

  uint64_t substrate_id = 0;
  enum SokrResult result = sokr_register_substrate(&CPU_PLUGIN, &substrate_id);

  if (result != SOKR_RESULT_OK) {
    fprintf(stderr, "Failed to register substrate: %s\n", sokr_result_str(result));
    return 1;
  }

  printf("  ✓ Registered CPU substrate with ID: %lu\n\n", substrate_id);

  /* ──────────────────────────────────────────────────────────────
   * Step 2: Query substrate capability
   * ────────────────────────────────────────────────────────────── */

  printf("Step 2: Query substrate capability\n");

  struct SokrComputationId comp_id = {
      .high = 0x0123456789ABCDEF,
      .low = 0xFEDCBA9876543210,
  };

  const char ir_format[] = "example";
  const char ir_data[] = "compute_payload";

  struct SokrCapabilityQuery capability_query = {
      .computation_id = comp_id,
      .ir_format = ir_format,
      .ir_data_ptr = (const void *)ir_data,
      .ir_data_len = sizeof(ir_data) - 1,
      .padding = {0},
  };

  struct SokrCapabilityResponse capability_response = {0};

  result = sokr_capability(&capability_query, &capability_response);

  if (result != SOKR_RESULT_OK) {
    fprintf(stderr, "Capability query failed: %s\n", sokr_result_str(result));
    sokr_deregister_substrate(substrate_id);
    return 1;
  }

  if (capability_response.result != SOKR_RESULT_OK) {
    fprintf(stderr, "Substrate denied capability: %s\n",
            sokr_result_str(capability_response.result));
    sokr_deregister_substrate(substrate_id);
    return 1;
  }

  printf("  ✓ Capability accepted by substrate %lu\n", capability_response.substrate_id);
  printf("    Estimated latency: %lu ns\n\n", capability_response.estimated_latency_ns);

  /* ──────────────────────────────────────────────────────────────
   * Step 3: Dispatch computation for execution
   * ────────────────────────────────────────────────────────────── */

  printf("Step 3: Dispatch computation\n");

  struct SokrDispatchRequest dispatch_request = {
      .computation_id = comp_id,
      .substrate_id = substrate_id,
      .ir_data_ptr = (const void *)ir_data,
      .ir_data_len = sizeof(ir_data) - 1,
      .params_ptr = NULL,
      .params_len = 0,
      .padding = {0},
  };

  struct SokrDispatchResponse dispatch_response = {0};

  result = sokr_dispatch(&dispatch_request, &dispatch_response);

  if (result != SOKR_RESULT_OK) {
    fprintf(stderr, "Dispatch failed: %s\n", sokr_result_str(result));
    sokr_deregister_substrate(substrate_id);
    return 1;
  }

  uint64_t completion_token = dispatch_response.completion_token;

  if (completion_token == 0) {
    fprintf(stderr, "Dispatch returned invalid completion token\n");
    sokr_deregister_substrate(substrate_id);
    return 1;
  }

  printf("  ✓ Computation dispatched\n");
  printf("    Completion token: 0x%016lx\n\n", completion_token);

  /* ──────────────────────────────────────────────────────────────
   * Step 4: Query completion status
   * ────────────────────────────────────────────────────────────── */

  printf("Step 4: Query completion status\n");

  struct SokrCompletionQuery completion_query = {
      .completion_token = completion_token,
      .timeout_ns = 0, /* Non-blocking poll */
      .padding = {0},
  };

  enum SokrCompletionSignal completion_signal = SOKR_COMPLETION_SIGNAL_PENDING;

  result = sokr_completion(&completion_query, &completion_signal);

  if (result != SOKR_RESULT_OK) {
    fprintf(stderr, "Completion query failed: %s\n", sokr_result_str(result));
    sokr_deregister_substrate(substrate_id);
    return 1;
  }

  printf("  ✓ Completion signal: %s\n\n", sokr_signal_str(completion_signal));

  /* ──────────────────────────────────────────────────────────────
   * Cleanup: Deregister substrate
   * ────────────────────────────────────────────────────────────── */

  printf("Cleanup: Deregister substrate\n");
  result = sokr_deregister_substrate(substrate_id);

  if (result != SOKR_RESULT_OK) {
    fprintf(stderr, "Failed to deregister: %s\n", sokr_result_str(result));
    return 1;
  }

  printf("  ✓ Substrate deregistered\n\n");

  printf("Example completed successfully!\n");
  return 0;
}
