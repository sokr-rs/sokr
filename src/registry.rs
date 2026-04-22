//! Plugin registry for substrate management.
//!
//! The registry maintains the set of registered substrate plugins and
//! provides lookup by substrate ID.
//!
//! **Note**: This is a placeholder implementation. Thread-safety will be added in Phase 1.3.

use crate::types::{SokrResult, SokrSubstratePlugin};

/// Maximum number of substrate plugins that can be registered.
pub const MAX_SUBSTRATES: usize = 16;

/// Plugin registry for managing substrate plugins.
///
/// This is a placeholder implementation for Phase 1.2.
/// Full implementation with thread-safety will be added in Phase 1.3.
pub struct Registry {
    /// Fixed-size array of substrate slots.
    substrates: [Option<SokrSubstratePlugin>; MAX_SUBSTRATES],
}

impl Registry {
    /// Creates a new empty registry.
    #[must_use]
    pub const fn new() -> Self {
        const EMPTY: Option<SokrSubstratePlugin> = None;
        Self {
            substrates: [EMPTY; MAX_SUBSTRATES],
        }
    }

    /// Returns the number of registered substrates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.substrates.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if no substrates are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.substrates.iter().all(Option::is_none)
    }

    /// Returns true if the registry is at capacity.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.substrates.iter().all(Option::is_some)
    }

    /// Registers a substrate plugin.
    ///
    /// Returns `SokrResult::RegistryFull` if at capacity.
    pub fn register(&mut self, plugin: SokrSubstratePlugin) -> SokrResult {
        if self.is_full() {
            return SokrResult::RegistryFull;
        }

        for slot in &mut self.substrates {
            if slot.is_none() {
                *slot = Some(plugin);
                return SokrResult::Ok;
            }
        }

        // All slots occupied — this path is only reachable when substrates
        // are modified concurrently (not yet supported) or if is_full() desyncs.
        unreachable!("registry not full but no empty slot found")
    }

    /// Returns a reference to the substrate at the given index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&SokrSubstratePlugin> {
        self.substrates.get(index)?.as_ref()
    }

    /// Iterates over all registered substrates.
    pub fn iter(&self) -> impl Iterator<Item = &SokrSubstratePlugin> {
        self.substrates.iter().filter_map(|s| s.as_ref())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SokrResult, SokrSubstratePlugin, SokrVersion};

    extern "C" fn dummy_capability(
        _version: *const crate::types::SokrVersion,
        _query: *const crate::types::SokrCapabilityQuery,
        _response: *mut crate::types::SokrCapabilityResponse,
    ) -> SokrResult {
        SokrResult::Ok
    }

    extern "C" fn dummy_dispatch(
        _request: *const crate::types::SokrDispatchRequest,
        _response: *mut crate::types::SokrDispatchResponse,
    ) -> SokrResult {
        SokrResult::Ok
    }

    extern "C" fn dummy_completion(
        _query: *const crate::types::SokrCompletionQuery,
        _signal: *mut crate::types::SokrCompletionSignal,
    ) -> SokrResult {
        SokrResult::Ok
    }

    extern "C" fn dummy_destroy() {}

    fn dummy_plugin() -> SokrSubstratePlugin {
        SokrSubstratePlugin {
            version: SokrVersion::CURRENT,
            capability_fn: dummy_capability,
            dispatch_fn: dummy_dispatch,
            completion_fn: dummy_completion,
            destroy_fn: dummy_destroy,
            padding: [0; 16],
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = Registry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn register_increases_count() {
        let mut reg = Registry::new();
        let result = reg.register(dummy_plugin());
        assert!(result.is_ok());
        assert_eq!(reg.len(), 1);
        assert!(!reg.is_empty());
    }

    #[test]
    fn register_full_returns_error() {
        let mut reg = Registry::new();
        for _ in 0..MAX_SUBSTRATES {
            assert!(reg.register(dummy_plugin()).is_ok());
        }
        assert!(reg.is_full());
        assert!(reg.register(dummy_plugin()).is_err());
    }

    #[test]
    fn get_returns_registered() {
        let mut reg = Registry::new();
        assert!(reg.register(dummy_plugin()).is_ok());
        assert!(reg.get(0).is_some());
        assert!(reg.get(1).is_none());
    }

    #[test]
    fn iter_visits_all() {
        let mut reg = Registry::new();
        assert!(reg.register(dummy_plugin()).is_ok());
        assert!(reg.register(dummy_plugin()).is_ok());
        let count = reg.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let mut reg = Registry::new();
        assert!(reg.register(dummy_plugin()).is_ok());
        assert!(reg.get(MAX_SUBSTRATES).is_none());
        assert!(reg.get(usize::MAX).is_none());
    }
}
