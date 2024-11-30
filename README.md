# ens-normalize-rs

![tests](https://github.com/sevenzing/ens-normalize-rs/actions/workflows/tests.yml/badge.svg)

A Rust implementation of ENS (Ethereum Name Service) name normalization.

## Description

`ens-normalize-rs` is a robust Rust library for normalizing and validating ENS names according to [ENSIP-15](https://docs.ens.domains/ensip/15) specifications. It handles Unicode normalization, validation, beautification of ENS names ensuring correct, consistent and idempotent behavior.

## Installation

```bash
cargo add ens-normalize-rs
```

Or add this to your project using Cargo:

```toml
[dependencies]
ens-normalize-rs = "0.1.0"
```

## Usage

```rust
fn main() {
    // Using processor to reuse preloaded data
    let processor = ens_normalize_rs::Processor::default();
    let name = "🅰️🅱.eth";
    let processed = processor.process(name).unwrap();
    let beautified_name = processed.beautify();
    let normalized_name = processed.normalize();

    assert_eq!(normalized_name, "🅰🅱.eth");
    assert_eq!(beautified_name, "🅰️🅱️.eth");

    // Using process directly
    let processed = ens_normalize_rs::process("Levvv.eth").unwrap();
    assert_eq!(processed.normalize(), "levvv.eth");
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


## Roadmap


- [x] Tokenization
- [x] Normalization
- [x] Beautification
- [x] ENSIP-15 Validation Tests
- [ ] Unicode Normalization Tests
- [ ] CLI to update `specs.json` and `nf.json`
- [ ] [ens_cure](https://github.com/namehash/ens-normalize-python?tab=readme-ov-file#ens_cure) analog function


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
