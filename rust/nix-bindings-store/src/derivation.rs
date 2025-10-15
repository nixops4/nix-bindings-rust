use nix_bindings_bindgen_raw as raw;
use std::ptr::NonNull;

/// A Nix derivation
pub struct Derivation {
    pub(crate) inner: NonNull<raw::derivation>,
}

impl Derivation {
    pub(crate) fn new_raw(inner: NonNull<raw::derivation>) -> Self {
        Derivation { inner }
    }
}

impl Drop for Derivation {
    fn drop(&mut self) {
        unsafe {
            raw::derivation_free(self.inner.as_ptr());
        }
    }
}
