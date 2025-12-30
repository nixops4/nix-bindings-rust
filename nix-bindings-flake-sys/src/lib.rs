#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

use nix_bindings_expr_sys::*;
use nix_bindings_fetchers_sys::*;
use nix_bindings_util_sys::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
