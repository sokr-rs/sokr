/**
 * SOKR C++ API Example: Hello Compute (with RAII wrapper)
 *
 * Demonstrates idiomatic C++ usage of SOKR through the RAII wrapper.
 * Automatic resource management, type safety, and exception handling.
 *
 * Build:
 *   g++ -std=c++17 -Wall -Wextra -O2 -I../../include \
 *       -L../../target/release -L../../sokr-plugins/target/release \
 *       -o hello_compute hello_compute.cpp -lsokr_cpu -lsokr
 */

#include <sokr.hpp>
#include <iostream>
#include <iomanip>

/* Declare the CPU plugin from sokr-cpu library */
extern "C" const SokrSubstratePlugin CPU_PLUGIN;

int main() {
  std::cout << "SOKR C++ API Example: Hello Compute (RAII)\n";
  std::cout << "==========================================\n\n";

  try {
    /* Create a registry to manage substrates and operations */
    sokr::Registry registry;

    /* ────────────────────────────────────────────────────────
     * Step 1: Register the substrate plugin
     * ──────────────────────────────────────────────────────── */

    std::cout << "Step 1: Register substrate plugin\n";

    auto substrate = registry.register_substrate(CPU_PLUGIN);

    std::cout << "  ✓ Registered CPU substrate with ID: " << substrate.get() << "\n\n";

    /* ────────────────────────────────────────────────────────
     * Step 2: Query substrate capability
     * ──────────────────────────────────────────────────────── */

    std::cout << "Step 2: Query substrate capability\n";

    SokrComputationId comp_id = {
        .high = 0x0123456789ABCDEF,
        .low = 0xFEDCBA9876543210,
    };

    const char ir_data[] = "compute_payload";

    sokr::CapabilityQuery capability_query(
        comp_id, "example", ir_data, sizeof(ir_data) - 1);

    auto capability_response = registry.query_capability(capability_query);

    std::cout << "  ✓ Capability accepted by substrate "
              << capability_response.substrate_id() << "\n";
    std::cout << "    Estimated latency: " << capability_response.estimated_latency_ns()
              << " ns\n\n";

    /* ────────────────────────────────────────────────────────
     * Step 3: Dispatch computation for execution
     * ──────────────────────────────────────────────────────── */

    std::cout << "Step 3: Dispatch computation\n";

    sokr::DispatchRequest dispatch_request(
        comp_id, substrate.get(), ir_data, sizeof(ir_data) - 1);

    auto completion_token = registry.dispatch(dispatch_request);

    std::cout << "  ✓ Computation dispatched\n";
    std::cout << "    Completion token: 0x" << std::hex << std::setw(16)
              << std::setfill('0') << completion_token.handle() << std::dec << "\n\n";

    /* ────────────────────────────────────────────────────────
     * Step 4: Query completion status
     * ──────────────────────────────────────────────────────── */

    std::cout << "Step 4: Query completion status\n";

    sokr::CompletionQuery completion_query(completion_token);
    auto completion_signal = registry.query_completion(completion_query);

    std::cout << "  ✓ Completion signal: ";
    if (completion_signal.is_complete()) {
      std::cout << "Complete";
    } else if (completion_signal.is_pending()) {
      std::cout << "Pending";
    } else if (completion_signal.is_failed()) {
      std::cout << "Failed";
    } else if (completion_signal.is_timed_out()) {
      std::cout << "TimedOut";
    } else {
      std::cout << "Unknown";
    }
    std::cout << "\n\n";

    /* ────────────────────────────────────────────────────────
     * Automatic cleanup: substrate RAII object destroyed here
     * ──────────────────────────────────────────────────────── */

    std::cout << "Cleanup: Substrate automatically deregistered (RAII)\n";
    std::cout << "  ✓ Substrate deregistered\n\n";

    std::cout << "Example completed successfully!\n";
    return 0;

  } catch (const sokr::SokrError& e) {
    std::cerr << "SOKR error: " << e.what() << " (result code: " << (int)e.result()
              << ")\n";
    return 1;
  } catch (const std::exception& e) {
    std::cerr << "Error: " << e.what() << "\n";
    return 1;
  }
}
