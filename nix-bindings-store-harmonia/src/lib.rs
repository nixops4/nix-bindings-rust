//! Harmonia integration for nix-bindings-store.
//!
//! This crate provides conversions between nix-bindings-store types and
//! harmonia-store-core types.
//!
//! **Requires Nix 2.33 or later.**

#[cfg(nix_at_least = "2.33")]
use anyhow::{Context as AnyhowContext, Result};

// Re-export the main types for convenience
#[cfg(nix_at_least = "2.33")]
pub use nix_bindings_store::derivation::Derivation;
pub use nix_bindings_store::path::{StorePath, STORE_PATH_HASH_SIZE};
#[cfg(nix_at_least = "2.33")]
pub use nix_bindings_store::store::Store;

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

// Derivation conversions - require Nix 2.33
#[cfg(nix_at_least = "2.33")]
impl TryFrom<&Derivation> for harmonia_store_core::derivation::Derivation {
    type Error = anyhow::Error;

    fn try_from(nix_drv: &Derivation) -> Result<Self> {
        let json = nix_drv
            .to_json()
            .context("Failed to convert nix Derivation to JSON")?;

        serde_json::from_str(&json).context("Failed to parse JSON as harmonia Derivation")
    }
}

#[cfg(nix_at_least = "2.33")]
impl TryFrom<Derivation> for harmonia_store_core::derivation::Derivation {
    type Error = anyhow::Error;

    fn try_from(nix_drv: Derivation) -> Result<Self> {
        (&nix_drv).try_into()
    }
}

/// Convert harmonia Derivation to nix-bindings Derivation.
///
/// This requires a Store instance because the Nix C API needs it for internal validation.
#[cfg(nix_at_least = "2.33")]
pub fn harmonia_derivation_to_nix(
    store: &mut Store,
    harmonia_drv: &harmonia_store_core::derivation::Derivation,
) -> Result<Derivation> {
    let json = serde_json::to_string(harmonia_drv)
        .context("Failed to serialize harmonia Derivation to JSON")?;

    store.derivation_from_json(&json)
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

    #[cfg(nix_at_least = "2.33")]
    fn create_test_derivation() -> harmonia_store_core::derivation::Derivation {
        use bytes::Bytes;
        use harmonia_store_core::derivation::{Derivation, DerivationOutput};
        use harmonia_store_core::derived_path::OutputName;
        use std::collections::{BTreeMap, BTreeSet};

        let platform = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        let out_path = "/1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9";

        let mut env = BTreeMap::new();
        env.insert(Bytes::from("builder"), Bytes::from("/bin/sh"));
        env.insert(Bytes::from("name"), Bytes::from("myname"));
        env.insert(Bytes::from("out"), Bytes::from(out_path));
        env.insert(Bytes::from("system"), Bytes::from(platform.clone()));

        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".parse::<OutputName>().unwrap(),
            DerivationOutput::InputAddressed(out_path.parse().unwrap()),
        );

        Derivation {
            args: vec![Bytes::from("-c"), Bytes::from("echo $name foo > $out")],
            builder: Bytes::from("/bin/sh"),
            env,
            inputs: BTreeSet::new(),
            name: "myname".parse().unwrap(),
            outputs,
            platform: Bytes::from(platform),
            structured_attrs: None,
        }
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn derivation_round_trip_harmonia() {
        let mut store = Store::open(Some("dummy://"), []).unwrap();
        let harmonia_drv = create_test_derivation();

        // Convert to nix-bindings Derivation
        let nix_drv = harmonia_derivation_to_nix(&mut store, &harmonia_drv).unwrap();

        // Convert back to harmonia Derivation
        let harmonia_round_trip: harmonia_store_core::derivation::Derivation =
            (&nix_drv).try_into().unwrap();

        assert_eq!(harmonia_drv, harmonia_round_trip);
    }

    #[test]
    #[cfg(nix_at_least = "2.33")]
    fn derivation_clone() {
        let mut store = Store::open(Some("dummy://"), []).unwrap();
        let harmonia_drv = create_test_derivation();

        let derivation = harmonia_derivation_to_nix(&mut store, &harmonia_drv).unwrap();
        let cloned_derivation = derivation.clone();

        let original_harmonia: harmonia_store_core::derivation::Derivation =
            (&derivation).try_into().unwrap();
        let cloned_harmonia: harmonia_store_core::derivation::Derivation =
            (&cloned_derivation).try_into().unwrap();

        assert_eq!(original_harmonia, cloned_harmonia);
    }
}
