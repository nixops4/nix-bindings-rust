use anyhow::{bail, Error, Result};
use lazy_static::lazy_static;
use nix_bindings_util::context::Context;
use nix_bindings_util::string_return::{
    callback_get_result_string, callback_get_result_string_data,
};
use nix_bindings_util::{check_call, result_string_init};
use nix_bindings_util_sys as raw;
#[cfg(nix_at_least = "2.33.0pre")]
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::{c_char, CString};
use std::ptr::null_mut;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, Weak};

#[cfg(nix_at_least = "2.33.0pre")]
use crate::derivation::Derivation;
use crate::path::StorePath;

/* TODO make Nix itself thread safe */
lazy_static! {
    static ref INIT: Result<()> = unsafe {
        check_call!(raw::libstore_init(&mut Context::new()))?;
        Ok(())
    };
}

struct StoreRef {
    inner: NonNull<raw::Store>,
}
impl StoreRef {
    /// # Safety
    ///
    /// The returned pointer is only valid as long as the `StoreRef` is alive.
    pub unsafe fn ptr(&self) -> *mut raw::Store {
        self.inner.as_ptr()
    }
}
impl Drop for StoreRef {
    fn drop(&mut self) {
        unsafe {
            raw::store_free(self.inner.as_ptr());
        }
    }
}
unsafe impl Send for StoreRef {}
/// Unlike pointers in general, operations on raw::Store are thread safe and it is therefore safe to share them between threads.
unsafe impl Sync for StoreRef {}

/// A [Weak] reference to a store.
pub struct StoreWeak {
    inner: Weak<StoreRef>,
}
impl StoreWeak {
    /// Upgrade the weak reference to a proper [Store].
    ///
    /// If no normal reference to the [Store] is around anymore elsewhere, this fails by returning `None`.
    pub fn upgrade(&self) -> Option<Store> {
        self.inner.upgrade().map(|inner| Store {
            inner,
            context: Context::new(),
        })
    }
}

/// Protects against https://github.com/NixOS/nix/issues/11979 (unless different parameters are passed, in which case it's up to luck, but you do get your own parameters as you asked for).
type StoreCacheMap = HashMap<(Option<String>, Vec<(String, String)>), StoreWeak>;

lazy_static! {
    static ref STORE_CACHE: Arc<Mutex<StoreCacheMap>> = Arc::new(Mutex::new(HashMap::new()));
}

#[cfg(nix_at_least = "2.33.0pre")]
unsafe extern "C" fn callback_get_result_store_path_set(
    _context: *mut raw::c_context,
    user_data: *mut std::os::raw::c_void,
    store_path: *const raw::StorePath,
) {
    let ret = user_data as *mut Vec<StorePath>;
    let ret: &mut Vec<StorePath> = &mut *ret;

    let store_path = raw::store_path_clone(store_path);

    let store_path =
        NonNull::new(store_path).expect("nix_store_parse_path returned a null pointer");
    let store_path = StorePath::new_raw(store_path);
    ret.push(store_path);
}

#[cfg(nix_at_least = "2.33.0pre")]
fn callback_get_result_store_path_set_data(vec: &mut Vec<StorePath>) -> *mut std::os::raw::c_void {
    vec as *mut Vec<StorePath> as *mut std::os::raw::c_void
}

pub struct Store {
    inner: Arc<StoreRef>,
    /* An error context to reuse. This way we don't have to allocate them for each store operation. */
    context: Context,
}
impl Store {
    /// Open a store.
    ///
    /// See [`nix_bindings_util_sys::store_open`] for more information.
    #[doc(alias = "nix_store_open")]
    pub fn open<'a, 'b>(
        url: Option<&str>,
        params: impl IntoIterator<Item = (&'a str, &'b str)>,
    ) -> Result<Self> {
        let params = params
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<Vec<(String, String)>>();
        let params2 = params.clone();
        let mut store_cache = STORE_CACHE
            .lock()
            .map_err(|_| Error::msg("Failed to lock store cache. This should never happen."))?;
        match store_cache.entry((url.map(Into::into), params)) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if let Some(store) = e.get().upgrade() {
                    Ok(store)
                } else {
                    let store = Self::open_uncached(
                        url,
                        params2.iter().map(|(k, v)| (k.as_str(), v.as_str())),
                    )?;
                    e.insert(store.weak_ref());
                    Ok(store)
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let store = Self::open_uncached(
                    url,
                    params2.iter().map(|(k, v)| (k.as_str(), v.as_str())),
                )?;
                e.insert(store.weak_ref());
                Ok(store)
            }
        }
    }
    fn open_uncached<'a, 'b>(
        url: Option<&str>,
        params: impl IntoIterator<Item = (&'a str, &'b str)>,
    ) -> Result<Self> {
        let x = INIT.as_ref();
        match x {
            Ok(_) => {}
            Err(e) => {
                // Couldn't just clone the error, so we have to print it here.
                bail!("nix_libstore_init error: {}", e);
            }
        }

        let mut context: Context = Context::new();

        let uri_cstring = match url {
            Some(url) => Some(CString::new(url)?),
            None => None,
        };
        let uri_ptr = uri_cstring
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(null_mut());

        // this intermediate value must be here and must not be moved
        // because it owns the data the `*const c_char` pointers point to.
        let params: Vec<(CString, CString)> = params
            .into_iter()
            .map(|(k, v)| Ok((CString::new(k)?, CString::new(v)?))) // to do. context
            .collect::<Result<_>>()?;
        // this intermediate value owns the data the `*mut *const c_char` pointer points to.
        let mut params: Vec<_> = params
            .iter()
            .map(|(k, v)| [k.as_ptr(), v.as_ptr()])
            .collect();
        // this intermediate value owns the data the `*mut *mut *const c_char` pointer points to.
        let mut params: Vec<*mut *const c_char> = params
            .iter_mut()
            .map(|t| t.as_mut_ptr())
            .chain(std::iter::once(null_mut())) // signal the end of the array
            .collect();

        let store =
            unsafe { check_call!(raw::store_open(&mut context, uri_ptr, params.as_mut_ptr())) }?;
        if store.is_null() {
            panic!("nix_c_store_open returned a null pointer without an error");
        }
        let store = Store {
            inner: Arc::new(StoreRef {
                inner: NonNull::new(store).unwrap(),
            }),
            context,
        };
        Ok(store)
    }

    /// # Safety
    ///
    /// The returned pointer is only valid as long as the `Store` is alive.
    pub unsafe fn raw_ptr(&self) -> *mut raw::Store {
        self.inner.ptr()
    }

    #[doc(alias = "nix_store_get_uri")]
    pub fn get_uri(&mut self) -> Result<String> {
        let mut r = result_string_init!();
        unsafe {
            check_call!(raw::store_get_uri(
                &mut self.context,
                self.inner.ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))
        }?;
        r
    }

    #[cfg(nix_at_least = "2.26")]
    #[doc(alias = "nix_store_get_storedir")]
    pub fn get_storedir(&mut self) -> Result<String> {
        let mut r = result_string_init!();
        unsafe {
            check_call!(raw::store_get_storedir(
                &mut self.context,
                self.inner.ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))
        }?;
        r
    }

    #[doc(alias = "nix_store_parse_path")]
    pub fn parse_store_path(&mut self, path: &str) -> Result<StorePath> {
        let path = CString::new(path)?;
        unsafe {
            let store_path = check_call!(raw::store_parse_path(
                &mut self.context,
                self.inner.ptr(),
                path.as_ptr()
            ))?;
            let store_path =
                NonNull::new(store_path).expect("nix_store_parse_path returned a null pointer");
            Ok(StorePath::new_raw(store_path))
        }
    }

    #[doc(alias = "nix_store_real_path")]
    pub fn real_path(&mut self, path: &StorePath) -> Result<String> {
        let mut r = result_string_init!();
        unsafe {
            check_call!(raw::store_real_path(
                &mut self.context,
                self.inner.ptr(),
                path.as_ptr(),
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))
        }?;
        r
    }

    /// Parse a derivation from JSON.
    ///
    /// **Requires Nix 2.33 or later.**
    ///
    /// The JSON format follows the [Nix derivation JSON schema](https://nix.dev/manual/nix/latest/protocols/json/derivation.html).
    /// Note that this format is experimental as of writing.
    /// The derivation is not added to the store; use [`Store::add_derivation`] for that.
    ///
    /// # Parameters
    /// - `json`: A JSON string representing the derivation
    ///
    /// # Returns
    /// A [`Derivation`] object if parsing succeeds, or an error if the JSON is invalid
    /// or malformed.
    #[cfg(nix_at_least = "2.33.0pre")]
    #[doc(alias = "nix_derivation_from_json")]
    pub fn derivation_from_json(&mut self, json: &str) -> Result<Derivation> {
        let json_cstr = CString::new(json)?;
        unsafe {
            let drv = check_call!(raw::derivation_from_json(
                &mut self.context,
                self.inner.ptr(),
                json_cstr.as_ptr()
            ))?;
            let inner = NonNull::new(drv)
                .ok_or_else(|| Error::msg("derivation_from_json returned null"))?;
            Ok(Derivation::new_raw(inner))
        }
    }

    /// Add a derivation to the store.
    ///
    /// **Requires Nix 2.33 or later.**
    ///
    /// This computes the store path for the derivation and registers it in the store.
    /// The derivation itself is written to the store as a `.drv` file.
    ///
    /// # Parameters
    /// - `drv`: The derivation to add
    ///
    /// # Returns
    /// The store path of the derivation (ending in `.drv`).
    #[cfg(nix_at_least = "2.33.0pre")]
    #[doc(alias = "nix_add_derivation")]
    pub fn add_derivation(&mut self, drv: &Derivation) -> Result<StorePath> {
        unsafe {
            let path = check_call!(raw::add_derivation(
                &mut self.context,
                self.inner.ptr(),
                drv.inner.as_ptr()
            ))?;
            let path =
                NonNull::new(path).ok_or_else(|| Error::msg("add_derivation returned null"))?;
            Ok(StorePath::new_raw(path))
        }
    }

    /// Build a derivation and return its outputs.
    ///
    /// **Requires Nix 2.33 or later.**
    ///
    /// This builds the derivation at the given store path and returns a map of output
    /// names to their realized store paths. The derivation must already exist in the store
    /// (see [`Store::add_derivation`]).
    ///
    /// # Parameters
    /// - `path`: The store path of the derivation to build (typically ending in `.drv`)
    ///
    /// # Returns
    /// A [`BTreeMap`] mapping output names (e.g., "out", "dev", "doc") to their store paths.
    /// The map is ordered alphabetically by output name for deterministic iteration.
    #[cfg(nix_at_least = "2.33.0pre")]
    #[doc(alias = "nix_store_realise")]
    pub fn realise(&mut self, path: &StorePath) -> Result<BTreeMap<String, StorePath>> {
        let mut outputs = BTreeMap::new();
        let userdata =
            &mut outputs as *mut BTreeMap<String, StorePath> as *mut std::os::raw::c_void;

        unsafe extern "C" fn callback(
            userdata: *mut std::os::raw::c_void,
            outname: *const c_char,
            out_path: *const raw::StorePath,
        ) {
            let outputs = userdata as *mut BTreeMap<String, StorePath>;
            let outputs = &mut *outputs;

            let name = std::ffi::CStr::from_ptr(outname)
                .to_string_lossy()
                .into_owned();

            let path = raw::store_path_clone(out_path);
            let path = NonNull::new(path).expect("store_path_clone returned null");
            let path = StorePath::new_raw(path);

            outputs.insert(name, path);
        }

        unsafe {
            check_call!(raw::store_realise(
                &mut self.context,
                self.inner.ptr(),
                path.as_ptr(),
                userdata,
                Some(callback)
            ))?;
        }

        Ok(outputs)
    }

    /// Get the closure of a specific store path.
    ///
    /// **Requires Nix 2.33 or later.**
    ///
    /// Computes the filesystem closure (dependency graph) of a store path, with options
    /// to control the direction and which related paths to include.
    ///
    /// # Parameters
    /// - `store_path`: The path to compute the closure from
    /// - `flip_direction`: If false, compute the forward closure (paths referenced by this path).
    ///   If true, compute the backward closure (paths that reference this path).
    /// - `include_outputs`: When `flip_direction` is false: for any derivation in the closure, include its outputs.
    ///   When `flip_direction` is true: for any output in the closure, include derivations that produce it.
    /// - `include_derivers`: When `flip_direction` is false: for any output in the closure, include the derivation that produced it.
    ///   When `flip_direction` is true: for any derivation in the closure, include its outputs.
    ///
    /// # Returns
    /// A vector of store paths in the closure, in no particular order.
    #[cfg(nix_at_least = "2.33.0pre")]
    #[doc(alias = "nix_store_get_fs_closure")]
    pub fn get_fs_closure(
        &mut self,
        store_path: &StorePath,
        flip_direction: bool,
        include_outputs: bool,
        include_derivers: bool,
    ) -> Result<Vec<StorePath>> {
        let mut r = Vec::new();
        unsafe {
            check_call!(raw::store_get_fs_closure(
                &mut self.context,
                self.inner.ptr(),
                store_path.as_ptr(),
                flip_direction,
                include_outputs,
                include_derivers,
                callback_get_result_store_path_set_data(&mut r),
                Some(callback_get_result_store_path_set)
            ))
        }?;
        Ok(r)
    }

    pub fn weak_ref(&self) -> StoreWeak {
        StoreWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Store {
            inner: self.inner.clone(),
            context: Context::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ctor::ctor;
    use std::collections::HashMap;

    use super::*;

    #[ctor]
    fn test_setup() {
        // Initialize settings for tests
        let _ = INIT.as_ref();

        // Enable ca-derivations for all tests
        nix_bindings_util::settings::set("experimental-features", "ca-derivations").ok();

        // Disable build hooks to prevent test recursion
        nix_bindings_util::settings::set("build-hook", "").ok();

        // Set custom build dir for sandbox
        if cfg!(target_os = "linux") {
            nix_bindings_util::settings::set("sandbox-build-dir", "/custom-build-dir-for-test")
                .ok();
        }

        std::env::set_var("_NIX_TEST_NO_SANDBOX", "1");

        // Tests run offline
        nix_bindings_util::settings::set("substituters", "").ok();
    }

    #[test]
    fn none_works() {
        let res = Store::open(None, HashMap::new());
        res.unwrap();
    }

    #[test]
    fn auto_works() {
        // This is not actually a given.
        // Maybe whatever is in NIX_REMOTE or nix.conf is really important.
        let res = Store::open(Some("auto"), HashMap::new());
        res.unwrap();
    }

    #[test]
    fn invalid_uri_fails() {
        let res = Store::open(Some("invalid://uri"), HashMap::new());
        assert!(res.is_err());
    }

    #[test]
    fn get_uri() {
        let mut store = Store::open(None, HashMap::new()).unwrap();
        let uri = store.get_uri().unwrap();
        assert!(!uri.is_empty());
        // must be ascii
        assert!(uri.is_ascii());
        // usually something like "daemon", but that's not something we can check here.
        println!("uri: {}", uri);
    }

    #[test]
    #[ignore] // Needs network access
    fn get_uri_nixos_cache() {
        let mut store = Store::open(Some("https://cache.nixos.org/"), HashMap::new()).unwrap();
        let uri = store.get_uri().unwrap();
        assert_eq!(uri, "https://cache.nixos.org");
    }

    #[test]
    #[cfg(nix_at_least = "2.26" /* get_storedir */)]
    fn parse_store_path_ok() {
        let mut store = crate::store::Store::open(Some("dummy://"), []).unwrap();
        let store_dir = store.get_storedir().unwrap();
        let store_path_string =
            format!("{store_dir}/rdd4pnr4x9rqc9wgbibhngv217w2xvxl-bash-interactive-5.2p26");
        let store_path = store.parse_store_path(store_path_string.as_str()).unwrap();
        let real_store_path = store.real_path(&store_path).unwrap();
        assert_eq!(store_path.name().unwrap(), "bash-interactive-5.2p26");
        assert_eq!(real_store_path, store_path_string);
    }

    #[test]
    fn parse_store_path_fail() {
        let mut store = crate::store::Store::open(Some("dummy://"), []).unwrap();
        let store_path_string = "bash-interactive-5.2p26".to_string();
        let r = store.parse_store_path(store_path_string.as_str());
        match r {
            Err(e) => {
                assert!(e.to_string().contains("bash-interactive-5.2p26"));
            }
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn weak_ref() {
        let mut store = Store::open(None, HashMap::new()).unwrap();
        let uri = store.get_uri().unwrap();
        let weak = store.weak_ref();
        let mut store2 = weak.upgrade().unwrap();
        assert_eq!(store2.get_uri().unwrap(), uri);
    }
    #[test]
    fn weak_ref_gone() {
        let weak = {
            // Concurrent tests calling Store::open will keep the weak reference to auto alive,
            // so for this test we need to bypass the global cache.
            let store = Store::open_uncached(None, HashMap::new()).unwrap();
            store.weak_ref()
        };
        assert!(weak.upgrade().is_none());
        assert!(weak.inner.upgrade().is_none());
    }

    #[cfg(nix_at_least = "2.33.0pre")]
    fn create_temp_store() -> (Store, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();

        let store_dir = temp_dir.path().join("store");
        let state_dir = temp_dir.path().join("state");
        let log_dir = temp_dir.path().join("log");

        let store_dir_str = store_dir.to_str().unwrap();
        let state_dir_str = state_dir.to_str().unwrap();
        let log_dir_str = log_dir.to_str().unwrap();

        let params = vec![
            ("store", store_dir_str),
            ("state", state_dir_str),
            ("log", log_dir_str),
        ];

        let store = Store::open(Some("local"), params).unwrap();
        (store, temp_dir)
    }

    fn current_system() -> Result<String> {
        nix_bindings_util::settings::get("system")
    }

    #[cfg(nix_at_least = "2.33")]
    fn create_test_derivation_json() -> String {
        let system = current_system().unwrap_or_else(|_| {
            // Fallback to Rust's platform detection
            format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
        });
        format!(
            r#"{{
                "args": ["-c", "echo $name foo > $out"],
                "builder": "/bin/sh",
                "env": {{
                    "builder": "/bin/sh",
                    "name": "myname",
                    "out": "/1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9",
                    "system": "{}"
                }},
                "inputs": {{
                    "drvs": {{}},
                    "srcs": []
                }},
                "name": "myname",
                "outputs": {{
                    "out": {{
                    "hashAlgo": "sha256",
                    "method": "nar"
                    }}
                }},
                "system": "{}",
                "version": 4
            }}"#,
            system, system
        )
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn derivation_from_json() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        // If we got here, parsing succeeded
        drop(drv);
        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33.0pre")]
    fn derivation_from_invalid_json() {
        let (mut store, temp_dir) = create_temp_store();
        let result = store.derivation_from_json("not valid json");
        assert!(result.is_err());
        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn add_derivation() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Verify we got a .drv path
        let name = drv_path.name().unwrap();
        assert!(name.ends_with(".drv"));

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn realise() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Build the derivation
        let outputs = store.realise(&drv_path).unwrap();

        // Verify we got the expected output
        assert!(outputs.contains_key("out"));
        let out_path = &outputs["out"];
        let out_name = out_path.name().unwrap();
        assert_eq!(out_name, "myname");

        drop(store);
        drop(temp_dir);
    }

    #[cfg(nix_at_least = "2.33")]
    fn create_multi_output_derivation_json() -> String {
        let system = current_system()
            .unwrap_or_else(|_| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));

        format!(
            r#"{{
                "version": 4,
                "name": "multi-output-test",
                "system": "{}",
                "builder": "/bin/sh",
                "args": ["-c", "echo a > $outa; echo b > $outb; echo c > $outc; echo d > $outd; echo e > $oute; echo f > $outf; echo g > $outg; echo h > $outh; echo i > $outi; echo j > $outj"],
                "env": {{
                    "builder": "/bin/sh",
                    "name": "multi-output-test",
                    "system": "{}",
                    "outf": "/1vkfzqpwk313b51x0xjyh5s7w1lx141mr8da3dr9wqz5aqjyr2fh",
                    "outd": "/1ypxifgmbzp5sd0pzsp2f19aq68x5215260z3lcrmy5fch567lpm",
                    "outi": "/1wmasjnqi12j1mkjbxazdd0qd0ky6dh1qry12fk8qyp5kdamhbdx",
                    "oute": "/1f9r2k1s168js509qlw8a9di1qd14g5lqdj5fcz8z7wbqg11qp1f",
                    "outh": "/1rkx1hmszslk5nq9g04iyvh1h7bg8p92zw0hi4155hkjm8bpdn95",
                    "outc": "/1rj4nsf9pjjqq9jsq58a2qkwa7wgvgr09kgmk7mdyli6h1plas4w",
                    "outb": "/1p7i1dxifh86xq97m5kgb44d7566gj7rfjbw7fk9iij6ca4akx61",
                    "outg": "/14f8qi0r804vd6a6v40ckylkk1i6yl6fm243qp6asywy0km535lc",
                    "outj": "/0gkw1366qklqfqb2lw1pikgdqh3cmi3nw6f1z04an44ia863nxaz",
                    "outa": "/039akv9zfpihrkrv4pl54f3x231x362bll9afblsgfqgvx96h198"
                }},
                "inputs": {{
                    "drvs": {{}},
                    "srcs": []
                }},
                "outputs": {{
                    "outd": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outf": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outg": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outb": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outc": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outi": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outj": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outh": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "outa": {{ "hashAlgo": "sha256", "method": "nar" }},
                    "oute": {{ "hashAlgo": "sha256", "method": "nar" }}
                }}
            }}"#,
            system, system
        )
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn realise_multi_output_ordering() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_multi_output_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Build the derivation
        let outputs = store.realise(&drv_path).unwrap();

        // Verify outputs are complete (BTreeMap guarantees ordering)
        let output_names: Vec<&String> = outputs.keys().collect();
        let expected_order = vec![
            "outa", "outb", "outc", "outd", "oute", "outf", "outg", "outh", "outi", "outj",
        ];
        assert_eq!(output_names, expected_order);

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn realise_invalid_system() {
        let (mut store, temp_dir) = create_temp_store();

        // Create a derivation with an invalid system
        let system = "bogus65-bogusos";
        let drv_json = format!(
            r#"{{
                "args": ["-c", "echo $name foo > $out"],
                "builder": "/bin/sh",
                "env": {{
                    "builder": "/bin/sh",
                    "name": "myname",
                    "out": "/1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9",
                    "system": "{}"
                }},
                "inputs": {{
                    "drvs": {{}},
                    "srcs": []
                }},
                "name": "myname",
                "outputs": {{
                    "out": {{
                    "hashAlgo": "sha256",
                    "method": "nar"
                    }}
                }},
                "system": "{}",
                "version": 4
            }}"#,
            system, system
        );

        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Try to build - should fail
        let result = store.realise(&drv_path);
        let err = match result {
            Ok(_) => panic!("Build should fail with invalid system"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("required system or feature not available"),
            "Error should mention system not available, got: {}",
            err
        );

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn realise_builder_fails() {
        let (mut store, temp_dir) = create_temp_store();

        let system = current_system()
            .unwrap_or_else(|_| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));

        // Create a derivation where the builder exits with error
        let drv_json = format!(
            r#"{{
                "args": ["-c", "exit 1"],
                "builder": "/bin/sh",
                "env": {{
                    "builder": "/bin/sh",
                    "name": "failing",
                    "out": "/1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9",
                    "system": "{}"
                }},
                "inputs": {{
                    "drvs": {{}},
                    "srcs": []
                }},
                "name": "failing",
                "outputs": {{
                    "out": {{
                    "hashAlgo": "sha256",
                    "method": "nar"
                    }}
                }},
                "system": "{}",
                "version": 4
            }}"#,
            system, system
        );

        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Try to build - should fail
        let result = store.realise(&drv_path);
        let err = match result {
            Ok(_) => panic!("Build should fail when builder exits with error"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("builder failed with exit code 1"),
            "Error should mention builder failed with exit code, got: {}",
            err
        );

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn realise_builder_no_output() {
        let (mut store, temp_dir) = create_temp_store();

        let system = current_system()
            .unwrap_or_else(|_| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));

        // Create a derivation where the builder succeeds but produces no output
        let drv_json = format!(
            r#"{{
                "args": ["-c", "true"],
                "builder": "/bin/sh",
                "env": {{
                    "builder": "/bin/sh",
                    "name": "no-output",
                    "out": "/1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9",
                    "system": "{}"
                }},
                "inputs": {{
                    "drvs": {{}},
                    "srcs": []
                }},
                "name": "no-output",
                "outputs": {{
                    "out": {{
                    "hashAlgo": "sha256",
                    "method": "nar"
                    }}
                }},
                "system": "{}",
                "version": 4
            }}"#,
            system, system
        );

        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Try to build - should fail
        let result = store.realise(&drv_path);
        let err = match result {
            Ok(_) => panic!("Build should fail when builder produces no output"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("failed to produce output path"),
            "Error should mention failed to produce output, got: {}",
            err
        );

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn get_fs_closure_with_outputs() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Build the derivation to get the output path
        let outputs = store.realise(&drv_path).unwrap();
        let out_path = &outputs["out"];
        let out_path_name = out_path.name().unwrap();

        // Get closure with include_outputs=true
        let closure = store.get_fs_closure(&drv_path, false, true, false).unwrap();

        // The closure should contain at least the derivation and its output
        assert!(
            closure.len() >= 2,
            "Closure should contain at least drv and output"
        );

        // Verify the output path is in the closure
        let out_in_closure = closure.iter().any(|p| p.name().unwrap() == out_path_name);
        assert!(
            out_in_closure,
            "Output path should be in closure when include_outputs=true"
        );

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn get_fs_closure_without_outputs() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Build the derivation to get the output path
        let outputs = store.realise(&drv_path).unwrap();
        let out_path = &outputs["out"];
        let out_path_name = out_path.name().unwrap();

        // Get closure with include_outputs=false
        let closure = store
            .get_fs_closure(&drv_path, false, false, false)
            .unwrap();

        // Verify the output path is NOT in the closure
        let out_in_closure = closure.iter().any(|p| p.name().unwrap() == out_path_name);
        assert!(
            !out_in_closure,
            "Output path should not be in closure when include_outputs=false"
        );

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn get_fs_closure_flip_direction() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();

        // Build the derivation to get the output path
        let outputs = store.realise(&drv_path).unwrap();
        let out_path = &outputs["out"];
        let out_path_name = out_path.name().unwrap();

        // Get closure with flip_direction=true (reverse dependencies)
        let closure = store.get_fs_closure(&drv_path, true, true, false).unwrap();

        // Verify the output path is NOT in the closure when direction is flipped
        let out_in_closure = closure.iter().any(|p| p.name().unwrap() == out_path_name);
        assert!(
            !out_in_closure,
            "Output path should not be in closure when flip_direction=true"
        );

        drop(store);
        drop(temp_dir);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn get_fs_closure_include_derivers() {
        let (mut store, temp_dir) = create_temp_store();
        let drv_json = create_test_derivation_json();
        let drv = store.derivation_from_json(&drv_json).unwrap();
        let drv_path = store.add_derivation(&drv).unwrap();
        let drv_path_name = drv_path.name().unwrap();

        // Build the derivation to get the output path
        let outputs = store.realise(&drv_path).unwrap();
        let out_path = &outputs["out"];

        // Get closure of the output path with include_derivers=true
        let closure = store.get_fs_closure(out_path, false, false, true).unwrap();

        // Verify the derivation path is in the closure
        let drv_in_closure = closure.iter().any(|p| p.name().unwrap() == drv_path_name);
        assert!(
            drv_in_closure,
            "Derivation should be in closure when include_derivers=true"
        );

        drop(store);
        drop(temp_dir);
    }
}
