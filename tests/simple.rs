use ens_normalize_rs::{normalize, DisallowedSequence, ProcessError};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn disallowed(sequence: &str) -> ProcessError {
    ProcessError::DisallowedSequence(DisallowedSequence::Invalid(sequence.to_string()))
}

fn empty_label() -> ProcessError {
    ProcessError::DisallowedSequence(DisallowedSequence::EmptyLabel)
}

#[rstest]
#[case("vitalik.eth", Ok("vitalik.eth"))]
#[case("vitalik .eth", Err(disallowed(" ")))]
#[case("vitalik..eth", Err(empty_label()))]
#[case("", Err(empty_label()))]
#[case("VITALIK.ETH", Ok("vitalik.eth"))]
fn simple(#[case] name: &str, #[case] expected: Result<&str, ProcessError>) {
    let actual = normalize(name);
    match expected {
        Ok(expected) => assert_eq!(actual.unwrap(), expected),
        Err(expected) => assert_eq!(actual.unwrap_err(), expected),
    }
}
