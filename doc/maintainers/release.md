# Release process

This project uses simple tags, that trigger a release of all crates using Hercules CI.
Based on the [HCI Effects cargo publish workflow].

## Steps

1. Create a `release` branch

2. Decide the version bump. Since the project is pre-1.0, semver `0.x.y` semantics apply:
   - **patch** (`y`): fixes and new features (non-breaking)
   - **minor** (`x`): breaking changes

   Note: the post-release version bump (step 7) already sets `Cargo.toml` to the next patch version, so a non-breaking release typically requires no version change.

3. Update `CHANGELOG.md`: make sure the Unreleased section is up to date, then change it to the new version and release date

4. Open a draft release PR and wait for CI to pass

5. Create and push a tag matching the version

6. Add a new Unreleased section to `CHANGELOG.md`

7. Bump version in all `Cargo.toml` files to the next patch version (e.g., `0.2.0` → `0.2.1`)
   and run `cargo update --workspace` to update `Cargo.lock`,
   so that `cargo publish --dry-run` passes on subsequent commits

8. Merge the release PR

9. Manually update gh-pages to include the tag in the list

---

Dissatisfied with the coarse grained release process? Complain to @roberth and he'll get it done for you.

[HCI Effects cargo publish workflow]: https://docs.hercules-ci.com/hercules-ci-effects/reference/flake-parts/cargo-publish/#_releasing_a_version
