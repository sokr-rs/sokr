//! Plugin registry for substrate management.
//!
//! The registry maintains the set of registered substrate plugins and
//! provides lookup by substrate ID.

use crate::types::{SokrResult, SokrSubstratePlugin};

/// Maximum number of substrate plugins that can be registered.
pub const MAX_SUBSTRATES: usize = 16;

/// Plugin registry for managing substrate plugins.
pub struct Registry {
    /// Fixed-size array of substrate slots.
    substrates: [Option<SokrSubstratePlugin>; MAX_SUBSTRATES],
    /// Next core-assigned substrate identifier (never returns 0).
    next_substrate_id: u64,
}

impl Registry {
    /// Creates a new empty registry.
    #[must_use]
    pub const fn new() -> Self {
        const EMPTY: Option<SokrSubstratePlugin> = None;
        Self {
            substrates: [EMPTY; MAX_SUBSTRATES],
            next_substrate_id: 1,
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

    fn allocate_substrate_id(&mut self) -> u64 {
        loop {
            let candidate = self.next_substrate_id;
            self.next_substrate_id = self.next_substrate_id.wrapping_add(1);
            if self.next_substrate_id == 0 {
                self.next_substrate_id = 1;
            }

            if candidate != 0 && self.find_by_substrate_id(candidate).is_none() {
                return candidate;
            }
        }
    }

    /// Registers a substrate plugin and returns the assigned non-zero substrate ID.
    ///
    /// Returns `Err(SokrResult::RegistryFull)` if at capacity.
    ///
    /// # Errors
    ///
    /// This function returns an error when the registry is full.
    pub fn register_with_id(&mut self, mut plugin: SokrSubstratePlugin) -> Result<u64, SokrResult> {
        if self.is_full() {
            return Err(SokrResult::RegistryFull);
        }

        let assigned_id = self.allocate_substrate_id();
        plugin.substrate_id = assigned_id;

        for slot in &mut self.substrates {
            if slot.is_none() {
                *slot = Some(plugin);
                return Ok(assigned_id);
            }
        }

        unreachable!("is_full() returned false but no empty slot found")
    }

    /// Registers a substrate plugin.
    ///
    /// Returns `SokrResult::RegistryFull` if at capacity.
    pub fn register(&mut self, plugin: SokrSubstratePlugin) -> SokrResult {
        match self.register_with_id(plugin) {
            Ok(_) => SokrResult::Ok,
            Err(err) => err,
        }
    }

    /// Deregisters a substrate plugin by its assigned substrate ID.
    ///
    /// Calls the plugin's `destroy_fn` exactly once before removal.
    pub fn deregister(&mut self, substrate_id: u64) -> SokrResult {
        for slot in &mut self.substrates {
            if let Some(plugin) = slot.as_ref() {
                if plugin.substrate_id == substrate_id {
                    let destroy_fn = plugin.destroy_fn;
                    destroy_fn();
                    *slot = None;
                    return SokrResult::Ok;
                }
            }
        }

        SokrResult::NotFound
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

    /// Finds a substrate by its unique identifier.
    #[must_use]
    pub fn find_by_substrate_id(&self, id: u64) -> Option<&SokrSubstratePlugin> {
        self.iter().find(|p| p.substrate_id == id)
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::types::{SokrResult, SokrSubstratePlugin, SokrVersion};

    static DESTROY_CALLS: AtomicUsize = AtomicUsize::new(0);

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

    extern "C" fn dummy_destroy() {
        DESTROY_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn dummy_plugin() -> SokrSubstratePlugin {
        SokrSubstratePlugin {
            version: SokrVersion::CURRENT,
            capability_fn: dummy_capability,
            dispatch_fn: dummy_dispatch,
            completion_fn: dummy_completion,
            destroy_fn: dummy_destroy,
            substrate_id: 1,
            padding: [0; 8],
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
    fn register_with_id_returns_assigned_non_zero_id() {
        let mut reg = Registry::new();
        let assigned = reg
            .register_with_id(dummy_plugin())
            .expect("registration should succeed");
        assert_ne!(assigned, 0);
        assert!(reg.find_by_substrate_id(assigned).is_some());
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

    #[test]
    fn deregister_existing_calls_destroy() {
        DESTROY_CALLS.store(0, Ordering::SeqCst);
        let mut reg = Registry::new();

        let assigned = reg
            .register_with_id(dummy_plugin())
            .expect("registration should succeed");

        assert_eq!(reg.deregister(assigned), SokrResult::Ok);
        assert_eq!(DESTROY_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn deregister_unknown_returns_not_found() {
        let mut reg = Registry::new();
        assert_eq!(reg.deregister(777), SokrResult::NotFound);
    }

    #[test]
    fn deregister_then_reregister_works() {
        let mut reg = Registry::new();

        let first = reg
            .register_with_id(dummy_plugin())
            .expect("registration should succeed");
        assert_eq!(reg.deregister(first), SokrResult::Ok);

        let second = reg
            .register_with_id(dummy_plugin())
            .expect("registration should succeed");
        assert_ne!(second, 0);
        assert!(reg.find_by_substrate_id(second).is_some());
    }
}
