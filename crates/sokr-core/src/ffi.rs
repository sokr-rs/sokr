//! C FFI exports for SOKR core.
//!
//! This module contains all `#[no_mangle] extern "C"` functions.
//! It is the only module that uses unsafe code.

#![allow(unsafe_code)]

use crate::types::{SokrResult, SokrVersion};

#[allow(missing_docs)]
static SOKR_VERSION_STATIC: SokrVersion = SokrVersion::CURRENT;

/// Returns the current SOKR core ABI version.
///
/// # Safety
/// The returned pointer is a static reference valid for the lifetime of the program.
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
        *result = if compatibility.is_ok() { 1 } else { 0 };
    }

    compatibility
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
}
