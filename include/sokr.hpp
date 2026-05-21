/**
 * SOKR C++ RAII Wrapper
 *
 * Provides type-safe, exception-based C++ bindings to the SOKR FFI interface.
 * All resources are automatically managed via RAII (Resource Acquisition Is Initialization).
 *
 * Example:
 *   try {
 *       sokr::Registry registry;
 *       auto id = registry.register_substrate(cpu_plugin);
 *
 *       sokr::CapabilityQuery query(computation_id, "ir_format", ir_data);
 *       auto response = registry.query_capability(query);
 *
 *       sokr::DispatchRequest request(computation_id, id, ir_data);
 *       auto token = registry.dispatch(request);
 *
 *       sokr::CompletionQuery completion(token);
 *       auto signal = registry.query_completion(completion);
 *
 *   } catch (const sokr::SokrError& e) {
 *       std::cerr << "SOKR error: " << e.what() << std::endl;
 *   }
 */

#ifndef SOKR_HPP
#define SOKR_HPP

#include <sokr.h>
#include <cstdint>
#include <stdexcept>
#include <memory>
#include <string>

namespace sokr {

/**
 * Exception type for SOKR errors.
 */
class SokrError : public std::runtime_error {
public:
  explicit SokrError(SokrResult result)
      : std::runtime_error(result_string(result)), result_(result) {}

  [[nodiscard]] SokrResult result() const noexcept { return result_; }

private:
  SokrResult result_;

  static std::string result_string(SokrResult result) {
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
};

/**
 * RAII wrapper for a registered substrate.
 * Automatically deregisters the substrate on destruction.
 */
class RegisteredSubstrate {
public:
  RegisteredSubstrate(uint64_t id) noexcept : id_(id) {}

  ~RegisteredSubstrate() {
    if (id_ != 0) {
      sokr_deregister_substrate(id_);
    }
  }

  // Non-copyable
  RegisteredSubstrate(const RegisteredSubstrate&) = delete;
  RegisteredSubstrate& operator=(const RegisteredSubstrate&) = delete;

  // Movable
  RegisteredSubstrate(RegisteredSubstrate&& other) noexcept : id_(other.release()) {}
  RegisteredSubstrate& operator=(RegisteredSubstrate&& other) noexcept {
    reset(other.release());
    return *this;
  }

  [[nodiscard]] uint64_t get() const noexcept { return id_; }
  [[nodiscard]] uint64_t release() noexcept {
    auto id = id_;
    id_ = 0;
    return id;
  }
  void reset(uint64_t id = 0) noexcept {
    if (id_ != 0) {
      sokr_deregister_substrate(id_);
    }
    id_ = id;
  }

  [[nodiscard]] explicit operator bool() const noexcept { return id_ != 0; }

private:
  uint64_t id_;
};

/**
 * Wrapper for completion token.
 */
class CompletionToken {
public:
  explicit CompletionToken(uint64_t handle = 0) noexcept : handle_(handle) {}

  [[nodiscard]] uint64_t handle() const noexcept { return handle_; }
  [[nodiscard]] bool is_valid() const noexcept { return handle_ != 0; }
  [[nodiscard]] explicit operator bool() const noexcept { return is_valid(); }

private:
  uint64_t handle_;
};

/**
 * Wrapper for completion signal.
 */
class CompletionSignal {
public:
  enum class Status {
    Pending = SOKR_COMPLETION_SIGNAL_PENDING,
    Complete = SOKR_COMPLETION_SIGNAL_COMPLETE,
    Failed = SOKR_COMPLETION_SIGNAL_FAILED,
    TimedOut = SOKR_COMPLETION_SIGNAL_TIMED_OUT,
  };

  explicit CompletionSignal(Status status = Status::Pending) noexcept
      : status_(status) {}

  [[nodiscard]] Status get() const noexcept { return status_; }
  [[nodiscard]] bool is_complete() const noexcept {
    return status_ == Status::Complete;
  }
  [[nodiscard]] bool is_pending() const noexcept {
    return status_ == Status::Pending;
  }
  [[nodiscard]] bool is_failed() const noexcept { return status_ == Status::Failed; }
  [[nodiscard]] bool is_timed_out() const noexcept {
    return status_ == Status::TimedOut;
  }

private:
  Status status_;
};

/**
 * Capability query builder.
 */
class CapabilityQuery {
public:
  CapabilityQuery(SokrComputationId id, const char* ir_format, const void* ir_data,
                  std::size_t ir_data_len)
      : id_(id), ir_format_(ir_format), ir_data_(ir_data), ir_data_len_(ir_data_len) {}

  [[nodiscard]] SokrCapabilityQuery to_ffi() const noexcept {
    return SokrCapabilityQuery{
        .computation_id = id_,
        .ir_format = ir_format_,
        .ir_data_ptr = ir_data_,
        .ir_data_len = ir_data_len_,
        .padding = {0},
    };
  }

private:
  SokrComputationId id_;
  const char* ir_format_;
  const void* ir_data_;
  std::size_t ir_data_len_;
};

/**
 * Capability response wrapper.
 */
class CapabilityResponse {
public:
  explicit CapabilityResponse(const SokrCapabilityResponse& resp)
      : result_(resp.result), substrate_id_(resp.substrate_id),
        estimated_latency_ns_(resp.estimated_latency_ns) {}

  [[nodiscard]] bool is_accepted() const noexcept {
    return result_ == SOKR_RESULT_OK;
  }
  [[nodiscard]] uint64_t substrate_id() const noexcept { return substrate_id_; }
  [[nodiscard]] uint64_t estimated_latency_ns() const noexcept {
    return estimated_latency_ns_;
  }

private:
  SokrResult result_;
  uint64_t substrate_id_;
  uint64_t estimated_latency_ns_;
};

/**
 * Dispatch request builder.
 */
class DispatchRequest {
public:
  DispatchRequest(SokrComputationId id, uint64_t substrate_id, const void* ir_data,
                  std::size_t ir_data_len, const void* params = nullptr,
                  std::size_t params_len = 0)
      : id_(id), substrate_id_(substrate_id), ir_data_(ir_data),
        ir_data_len_(ir_data_len), params_(params), params_len_(params_len) {}

  [[nodiscard]] SokrDispatchRequest to_ffi() const noexcept {
    return SokrDispatchRequest{
        .computation_id = id_,
        .substrate_id = substrate_id_,
        .ir_data_ptr = ir_data_,
        .ir_data_len = ir_data_len_,
        .params_ptr = params_,
        .params_len = params_len_,
        .padding = {0},
    };
  }

private:
  SokrComputationId id_;
  uint64_t substrate_id_;
  const void* ir_data_;
  std::size_t ir_data_len_;
  const void* params_;
  std::size_t params_len_;
};

/**
 * Completion query wrapper.
 */
class CompletionQuery {
public:
  explicit CompletionQuery(CompletionToken token, uint64_t timeout_ns = 0)
      : token_(token.handle()), timeout_ns_(timeout_ns) {}

  [[nodiscard]] SokrCompletionQuery to_ffi() const noexcept {
    return SokrCompletionQuery{
        .completion_token = token_,
        .timeout_ns = timeout_ns_,
        .padding = {0},
    };
  }

private:
  uint64_t token_;
  uint64_t timeout_ns_;
};

/**
 * SOKR Registry RAII wrapper.
 *
 * Provides a type-safe interface to the SOKR FFI, with automatic resource
 * management and exception-based error handling.
 */
class Registry {
public:
  Registry() = default;

  /**
   * Register a substrate plugin.
   *
   * @param plugin The plugin to register
   * @return RAII wrapper for the registered substrate
   * @throws SokrError on registration failure
   */
  RegisteredSubstrate register_substrate(const SokrSubstratePlugin& plugin) {
    uint64_t id = 0;
    auto result = sokr_register_substrate(&plugin, &id);
    if (result != SOKR_RESULT_OK) {
      throw SokrError(result);
    }
    return RegisteredSubstrate(id);
  }

  /**
   * Query a substrate's capability.
   *
   * @param query The capability query
   * @return The capability response
   * @throws SokrError on query failure
   */
  CapabilityResponse query_capability(const CapabilityQuery& query) {
    auto ffi_query = query.to_ffi();
    SokrCapabilityResponse ffi_response = {};
    auto result = sokr_capability(&ffi_query, &ffi_response);
    if (result != SOKR_RESULT_OK) {
      throw SokrError(result);
    }
    if (ffi_response.result != SOKR_RESULT_OK) {
      throw SokrError(ffi_response.result);
    }
    return CapabilityResponse(ffi_response);
  }

  /**
   * Dispatch a computation.
   *
   * @param request The dispatch request
   * @return Completion token for tracking the computation
   * @throws SokrError on dispatch failure
   */
  CompletionToken dispatch(const DispatchRequest& request) {
    auto ffi_request = request.to_ffi();
    SokrDispatchResponse ffi_response = {};
    auto result = sokr_dispatch(&ffi_request, &ffi_response);
    if (result != SOKR_RESULT_OK) {
      throw SokrError(result);
    }
    if (ffi_response.result != SOKR_RESULT_OK) {
      throw SokrError(ffi_response.result);
    }
    return CompletionToken(ffi_response.completion_token);
  }

  /**
   * Query computation completion status.
   *
   * @param query The completion query
   * @return The completion signal
   * @throws SokrError on query failure
   */
  CompletionSignal query_completion(const CompletionQuery& query) {
    auto ffi_query = query.to_ffi();
    SokrCompletionSignal ffi_signal = SOKR_COMPLETION_SIGNAL_PENDING;
    auto result = sokr_completion(&ffi_query, &ffi_signal);
    if (result != SOKR_RESULT_OK) {
      throw SokrError(result);
    }
    return CompletionSignal(static_cast<CompletionSignal::Status>(ffi_signal));
  }

  /**
   * Get the current SOKR core version.
   *
   * @return The version information
   */
  [[nodiscard]] static const SokrVersion& version() noexcept {
    return *sokr_version();
  }
};

} // namespace sokr

#endif // SOKR_HPP
