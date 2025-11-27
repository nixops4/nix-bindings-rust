use std::ptr::NonNull;

#[cfg(nix_at_least = "2.33")]
use anyhow::Context as AnyhowContext;
use anyhow::Result;
use nix_bindings_store_sys as raw;
#[cfg(nix_at_least = "2.33")]
use nix_bindings_util::{check_call, context::Context};
use nix_bindings_util::{
    result_string_init,
    string_return::{callback_get_result_string, callback_get_result_string_data},
};

/// The size of a store path hash in bytes (20 bytes, decoded from nix32).
pub const STORE_PATH_HASH_SIZE: usize = 20;

#[cfg(nix_at_least = "2.33")]
const _: () = assert!(std::mem::size_of::<raw::store_path_hash_part>() == STORE_PATH_HASH_SIZE);

pub struct StorePath {
    raw: NonNull<raw::StorePath>,
}

impl StorePath {
    /// Get the name of the store path.
    ///
    /// For a store path like `/nix/store/abc1234...-foo-1.2`, this function will return `foo-1.2`.
    pub fn name(&self) -> Result<String> {
        unsafe {
            let mut r = result_string_init!();
            raw::store_path_name(
                self.as_ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r),
            );
            r
        }
    }

    /// Get the hash part of the store path.
    ///
    /// This returns the decoded hash (not the nix32-encoded string).
    #[cfg(nix_at_least = "2.33")]
    pub fn hash(&self) -> Result<[u8; STORE_PATH_HASH_SIZE]> {
        let mut result = [0u8; STORE_PATH_HASH_SIZE];
        let hash_part: &mut raw::store_path_hash_part = zerocopy::transmute_mut!(&mut result);

        let mut ctx = Context::new();

        unsafe {
            check_call!(raw::store_path_hash(&mut ctx, self.as_ptr(), hash_part))?;
        }
        Ok(result)
    }

    /// Create a StorePath from hash and name components.
    #[cfg(nix_at_least = "2.33")]
    pub fn from_parts(hash: &[u8; STORE_PATH_HASH_SIZE], name: &str) -> Result<Self> {
        let hash_part: &raw::store_path_hash_part = zerocopy::transmute_ref!(hash);

        let mut ctx = Context::new();

        let out_path = unsafe {
            check_call!(raw::store_create_from_parts(
                &mut ctx,
                hash_part,
                name.as_ptr() as *const i8,
                name.len()
            ))?
        };

        NonNull::new(out_path)
            .map(|ptr| unsafe { Self::new_raw(ptr) })
            .context("store_create_from_parts returned null")
    }

    /// This is a low level function that you shouldn't have to call unless you are developing the Nix bindings.
    ///
    /// Construct a new `StorePath` by first cloning the C store path.
    ///
    /// # Safety
    ///
    /// This does not take ownership of the C store path, so it should be a borrowed pointer, or you should free it.
    pub unsafe fn new_raw_clone(raw: NonNull<raw::StorePath>) -> Self {
        Self::new_raw(
            NonNull::new(raw::store_path_clone(raw.as_ptr()))
                .or_else(|| panic!("nix_store_path_clone returned a null pointer"))
                .unwrap(),
        )
    }

    /// This is a low level function that you shouldn't have to call unless you are developing the Nix bindings.
    ///
    /// Takes ownership of a C `nix_store_path`. It will be freed when the `StorePath` is dropped.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided `NonNull<raw::StorePath>` is valid and that the ownership
    /// semantics are correctly followed. The `raw` pointer must not be used after being passed to this function.
    pub unsafe fn new_raw(raw: NonNull<raw::StorePath>) -> Self {
        StorePath { raw }
    }

    /// This is a low level function that you shouldn't have to call unless you are developing the Nix bindings.
    ///
    /// Get a pointer to the underlying Nix C API store path.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it returns a raw pointer. The caller must ensure that the pointer is not used beyond the lifetime of this `StorePath`.
    pub unsafe fn as_ptr(&self) -> *mut raw::StorePath {
        self.raw.as_ptr()
    }
}

impl Clone for StorePath {
    fn clone(&self) -> Self {
        unsafe { Self::new_raw_clone(self.raw) }
    }
}

impl Drop for StorePath {
    fn drop(&mut self) {
        unsafe {
            raw::store_path_free(self.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    #[cfg(nix_at_least = "2.26" /* get_storedir */)]
    fn store_path_name() {
        let mut store = crate::store::Store::open(Some("dummy://"), []).unwrap();
        let store_dir = store.get_storedir().unwrap();
        let store_path_string =
            format!("{store_dir}/rdd4pnr4x9rqc9wgbibhngv217w2xvxl-bash-interactive-5.2p26");
        let store_path = store.parse_store_path(store_path_string.as_str()).unwrap();
        assert_eq!(store_path.name().unwrap(), "bash-interactive-5.2p26");
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn store_path_round_trip() {
        let original_hash: [u8; STORE_PATH_HASH_SIZE] =
            hex!("0123456789abcdef0011223344556677deadbeef");
        let original_name = "foo.drv";

        let store_path = StorePath::from_parts(&original_hash, original_name).unwrap();

        // Round trip gets back what we started with
        assert_eq!(store_path.hash().unwrap(), original_hash);
        assert_eq!(store_path.name().unwrap(), original_name);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn store_path_clone() {
        use super::StorePath;
        // Import TryFrom impls from the harmonia integration crate
        use nix_bindings_store_harmonia as _;

        let harmonia_path: harmonia_store_core::store_path::StorePath =
            "g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-foo.drv".parse().unwrap();

        let nix_path: StorePath = (&harmonia_path).try_into().unwrap();
        let cloned_path = nix_path.clone();

        // Verify both paths have the same name and hash
        assert_eq!(nix_path.name().unwrap(), cloned_path.name().unwrap());
        assert_eq!(nix_path.hash().unwrap(), cloned_path.hash().unwrap());
    }
}
