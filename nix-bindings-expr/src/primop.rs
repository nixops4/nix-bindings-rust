use crate::eval_state::EvalState;
use crate::value::Value;
use anyhow::Result;
use nix_bindings_expr_sys as raw;
use nix_bindings_util::check_call;
use nix_bindings_util_sys as raw_util;
use std::ffi::{c_int, c_void, CStr, CString};
use std::ptr::{null, null_mut};

/// A primop error that is not memoized in the thunk that triggered it,
/// allowing the thunk to be forced again.
///
/// Since [Nix 2.34](https://nix.dev/manual/nix/2.34/release-notes/rl-2.34.html#c-api-changes),
/// primop errors are memoized by default: once a thunk fails, forcing it
/// again returns the same error. Use `RecoverableError` for errors that
/// are transient, so the caller can retry.
///
/// On Nix < 2.34, all errors are already recoverable, so this type has
/// no additional effect.
///
/// Available since nix-bindings-expr 0.2.1.
#[derive(Debug)]
pub struct RecoverableError(String);

impl RecoverableError {
    pub fn new(msg: impl Into<String>) -> Self {
        RecoverableError(msg.into())
    }
}

impl std::fmt::Display for RecoverableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for RecoverableError {}

/// Metadata for a primop, used with `PrimOp::new`.
pub struct PrimOpMeta<'a, const N: usize> {
    /// Name of the primop. Note that primops do not have to be registered as
    /// builtins. Nonetheless, a name is required for documentation purposes, e.g.
    /// :doc in the repl.
    pub name: &'a CStr,

    /// Documentation for the primop. This is displayed in the repl when using
    /// :doc. The format is markdown.
    pub doc: &'a CStr,

    /// The number of arguments the function takes, as well as names for the
    /// arguments, to be presented in the documentation (if applicable, e.g.
    /// :doc in the repl).
    pub args: [&'a CStr; N],
}

pub struct PrimOp<'a> {
    ptr: *mut raw::PrimOp,
    eval_state: &'a mut EvalState,
}
impl Drop for PrimOp<'_> {
    fn drop(&mut self) {
        unsafe {
            raw::gc_decref(null_mut(), self.ptr as *mut c_void);
        }
    }
}
impl<'a> PrimOp<'a> {
    /// Create a new primop with the given metadata and implementation.
    ///
    /// When `f` returns an `Err`, the error is propagated to the Nix evaluator.
    /// To return a [recoverable error](RecoverableError), include it in the
    /// error chain (e.g. `Err(RecoverableError::new("...").into())`).
    pub fn new<const N: usize>(
        eval_state: &'a mut EvalState,
        meta: PrimOpMeta<N>,
        f: Box<dyn Fn(&mut EvalState, &[Value; N]) -> Result<Value>>,
    ) -> Result<PrimOp<'a>> {
        assert!(N != 0);

        let mut args = Vec::new();
        for arg in meta.args {
            args.push(arg.as_ptr());
        }
        args.push(null());

        // Primops weren't meant to be dynamically created, as of writing.
        // This leaks, and so do the primop fields in Nix internally.
        let user_data = {
            // We'll be leaking this Box.
            // TODO: Use the GC with finalizer, if possible.
            let user_data = Box::leak(Box::new(PrimOpContext {
                arity: N,
                function: Box::new(move |eval_state, args| f(eval_state, args.try_into().unwrap())),
                eval_state,
            }));
            user_data as *const PrimOpContext as *mut c_void
        };
        let op = unsafe {
            check_call!(raw::alloc_primop(
                &mut eval_state.context,
                FUNCTION_ADAPTER,
                N as c_int,
                meta.name.as_ptr(),
                args.as_mut_ptr(), /* TODO add an extra const to bindings to avoid mut here. */
                meta.doc.as_ptr(),
                user_data
            ))?
        };

        Ok(PrimOp {
            ptr: op,
            eval_state,
        })
    }

    /// Creates a new [`function`](crate::value::ValueType::Function) Nix value implemented by a Rust function.
    ///
    /// This is also known as a "primop" in Nix, short for primitive operation.
    /// Most of the `builtins.*` values are examples of primops, but this function
    /// does not affect `builtins`.
    #[doc(alias = "make_primop")]
    #[doc(alias = "create_function")]
    #[doc(alias = "builtin")]
    pub fn new_value(mut self) -> Result<Value> {
        self.with_state_and_ptr(|ptr, this| {
            let value = this.new_value_uninitialized()?;
            unsafe {
                check_call!(raw::init_primop(&mut this.context, value.raw_ptr(), ptr))?;
            };
            Ok(value)
        })
    }

    pub(crate) fn with_state_and_ptr<F, T>(&mut self, f: F) -> T
    where
        F: Fn(*mut raw::PrimOp, &mut EvalState) -> T,
    {
        f(self.ptr, self.eval_state)
    }
}

/// The user_data for our Nix primops
struct PrimOpContext<'a> {
    arity: usize,
    function: Box<dyn Fn(&mut EvalState, &[Value]) -> Result<Value>>,
    eval_state: &'a mut EvalState,
}

unsafe extern "C" fn function_adapter(
    user_data: *mut ::std::os::raw::c_void,
    context_out: *mut raw_util::c_context,
    _state: *mut raw::EvalState,
    args: *mut *mut raw::Value,
    ret: *mut raw::Value,
) {
    let primop_info = (user_data as *mut PrimOpContext).as_mut().unwrap();
    let args_raw_slice = unsafe { std::slice::from_raw_parts(args, primop_info.arity) };
    let args_vec: Vec<Value> = args_raw_slice
        .iter()
        .map(|v| Value::new_borrowed(*v))
        .collect();
    let args_slice = args_vec.as_slice();

    let r = primop_info.function.as_ref()(primop_info.eval_state, args_slice);

    match r {
        Ok(v) => unsafe {
            raw::copy_value(context_out, ret, v.raw_ptr());
        },
        Err(e) => unsafe {
            let err_code = error_code(&e);
            let cstr = CString::new(e.to_string()).unwrap_or_else(|_e| {
                CString::new("<rust nix-expr application error message contained null byte>")
                    .unwrap()
            });
            raw_util::set_err_msg(context_out, err_code, cstr.as_ptr());
        },
    }
}

fn error_code(e: &anyhow::Error) -> raw_util::err {
    #[cfg(nix_at_least = "2.34.0pre")]
    if e.downcast_ref::<RecoverableError>().is_some() {
        return raw_util::err_NIX_ERR_RECOVERABLE;
    }
    raw_util::err_NIX_ERR_UNKNOWN
}

static FUNCTION_ADAPTER: raw::PrimOpFun = Some(function_adapter);
