use anyhow::Context as _;

use super::Derivation;

impl Derivation {
    /// Convert harmonia Derivation to nix-bindings Derivation.
    ///
    /// This requires a Store instance because the Nix C API needs it for internal validation.
    pub fn from_harmonia(
        store: &mut crate::store::Store,
        harmonia_drv: &harmonia_store_core::derivation::Derivation,
    ) -> anyhow::Result<Self> {
        let json = serde_json::to_string(harmonia_drv)
            .context("Failed to serialize harmonia Derivation to JSON")?;

        store.derivation_from_json(&json)
    }
}

impl TryFrom<&Derivation> for harmonia_store_core::derivation::Derivation {
    type Error = anyhow::Error;

    fn try_from(nix_drv: &Derivation) -> anyhow::Result<Self> {
        let json = nix_drv
            .to_json_string()
            .context("Failed to convert nix Derivation to JSON")?;

        serde_json::from_str(&json).context("Failed to parse JSON as harmonia Derivation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_harmonia_derivation() -> harmonia_store_core::derivation::Derivation {
        use harmonia_store_core::derivation::{Derivation, DerivationOutput};
        use harmonia_store_core::derived_path::OutputName;
        use harmonia_store_core::store_path::StorePath;
        use std::collections::{BTreeMap, BTreeSet};
        use std::str::FromStr;

        let system = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        let out_path = "8bs8sd27bzzy6w94fznjd2j8ldmdg7x6-myname";

        let env = BTreeMap::from([
            ("builder".into(), "/bin/sh".into()),
            ("name".into(), "myname".into()),
            ("out".into(), format!("/{out_path}").into()),
            ("system".into(), system.clone().into()),
        ]);
        let mut outputs = BTreeMap::new();
        outputs.insert(
            OutputName::from_str("out").unwrap(),
            DerivationOutput::InputAddressed(StorePath::from_base_path(out_path).unwrap()),
        );

        Derivation {
            args: vec!["-c".into(), "echo $name foo > $out".into()],
            builder: "/bin/sh".into(),
            env,
            inputs: BTreeSet::new(),
            name: b"myname".as_slice().try_into().unwrap(),
            outputs,
            platform: system.into(),
            structured_attrs: None,
        }
    }

    #[test]
    fn derivation_round_trip_harmonia() {
        let mut store = crate::store::Store::open(Some("dummy://"), []).unwrap();
        let harmonia_drv = create_harmonia_derivation();

        // Convert to nix-bindings Derivation
        let nix_drv = Derivation::from_harmonia(&mut store, &harmonia_drv).unwrap();

        // Convert back to harmonia Derivation
        let harmonia_round_trip: harmonia_store_core::derivation::Derivation =
            (&nix_drv).try_into().unwrap();

        assert_eq!(harmonia_drv, harmonia_round_trip);
    }

    #[test]
    fn derivation_clone() {
        let mut store = crate::store::Store::open(Some("dummy://"), []).unwrap();
        let harmonia_drv = create_harmonia_derivation();

        let derivation = Derivation::from_harmonia(&mut store, &harmonia_drv).unwrap();
        let cloned_derivation = derivation.clone();

        let original_harmonia: harmonia_store_core::derivation::Derivation =
            (&derivation).try_into().unwrap();
        let cloned_harmonia: harmonia_store_core::derivation::Derivation =
            (&cloned_derivation).try_into().unwrap();

        assert_eq!(original_harmonia, cloned_harmonia);
    }
}
