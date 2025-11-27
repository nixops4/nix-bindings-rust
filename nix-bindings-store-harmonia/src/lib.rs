//! Harmonia integration for nix-bindings-store.
//!
//! This crate provides conversions between nix-bindings-store types and
//! harmonia-store-core types.
//!
//! **Requires Nix 2.33 or later for StorePath conversions.**

#[cfg(nix_at_least = "2.33")]
use anyhow::{Context as AnyhowContext, Result};

// Re-export the main types for convenience
pub use nix_bindings_store::path::{StorePath, STORE_PATH_HASH_SIZE};

// StorePath conversions - require Nix 2.33 for from_parts() and hash()
#[cfg(nix_at_least = "2.33")]
impl TryFrom<&harmonia_store_core::store_path::StorePath> for StorePath {
    type Error = anyhow::Error;

    fn try_from(harmonia_path: &harmonia_store_core::store_path::StorePath) -> Result<Self> {
        let hash: &[u8; STORE_PATH_HASH_SIZE] = harmonia_path.hash().as_ref();
        StorePath::from_parts(hash, harmonia_path.name().as_ref())
    }
}

#[cfg(nix_at_least = "2.33")]
impl TryFrom<harmonia_store_core::store_path::StorePath> for StorePath {
    type Error = anyhow::Error;

    fn try_from(harmonia_path: harmonia_store_core::store_path::StorePath) -> Result<Self> {
        (&harmonia_path).try_into()
    }
}

#[cfg(nix_at_least = "2.33")]
impl TryFrom<&StorePath> for harmonia_store_core::store_path::StorePath {
    type Error = anyhow::Error;

    fn try_from(nix_path: &StorePath) -> Result<Self> {
        let hash = nix_path
            .hash()
            .context("Failed to get hash from nix StorePath")?;
        let harmonia_hash = harmonia_store_core::store_path::StorePathHash::new(hash);

        let name = nix_path
            .name()
            .context("Failed to get name from nix StorePath")?;

        let harmonia_name: harmonia_store_core::store_path::StorePathName = name
            .parse()
            .context("Failed to parse name as StorePathName")?;

        Ok(harmonia_store_core::store_path::StorePath::from((
            harmonia_hash,
            harmonia_name,
        )))
    }
}

#[cfg(nix_at_least = "2.33")]
impl TryFrom<StorePath> for harmonia_store_core::store_path::StorePath {
    type Error = anyhow::Error;

    fn try_from(nix_path: StorePath) -> Result<Self> {
        (&nix_path).try_into()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn store_path_round_trip_harmonia() {
        let harmonia_path: harmonia_store_core::store_path::StorePath =
            "g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-foo.drv".parse().unwrap();

        let nix_path: StorePath = (&harmonia_path).try_into().unwrap();

        let harmonia_round_trip: harmonia_store_core::store_path::StorePath =
            (&nix_path).try_into().unwrap();

        assert_eq!(harmonia_path, harmonia_round_trip);
    }
}
