//! Functions that are relevant for other bindings modules, but normally not end users.
use super::Value;
use nix_bindings_bindgen_raw as raw;

/// See [Value::new].
///
/// # Safety
///
/// See underlying function.
pub unsafe fn raw_value_new(ptr: *mut raw::Value) -> Value {
    Value::new(ptr)
}

/// See [Value::new_borrowed].
///
/// # Safety
///
/// See underlying function.
pub unsafe fn raw_value_new_borrowed(ptr: *mut raw::Value) -> Value {
    Value::new_borrowed(ptr)
}
