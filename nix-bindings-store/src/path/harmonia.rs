use anyhow::{Context as _, Result};

use super::{StorePath, STORE_PATH_HASH_SIZE};

impl TryFrom<&harmonia_store_core::store_path::StorePath> for StorePath {
    type Error = anyhow::Error;

    fn try_from(harmonia_path: &harmonia_store_core::store_path::StorePath) -> Result<Self> {
        let hash: &[u8; STORE_PATH_HASH_SIZE] = harmonia_path.hash().as_ref();
        StorePath::from_parts(hash, harmonia_path.name().as_ref())
    }
}

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

#[cfg(test)]
mod tests {

    #[test]
    fn store_path_round_trip_harmonia() {
        let harmonia_path: harmonia_store_core::store_path::StorePath =
            "g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-foo.drv".parse().unwrap();

        let nix_path: crate::path::StorePath = (&harmonia_path).try_into().unwrap();

        let harmonia_round_trip: harmonia_store_core::store_path::StorePath =
            (&nix_path).try_into().unwrap();

        assert_eq!(harmonia_path, harmonia_round_trip);
    }

    #[test]
    fn store_path_harmonia_clone() {
        let harmonia_path: harmonia_store_core::store_path::StorePath =
            "g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-foo.drv".parse().unwrap();

        let nix_path: crate::path::StorePath = (&harmonia_path).try_into().unwrap();
        let cloned_path = nix_path.clone();

        assert_eq!(nix_path.name().unwrap(), cloned_path.name().unwrap());
        assert_eq!(nix_path.hash().unwrap(), cloned_path.hash().unwrap());
    }
}
