//! Functions that are relevant for other bindings modules, but normally not end users.
use super::Value;
use nix_bindings_expr_sys as raw;

/// Take ownership of a new [`Value`].
///
/// This does not call `nix_gc_incref`, but does call `nix_gc_decref` when dropped.
///
/// # Safety
///
/// The caller must ensure that the provided `ptr` has a positive reference count,
/// and that `ptr` is not used after the returned `Value` is dropped.
pub unsafe fn raw_value_new(ptr: *mut raw::Value) -> Value {
    Value::new(ptr)
}

/// Borrow a reference to a [`Value`].
///
/// This calls `value_incref`, and the returned Value will call `value_decref` when dropped.
///
/// # Safety
///
/// The caller must ensure that the provided `ptr` has a positive reference count.
pub unsafe fn raw_value_new_borrowed(ptr: *mut raw::Value) -> Value {
    Value::new_borrowed(ptr)
}
