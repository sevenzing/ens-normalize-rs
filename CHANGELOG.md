# Changelog

## [0.2.0](https://github.com/sevenzing/ens-normalize-rs/compare/v0.1.1...v0.2.0) - 2026-06-03

### Added

- Unicode 17.0 normalization data and conformance tests ([#233](https://github.com/sevenzing/ens-normalize-rs/pull/233))
- Automated crates.io releases via [release-plz](https://release-plz.dev/)
- CI workflow to keep `nf.json`, `spec.json`, and `tests.json` in sync with [ens-normalize.js](https://github.com/adraffy/ens-normalize.js)

### Changed

- **Breaking:** Unicode 17 `spec.json` schema (`wholes`, tuple `mapped`/`fenced`); emoji matching uses FE0F-aware trie; valid codepoint set expanded with Unicode NFD
- **Breaking:** removed public types `ParsedWholeValue`, `ParsedWholeObject`; `ParsedWhole` and related internals are not a stable API

### Fixed

- ENS validation tests after aligning `tests.json` with Unicode 17
- Unicode updater no longer commits `.backup` files; only changed data files are replaced

## [0.1.1] - 2024-01-01

### Added

- Initial published crate
