//! Nix version parsing and conditional compilation support.

/// Emit [`cargo:rustc-cfg`] directives for Nix version-based conditional compilation.
///
/// Call from build.rs with the Nix version and desired version gates.
///
/// [`cargo:rustc-cfg`]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-cfg
///
/// # Example
///
/// ```
/// use nix_bindings_util::nix_version::emit_version_cfg;
/// # // Stub pkg_config so that we can render a full usage example
/// # mod pkg_config { pub fn probe_library(_: &str) -> Result<Library, ()> { Ok(Library { version: "2.33.0pre".into() }) }
/// #   pub struct Library { pub version: String } }
///
/// let nix_version = pkg_config::probe_library("nix-store-c").unwrap().version;
/// emit_version_cfg(&nix_version, &["2.26", "2.33.0pre", "2.33"]);
/// ```
///
/// Emits `nix_at_least="2.26"` and `nix_at_least="2.33.0pre"` for version 2.33.0pre,
/// usable as `#[cfg(nix_at_least = "2.26")]`.
pub fn emit_version_cfg(nix_version: &str, relevant_versions: &[&str]) {
    // Declare the known versions for cargo check-cfg
    let versions = relevant_versions
        .iter()
        .map(|v| format!("\"{}\"", v))
        .collect::<Vec<_>>()
        .join(",");

    println!(
        "cargo:rustc-check-cfg=cfg(nix_at_least,values({}))",
        versions
    );

    let nix_version = parse_version(nix_version);

    for version_str in relevant_versions {
        let version = parse_version(version_str);
        if nix_version >= version {
            println!("cargo:rustc-cfg=nix_at_least=\"{}\"", version_str);
        }
    }
}

/// Parse a Nix version string into a comparable tuple `(major, minor, patch)`.
///
/// Pre-release versions (containing `"pre"`) get patch = -1, sorting before stable releases.
/// Omitted patch defaults to 0.
///
/// # Examples
///
/// ```
/// use nix_bindings_util::nix_version::parse_version;
///
/// assert_eq!(parse_version("2.26"), (2, 26, 0));
/// assert_eq!(parse_version("2.33.0pre"), (2, 33, -1));
/// assert_eq!(parse_version("2.33"), (2, 33, 0));
/// assert_eq!(parse_version("2.33.1"), (2, 33, 1));
///
/// // Pre-release versions sort before stable
/// assert!(parse_version("2.33.0pre") < parse_version("2.33"));
/// ```
pub fn parse_version(version_str: &str) -> (u32, u32, i32) {
    let parts = version_str.split('.').collect::<Vec<&str>>();
    let major = parts[0].parse::<u32>().unwrap();
    let minor = parts[1].parse::<u32>().unwrap();
    let patch = if parts.get(2).map_or(false, |s| s.contains("pre")) {
        -1i32
    } else {
        parts
            .get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
    };
    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("2.26"), (2, 26, 0));
        assert_eq!(parse_version("2.33.0pre"), (2, 33, -1));
        assert_eq!(parse_version("2.33"), (2, 33, 0));
        assert_eq!(parse_version("2.33.1"), (2, 33, 1));
    }

    #[test]
    fn test_version_ordering() {
        // Pre-release versions should sort before stable
        assert!(parse_version("2.33.0pre") < parse_version("2.33"));
        assert!(parse_version("2.33.0pre") < parse_version("2.33.0"));

        // Normal version ordering
        assert!(parse_version("2.26") < parse_version("2.33"));
        assert!(parse_version("2.33") < parse_version("2.33.1"));
    }
}
