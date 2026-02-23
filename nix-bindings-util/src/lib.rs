use crate::raw_sys as raw;
use anyhow::Result;
use std::ffi::{c_char, CStr};

pub mod context;
pub mod settings;
#[macro_use]
pub mod string_return;
pub mod nix_version;

// Re-export for use in macros
pub use nix_bindings_util_sys as raw_sys;

#[doc(alias = "nix_libutil_init")]
pub fn init() -> Result<()> {
    let mut ctx = context::Context::new();
    unsafe {
        check_call!(raw::libutil_init(&mut ctx))?;
    }
    Ok(())
}

#[doc(alias = "nix_version_get")]
pub fn get_version() -> Result<&'static str> {
    let c_str = unsafe {
        let ptr = raw::version_get();
        CStr::from_ptr(ptr as *const c_char)
    };

    Ok(c_str.to_str()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nix_version::parse_version;

    #[test]
    fn init() {
        super::init().unwrap();
    }

    #[test]
    fn version() {
        assert!(parse_version(get_version().unwrap()) > (0, 0, 0));
    }
}
