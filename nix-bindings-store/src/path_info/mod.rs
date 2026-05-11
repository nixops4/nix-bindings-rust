#![cfg(nix_at_least = "2.35.0pre")]

use std::os::raw::{c_char, c_uint, c_void};
use std::ptr::NonNull;

use anyhow::Result;
use nix_bindings_store_sys as raw;
use nix_bindings_util::{
    check_call,
    context::Context,
    result_string_init,
    string_return::{callback_get_result_string, callback_get_result_string_data},
};

use crate::path::StorePath;

/// Metadata about a store path.
///
/// **Requires Nix 2.35 or later.**
pub struct PathInfo {
    pub(crate) inner: NonNull<raw::path_info>,
}

impl PathInfo {
    pub(crate) fn new_raw(inner: NonNull<raw::path_info>) -> Self {
        PathInfo { inner }
    }

    /// This is a low level function that you shouldn't have to call unless you are developing the bindings.
    ///
    /// Get a pointer to the underlying API path_info.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as this `PathInfo` is alive.
    pub unsafe fn as_ptr(&self) -> *mut raw::path_info {
        self.inner.as_ptr()
    }

    /// Get the NAR hash of this store path.
    ///
    /// Returns a string with algorithm prefix in base-32 encoding, e.g. `"sha256:1b8m..."`.
    pub fn nar_hash(&self) -> Result<String> {
        let mut ctx = Context::new();
        let mut r = result_string_init!();
        unsafe {
            check_call!(raw::path_info_get_nar_hash(
                &mut ctx,
                self.inner.as_ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))?;
        }
        r
    }

    /// Get the NAR size of this store path in bytes.
    ///
    /// Returns 0 if the size is unknown.
    pub fn nar_size(&self) -> Result<u64> {
        let mut ctx = Context::new();
        let size =
            unsafe { check_call!(raw::path_info_get_nar_size(&mut ctx, self.inner.as_ptr()))? };
        Ok(size)
    }

    /// Get the references of this store path.
    pub fn references(&self) -> Result<Vec<StorePath>> {
        let mut ctx = Context::new();
        let mut refs: Vec<StorePath> = Vec::new();

        unsafe extern "C" fn callback(user_data: *mut c_void, store_path: *const raw::StorePath) {
            let refs = &mut *(user_data as *mut Vec<StorePath>);
            let cloned = raw::store_path_clone(store_path);
            let cloned = NonNull::new(cloned).expect("store_path_clone returned null");
            refs.push(StorePath::new_raw(cloned));
        }

        let user_data = &mut refs as *mut Vec<StorePath> as *mut c_void;
        unsafe {
            check_call!(raw::path_info_get_references(
                &mut ctx,
                self.inner.as_ptr(),
                user_data,
                Some(callback)
            ))?;
        }
        Ok(refs)
    }

    /// Get the deriver of this store path, if known.
    ///
    /// Returns `None` if no deriver is recorded (e.g., paths added directly).
    pub fn deriver(&self) -> Result<Option<StorePath>> {
        let mut ctx = Context::new();
        let ptr =
            unsafe { check_call!(raw::path_info_get_deriver(&mut ctx, self.inner.as_ptr()))? };
        Ok(NonNull::new(ptr).map(|p| unsafe { StorePath::new_raw(p) }))
    }

    /// Get the signatures of this store path.
    ///
    /// Returns an empty vector for unsigned paths (e.g., a local store).
    /// Each signature has the format `"keyName:base64sig"`.
    pub fn sigs(&self) -> Result<Vec<String>> {
        let mut ctx = Context::new();
        let mut sigs: Vec<String> = Vec::new();

        unsafe extern "C" fn callback(user_data: *mut c_void, sig: *const c_char, sig_len: c_uint) {
            let sigs = &mut *(user_data as *mut Vec<String>);
            let bytes = std::slice::from_raw_parts(sig as *const u8, sig_len as usize);
            sigs.push(String::from_utf8_lossy(bytes).into_owned());
        }

        let user_data = &mut sigs as *mut Vec<String> as *mut c_void;
        unsafe {
            check_call!(raw::path_info_get_sigs(
                &mut ctx,
                self.inner.as_ptr(),
                user_data,
                Some(callback)
            ))?;
        }
        Ok(sigs)
    }

    /// Get the content address of this store path, if it is content-addressed.
    ///
    /// Returns `None` for input-addressed paths.
    pub fn ca(&self) -> Result<Option<String>> {
        let mut ctx = Context::new();
        let mut r = result_string_init!();
        unsafe {
            check_call!(raw::path_info_get_ca(
                &mut ctx,
                self.inner.as_ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))?;
        }
        Ok(r.ok().filter(|s| !s.is_empty()))
    }
}

impl Drop for PathInfo {
    fn drop(&mut self) {
        unsafe {
            raw::path_info_free(self.inner.as_ptr());
        }
    }
}
