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
    let normalized = normalizer.normalize("LLEV.me").unwrap();
    assert_eq!(normalized, "llev.me");

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
