#![cfg(nix_at_least = "2.31")]

use anyhow::Result;
use nix_bindings_bindgen_raw as raw;
use nix_bindings_util::context::Context;
use nix_bindings_util::string_return::{callback_get_result_string, callback_get_result_string_data};
use nix_bindings_util::{check_call, result_string_init};
use std::ptr::NonNull;

/// A Nix derivation
///
/// **Requires Nix 2.33 or later.**
pub struct Derivation {
    pub(crate) inner: NonNull<raw::derivation>,
    /* An error context to reuse. This way we don't have to allocate them for each store operation. */
    context: Context,
}

impl Derivation {
    pub(crate) fn new_raw(inner: NonNull<raw::derivation>) -> Self {
        Derivation {
            inner, 
            context: Context::new(),
        }
    }

    #[doc(alias = "nix_derivation_to_json")]
    pub fn to_json_string(&mut self) -> Result<String> {
        let mut r = result_string_init!();
        unsafe {
            check_call!(raw::derivation_to_json(
                &mut self.context,
                self.inner.as_ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))
        }?;
        r
    }
}

impl Drop for Derivation {
    fn drop(&mut self) {
        unsafe {
            raw::derivation_free(self.inner.as_ptr());
        }
    }
}
