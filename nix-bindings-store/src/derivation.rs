#![cfg(nix_at_least = "2.33.0pre")]

use nix_bindings_store_sys as raw;
#[cfg(nix_at_least = "2.33")]
use nix_bindings_util::{
    check_call,
    context::Context,
    result_string_init,
    string_return::{callback_get_result_string, callback_get_result_string_data},
};
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

    /// Convert the derivation to JSON (which is encoded to a string).
    ///
    /// **Requires Nix 2.33 or later.**
    ///
    /// The JSON format follows the [Nix derivation JSON schema](https://nix.dev/manual/nix/latest/protocols/json/derivation.html).
    /// Note that this format is experimental as of writing.
    #[cfg(nix_at_least = "2.33")]
    pub fn to_json_string(&self) -> anyhow::Result<String> {
        let mut ctx = Context::new();

        unsafe {
            let mut r = result_string_init!();
            check_call!(raw::derivation_to_json(
                &mut ctx,
                self.inner.as_ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))?;
            r
        }
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
