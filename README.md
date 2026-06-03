# ens-normalize-rs

![tests](https://github.com/sevenzing/ens-normalize-rs/actions/workflows/tests.yml/badge.svg)
![Crates.io Version](https://img.shields.io/crates/v/ens-normalize-rs)


A Rust implementation of ENS (Ethereum Name Service) name normalization.

## Description

`ens-normalize-rs` is a robust Rust library for normalizing and validating ENS names according to [ENSIP-15](https://docs.ens.domains/ensip/15) specifications. It handles Unicode normalization, validation, beautification of ENS names ensuring correct, consistent and idempotent behavior.

## Installation

```bash
cargo add ens-normalize-rs
```

Or pin a semver range in `Cargo.toml` (see [crates.io](https://crates.io/crates/ens-normalize-rs) for the latest version):

```toml
[dependencies]
ens-normalize-rs = "0.2"
```

## Usage

```rust
fn main() {
    fn main() {
    // Using normalizer to reuse preloaded data
    let normalizer = ens_normalize_rs::EnsNameNormalizer::default();
    let name = "🅰️🅱.eth";
    let processed = normalizer.process(name).unwrap();
    let beautified_name = processed.beautify();
    let normalized_name = processed.normalize();

    assert_eq!(normalized_name, "🅰🅱.eth");
    assert_eq!(beautified_name, "🅰️🅱️.eth");

    // Using normalize directly
    let normalized = normalizer.normalize("Levvv.eth").unwrap();
    assert_eq!(normalized, "levvv.eth");

    // Handling errors
    assert!(matches!(
        normalizer.normalize("Levvv..eth"),
        Err(ens_normalize_rs::ProcessError::DisallowedSequence(
            ens_normalize_rs::DisallowedSequence::EmptyLabel
        ))
    ));
    assert!(matches!(
        // U+200D ZERO WIDTH JOINER
        normalizer.normalize("Ni‍ck.ETH"),
        Err(ens_normalize_rs::ProcessError::DisallowedSequence(
            ens_normalize_rs::DisallowedSequence::InvisibleCharacter(0x200d)
        ))
    ));
}
}
```

## Testing

Crate contains several types of tests:

- Unit tests
- Integration (e2e) tests -- `tests/e2e.rs`
- [Validation ENS docs tests](https://docs.ens.domains/ensip/15#appendix-validation-tests) -- `tests/ens_tests.rs`


To run all tests simply run:

```
cargo test
```


## Releasing (maintainers)

Releases are automated with [release-plz](https://release-plz.dev/):

1. Merge PRs using [Conventional Commits](https://www.conventionalcommits.org/) (`feat:` → minor, `fix:` → patch, `feat!:` / `BREAKING:` → major).
2. release-plz opens or updates a **Release PR** (version bump in `Cargo.toml` + `CHANGELOG.md`).
3. Merge the Release PR → CI tags the release and runs `cargo publish` to [crates.io](https://crates.io/crates/ens-normalize-rs).

**One-time setup:** add a [crates.io API token](https://crates.io/settings/tokens) with publish permission for `ens-normalize-rs` as the GitHub Actions secret `CARGO_REGISTRY_TOKEN` (repo → Settings → Secrets and variables → Actions).

## Roadmap


- [x] Tokenization
- [x] Normalization
- [x] Beautification
- [x] ENSIP-15 Validation Tests
- [ ] Unicode Normalization Tests
- [ ] CLI to update `specs.json` and `nf.json`
- [ ] analog of [ens_cure](https://github.com/namehash/ens-normalize-python?tab=readme-ov-file#ens_cure) function
- [ ] analog of [ens_normalizations](https://github.com/namehash/ens-normalize-python/tree/main?tab=readme-ov-file#ens_normalizations) function


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
