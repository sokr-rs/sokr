#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Maximum number of substrate plugins that can be registered.
 */
#define MAX_SUBSTRATES 16

/**
 * Result codes for SOKR operations.
 */
enum SokrResult
#ifdef __cplusplus
  : uint32_t
#endif // __cplusplus
 {
  /**
   * Operation succeeded.
   */
  SOKR_RESULT_OK = 0,
  /**
   * Substrate cannot fulfill this computation.
   */
  SOKR_RESULT_CAPABILITY_DENIED = 1,
  /**
   * Dispatch failed at runtime.
   */
  SOKR_RESULT_DISPATCH_FAILED = 2,
  /**
   * Operation timed out.
   */
  SOKR_RESULT_TIMEOUT = 3,
  /**
   * Plugin ABI version incompatible with core.
   */
  SOKR_RESULT_VERSION_MISMATCH = 4,
  /**
   * No registered substrate can fulfill this computation.
   */
  SOKR_RESULT_NO_CAPABLE_SUBSTRATE = 5,
  /**
   * Invalid input parameters.
   */
  SOKR_RESULT_INVALID_INPUT = 6,
  /**
   * Invalid IR format.
   */
  SOKR_RESULT_INVALID_IR = 7,
  /**
   * Resource not found.
   */
  SOKR_RESULT_NOT_FOUND = 8,
  /**
   * Plugin registry is full.
   */
  SOKR_RESULT_REGISTRY_FULL = 9,
};
#ifndef __cplusplus
typedef uint32_t SokrResult;
#endif // __cplusplus

/**
 * Completion status signal.
 */
enum SokrCompletionSignal
#ifdef __cplusplus
  : uint32_t
#endif // __cplusplus
 {
  /**
   * Operation is still pending.
   */
  SOKR_COMPLETION_SIGNAL_PENDING = 0,
  /**
   * Operation completed successfully.
   */
  SOKR_COMPLETION_SIGNAL_COMPLETE = 1,
  /**
   * Operation failed.
   */
  SOKR_COMPLETION_SIGNAL_FAILED = 2,
  /**
   * Operation timed out.
   */
  SOKR_COMPLETION_SIGNAL_TIMED_OUT = 3,
};
#ifndef __cplusplus
typedef uint32_t SokrCompletionSignal;
#endif // __cplusplus

/**
 * Version handshake struct for plugin compatibility negotiation.
 *
 * ## Version Compatibility Rules
 *
 * | Core Version | Plugin Version | Compatible? | Reason |
 * |--------------|----------------|-------------|--------|
 * | 1.2.3 | 1.1.0 | ✅ Yes | Same major, plugin minor ≤ core minor |
 * | 1.2.3 | 1.2.0 | ✅ Yes | Same major, same minor |
 * | 1.2.3 | 1.3.0 | ❌ No | Plugin minor > core minor |
 * | 1.2.3 | 2.0.0 | ❌ No | Major version mismatch |
 * | 1.2.3 | 0.9.0 | ❌ No | Major version mismatch |
 *
 * ## Negotiation Sequence
 *
 * 1. Core sends its version pointer as first argument to `capability_fn`
 * 2. Plugin inspects core version and determines compatibility
 * 3. Plugin returns `VersionMismatch` if incompatible (never panics)
 * 4. On success, plugin fills response and returns `Ok`
 *
 * ## Forward Compatibility
 *
 * - Newer plugin on older core: Plugin must check and return `VersionMismatch`
 * - Older plugin on newer core: Allowed if major matches and plugin minor ≤ core minor
 *
 * ## Version Bump Triggers
 *
 * - **Major**: Any breaking change to C ABI (struct layout, function signatures)
 * - **Minor**: New features, new result codes, new optional fields (backwards compatible)
 * - **Patch**: Documentation fixes, implementation corrections (no ABI change)
 */
typedef struct SokrVersion {
  /**
   * Major version - must match between core and plugin.
   */
  uint32_t major;
  /**
   * Minor version - plugin must be ≤ core.
   */
  uint32_t minor;
  /**
   * Patch version - informational only.
   */
  uint32_t patch;
} SokrVersion;
/**
 * Current SOKR core ABI version (0.2.0).
 */
#define SokrVersion_CURRENT (SokrVersion){ .major = 0, .minor = 2, .patch = 0 }

/**
 * Opaque 128-bit identifier for a computation unit.
 */
typedef struct SokrComputationId {
  /**
   * High 64 bits of the identifier.
   */
  uint64_t high;
  /**
   * Low 64 bits of the identifier.
   */
  uint64_t low;
} SokrComputationId;

/**
 * Computation descriptor for capability queries.
 */
typedef struct SokrCapabilityQuery {
  /**
   * Computation to query capability for.
   */
  struct SokrComputationId computation_id;
  /**
   * IR format identifier (null-terminated C string).
   */
  const char *ir_format;
  /**
   * Pointer to IR data.
   */
  const void *ir_data_ptr;
  /**
   * Length of IR data in bytes.
   */
  uintptr_t ir_data_len;
  /**
   * Reserved padding for ABI alignment.
   */
  uint8_t padding[8];
} SokrCapabilityQuery;

/**
 * Response from a capability query.
 */
typedef struct SokrCapabilityResponse {
  /**
   * Result of the capability query.
   */
  SokrResult result;
  /**
   * Reserved padding for ABI alignment.
   */
  uint32_t padding;
  /**
   * Substrate that can fulfill this computation (if capable).
   */
  uint64_t substrate_id;
  /**
   * Estimated latency in nanoseconds (0 if unknown).
   */
  uint64_t estimated_latency_ns;
} SokrCapabilityResponse;

/**
 * Capability query function pointer type.
 *
 * # Pointer contract
 * All pointers are valid and non-null for the duration of the call.
 * Implementations MUST NOT retain any pointer past return.
 * Return `SokrResult::CapabilityDenied` to disclaim the computation
 * without claiming ownership; any other non-`Ok` result is propagated
 * to the caller as a hard failure.
 */
typedef SokrResult (*SokrCapabilityFn)(const struct SokrVersion *version,
                                       const struct SokrCapabilityQuery *query,
                                       struct SokrCapabilityResponse *response);

/**
 * Dispatch payload struct.
 */
typedef struct SokrDispatchRequest {
  /**
   * Computation to dispatch.
   */
  struct SokrComputationId computation_id;
  /**
   * Substrate to dispatch to.
   */
  uint64_t substrate_id;
  /**
   * Pointer to IR data.
   */
  const void *ir_data_ptr;
  /**
   * Length of IR data in bytes.
   */
  uintptr_t ir_data_len;
  /**
   * Pointer to dispatch parameters.
   */
  const void *params_ptr;
  /**
   * Length of parameters in bytes.
   */
  uintptr_t params_len;
  /**
   * Reserved padding for ABI alignment and future extension.
   */
  uint8_t padding[16];
} SokrDispatchRequest;

/**
 * Opaque 64-bit completion handle.
 *
 * ## Valid Token Contract
 * - `handle = 0` is reserved as the "invalid / unset" sentinel
 * - Valid tokens are always non-zero (assigned by substrate on successful dispatch)
 * - Callers receiving `handle = 0` on error should not use it for completion queries
 */
typedef struct SokrCompletionToken {
  /**
   * Opaque handle identifying this completion.
   * Value of 0 indicates an invalid or unset token.
   */
  uint64_t handle;
} SokrCompletionToken;

/**
 * Response from a dispatch request.
 */
typedef struct SokrDispatchResponse {
  /**
   * Result of the dispatch request.
   */
  SokrResult result;
  /**
   * Reserved padding for ABI alignment.
   */
  uint32_t padding;
  /**
   * Token to query completion status.
   */
  struct SokrCompletionToken completion_token;
} SokrDispatchResponse;

/**
 * Dispatch function pointer type.
 *
 * # Pointer contract
 * All pointers are valid and non-null for the duration of the call.
 * Implementations MUST NOT retain any pointer past return.
 * On non-`Ok` return the core zeroes `response`; any error is propagated
 * verbatim to the caller.
 */
typedef SokrResult (*SokrDispatchFn)(const struct SokrDispatchRequest *request,
                                     struct SokrDispatchResponse *response);

/**
 * Query for completion status.
 */
typedef struct SokrCompletionQuery {
  /**
   * Completion token to query.
   */
  struct SokrCompletionToken completion_token;
  /**
   * Timeout in nanoseconds (0 for no timeout).
   */
  uint64_t timeout_ns;
  /**
   * Reserved padding for ABI alignment.
   */
  uint8_t padding[8];
} SokrCompletionQuery;

/**
 * Completion query function pointer type.
 *
 * # Pointer contract
 * All pointers are valid and non-null for the duration of the call.
 * Implementations MUST NOT retain any pointer past return.
 * Return `SokrResult::NotFound` to disclaim the token without claiming
 * ownership; any other non-`Ok` result is propagated to the caller as
 * a hard failure and `signal` is set to `SokrCompletionSignal::Failed`.
 */
typedef SokrResult (*SokrCompletionFn)(const struct SokrCompletionQuery *query,
                                       SokrCompletionSignal *signal);

/**
 * Cleanup function called when a plugin is deregistered.
 */
typedef void (*SokrDestroyFn)(void);

/**
 * `VTable` struct for substrate plugins.
 */
typedef struct SokrSubstratePlugin {
  /**
   * Plugin ABI version for compatibility check.
   */
  struct SokrVersion version;
  /**
   * Capability query function pointer.
   */
  SokrCapabilityFn capability_fn;
  /**
   * Dispatch function pointer.
   */
  SokrDispatchFn dispatch_fn;
  /**
   * Completion query function pointer.
   */
  SokrCompletionFn completion_fn;
  /**
   * Cleanup function called on deregistration.
   */
  SokrDestroyFn destroy_fn;
  /**
   * Unique identifier for this substrate plugin.
   */
  uint64_t substrate_id;
  /**
   * Reserved padding for ABI alignment.
   */
  uint8_t padding[8];
} SokrSubstratePlugin;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Static export of current SOKR version for C FFI.
 *
 * This is exported as `extern const` (valid at file scope in C),
 * unlike the `SokrVersion_CURRENT` macro which uses compound literals.
 */
extern const struct SokrVersion SOKR_VERSION_CURRENT;

#if defined(SOKR_FFI_ENABLED)
/**
 * Returns the current SOKR core ABI version.
 *
 * # Returns
 * Pointer to a `static` `SokrVersion` valid for the lifetime of the program.
 * Do not free or modify it.
 */
 const struct SokrVersion *sokr_version(void) ;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Checks if a plugin version is compatible with this core.
 *
 * # Arguments
 * - `plugin`: Pointer to the plugin's version struct
 * - `result`: Pointer to store the result (`1` for compatible, `0` for not)
 *
 * # Returns
 * - `SokrResult::Ok` on success
 * - `SokrResult::InvalidInput` if either pointer is null
 *
 * # Safety
 * Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
 */
 SokrResult sokr_check_version(const struct SokrVersion *plugin, int32_t *result) ;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Registers a substrate plugin with the core registry.
 *
 * # Arguments
 * - `plugin`: Pointer to plugin vtable
 * - `substrate_id_out`: Output pointer for assigned non-zero substrate ID
 *
 * # Returns
 * - `SokrResult::Ok` on success
 * - `SokrResult::InvalidInput` if pointers are null or any vtable fn pointer is null
 * - `SokrResult::VersionMismatch` if plugin ABI is incompatible
 * - `SokrResult::RegistryFull` if registry has no free slots
 *
 * # Safety
 * Non-null pointers must be valid and properly aligned.
 */

SokrResult sokr_register_substrate(const struct SokrSubstratePlugin *plugin,
                                   uint64_t *substrate_id_out)
;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Deregisters a substrate plugin by ID.
 *
 * # Returns
 * - `SokrResult::Ok` if deregistered
 * - `SokrResult::InvalidInput` if `substrate_id` is 0
 * - `SokrResult::NotFound` if ID is unknown
 */
 SokrResult sokr_deregister_substrate(uint64_t substrate_id) ;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Lists currently registered substrate IDs.
 *
 * # Arguments
 * - `substrate_ids_out`: Output buffer for substrate IDs (may be null only when `capacity == 0`)
 * - `capacity`: Number of `u64` entries available in `substrate_ids_out`
 * - `count_out`: Output pointer for total number of registered substrates
 *
 * # Returns
 * - `SokrResult::Ok` if all IDs were written
 * - `SokrResult::RegistryFull` if `capacity` is smaller than the number of registered substrates
 * - `SokrResult::InvalidInput` if pointers are invalid
 *
 * # Safety
 * - `count_out` must be non-null and valid for writes
 * - If `capacity > 0`, `substrate_ids_out` must be non-null and valid for `capacity` writes
 */

SokrResult sokr_list_substrates(uint64_t *substrate_ids_out,
                                uintptr_t capacity,
                                uintptr_t *count_out)
;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Queries a substrate's capability to fulfill a computation.
 *
 * # Arguments
 * - `query`: Pointer to capability query descriptor
 * - `response`: Pointer to store the response
 *
 * # Returns
 * - `SokrResult::Ok` on success
 * - `SokrResult::InvalidInput` if either pointer is null or IR data length is zero
 * - `SokrResult::CapabilityDenied` if no registered substrate accepts the computation
 *
 * # Response Contract (Failure Path)
 * On `InvalidInput` or `CapabilityDenied`, the response is fully zeroed
 * (`result`, `padding`, `substrate_id`, `estimated_latency_ns`).
 * Callers must not read uninitialized fields.
 *
 * # Safety
 * Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
 */

SokrResult sokr_capability(const struct SokrCapabilityQuery *query,
                           struct SokrCapabilityResponse *response)
;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Dispatches a computation to a substrate for execution.
 *
 * # Arguments
 * - `request`: Pointer to dispatch request descriptor
 * - `response`: Pointer to store the response (contains completion token)
 *
 * # Returns
 * - `SokrResult::Ok` on successful dispatch
 * - `SokrResult::InvalidInput` if either pointer is null or IR data length is zero
 * - `SokrResult::NoCapableSubstrate` if no substrate can fulfill the computation
 *
 * # Response Contract (Failure Path)
 * On any non-`Ok` result, the response is fully zeroed (`result`, `padding`,
 * `completion_token.handle = 0`). Callers must not read uninitialized fields.
 *
 * # Safety
 * Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
 */

SokrResult sokr_dispatch(const struct SokrDispatchRequest *request,
                         struct SokrDispatchResponse *response)
;
#endif

#if defined(SOKR_FFI_ENABLED)
/**
 * Queries the completion status of a dispatched computation.
 *
 * # Arguments
 * - `query`: Pointer to completion query descriptor
 * - `signal`: Pointer to store the completion signal
 *
 * # Returns
 * - `SokrResult::Ok` on success (a substrate recognized the token)
 * - `SokrResult::NotFound` if no registered substrate recognizes the token
 * - `SokrResult::InvalidInput` if either pointer is null or token handle is 0 (invalid)
 *
 * # Signal Contract (Failure Path)
 * On any non-`Ok` result where `signal` is non-null, `signal` is set to
 * `SokrCompletionSignal::Failed`. Exception: when `signal` itself is null,
 * no write occurs. Callers must not treat a failed query as `Pending`.
 *
 * # Timeout Behavior
 * - `timeout_ns = 0`: Non-blocking poll, returns immediately with current status
 * - `timeout_ns > 0`: Blocks up to the specified nanoseconds per plugin attempt
 *   (TODO: implement in Phase 1.3). With N registered plugins, the worst-case
 *   wall time before returning is `N × timeout_ns`.
 *
 * # Safety
 * Pointers must be properly aligned if non-null. Null pointers return `InvalidInput`.
 */
 SokrResult sokr_completion(const struct SokrCompletionQuery *query, SokrCompletionSignal *signal) ;
#endif

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
