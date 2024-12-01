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
}
