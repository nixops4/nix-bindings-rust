# Release process

This project uses simple tags, that trigger a release of all crates using Hercules CI.
Based on the [HCI Effects cargo publish workflow].

## Steps

1. Create a `release` branch
2. Decide the version bump (patch for fixes, minor for features, major for breaking changes)
3. Update `CHANGELOG.md`: make sure the Unreleased section is up to date, then change it to the new version and release date
4. Open a draft release PR and wait for CI to pass
5. Create and push a tag matching the version
6. Add a new Unreleased section to `CHANGELOG.md`
7. Bump version in all `Cargo.toml` files to the next patch version (e.g., `0.2.0` → `0.2.1`)
   and run `cargo update --workspace` to update `Cargo.lock`,
   so that `cargo publish --dry-run` passes on subsequent commits
8. Merge the release PR

---

Dissatisfied with the coarse grained release process? Complain to @roberth and he'll get it done for you.

[HCI Effects cargo publish workflow]: https://docs.hercules-ci.com/hercules-ci-effects/reference/flake-parts/cargo-publish/#_releasing_a_version
