# Release process

This project uses simple tags, that trigger a release of all crates using Hercules CI.
See [HCI Effects cargo publish workflow].

## Release branch

Create a `release` branch and PR for release preparation. This allows CI to validate the release before tagging.

## Before tagging

- Update `CHANGELOG.md`:
  - Make sure the Unreleased section is up to date
  - Change it to the new version and release date

## After tagging

- Add a new Unreleased section to `CHANGELOG.md`
- Merge the release PR

---

Dissatisfied with the coarse grained release process? Complain to @roberth and he'll get it done for you.

[HCI Effects cargo publish workflow]: https://docs.hercules-ci.com/hercules-ci-effects/reference/flake-parts/cargo-publish/#_releasing_a_version
