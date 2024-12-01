use ens_normalize_rs::{CurrableError, DisallowedSequence, EnsNameNormalizer, ProcessError};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};

#[fixture]
#[once]
fn normalizer() -> EnsNameNormalizer {
    EnsNameNormalizer::default()
}

#[rstest]
#[case("vitalik.eth", Ok(("vitalik.eth", "vitalik.eth")))]
#[case("VITALIK.ETH", Ok(("vitalik.eth", "vitalik.eth")))]
#[case("vitalik❤️‍🔥.eth", Ok(("vitalik❤‍🔥.eth", "vitalik❤️‍🔥.eth")))]
#[case("🅰🅱🅲", Ok(("🅰🅱🅲", "🅰️🅱️🅲")))]
#[case("-ξ1⃣", Ok(("-ξ1⃣", "-Ξ1️⃣")))]
#[case("______________vitalik", Ok(("______________vitalik", "______________vitalik")))]
#[case(
    "vitalik__",
    Err(currable_error(CurrableError::UnderscoreInMiddle, 7, "_", Some("")))
)]
#[case(
    "xx--xx",
    Err(currable_error(CurrableError::HyphenAtSecondAndThird, 2, "--", Some("")))
)]
#[case(
    "abcd.\u{303}eth",
    Err(currable_error(CurrableError::CmStart, 0, "\u{303}", Some("")))
)]
#[case(
    "vi👍\u{303}talik",
    Err(currable_error(CurrableError::CmAfterEmoji, 3, "\u{303}", Some("")))
)]
#[case(
    "・abcd",
    Err(currable_error(CurrableError::FencedLeading, 0, "・", Some("")))
)]
#[case(
    "abcd・",
    Err(currable_error(CurrableError::FencedTrailing, 4, "・", Some("")))
)]
#[case(
    "a・’a",
    Err(currable_error(CurrableError::FencedConsecutive, 1, "・’", Some("・")))
)]
#[case("vitalik .eth", Err(disallowed(" ")))]
#[case("vitalik..eth", Err(empty_label()))]
#[case("..", Err(empty_label()))]
fn e2e_tests(
    #[case] name: &str,
    #[case] expected: Result<(&str, &str), ProcessError>,
    normalizer: &EnsNameNormalizer,
) {
    let actual = normalizer.process(name);
    match expected {
        Ok((expected_normalized, expected_beautified)) => {
            let res = actual.expect("process should succeed");
            let normalized = res.normalize();
            assert_eq!(
                normalized, expected_normalized,
                "expected '{expected_normalized}', got '{normalized}'"
            );
            let beautified = res.beautify();
            assert_eq!(
                beautified, expected_beautified,
                "expected '{expected_beautified}', got '{beautified}'"
            );
        }
        Err(expected) => assert_eq!(actual.unwrap_err(), expected),
    }
}

fn disallowed(sequence: &str) -> ProcessError {
    ProcessError::DisallowedSequence(DisallowedSequence::Invalid(sequence.to_string()))
}

fn empty_label() -> ProcessError {
    ProcessError::DisallowedSequence(DisallowedSequence::EmptyLabel)
}

fn currable_error(
    inner: CurrableError,
    index: usize,
    sequence: &str,
    maybe_suggest: Option<&str>,
) -> ProcessError {
    ProcessError::CurrableError {
        inner,
        index,
        sequence: sequence.to_string(),
        maybe_suggest: maybe_suggest.map(|s| s.to_string()),
    }
}
