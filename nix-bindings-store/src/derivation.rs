#![cfg(nix_at_least = "2.33")]

use nix_bindings_bindgen_raw as raw;
use std::ptr::NonNull;

/// A Nix derivation
///
/// **Requires Nix 2.33 or later.**
pub struct Derivation {
    pub(crate) inner: NonNull<raw::derivation>,
}

impl Derivation {
    pub(crate) fn new_raw(inner: NonNull<raw::derivation>) -> Self {
        Derivation { inner }
    }

    /// This is a low level function that you shouldn't have to call unless you are developing the Nix bindings.
    ///
    /// Construct a new `Derivation` by first cloning the C derivation.
    ///
    /// # Safety
    ///
    /// This does not take ownership of the C derivation, so it should be a borrowed pointer, or you should free it.
    pub unsafe fn new_raw_clone(inner: NonNull<raw::derivation>) -> Self {
        Self::new_raw(
            NonNull::new(raw::derivation_clone(inner.as_ptr()))
                .or_else(|| panic!("nix_derivation_clone returned a null pointer"))
                .unwrap(),
        )
    }

    /// This is a low level function that you shouldn't have to call unless you are developing the Nix bindings.
    ///
    /// Get a pointer to the underlying Nix C API derivation.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it returns a raw pointer. The caller must ensure that the pointer is not used beyond the lifetime of this `Derivation`.
    pub unsafe fn as_ptr(&self) -> *mut raw::derivation {
        self.inner.as_ptr()
    }
}

impl Clone for Derivation {
    fn clone(&self) -> Self {
        unsafe { Self::new_raw_clone(self.inner) }
    }
}

impl Drop for Derivation {
    fn drop(&mut self) {
        unsafe {
            raw::derivation_free(self.inner.as_ptr());
        }
    }
}
