fn main() {
    let processor = ens_normalize_rs::Processor::default();
    let name = "🅰️🅱.eth";
    let processed = processor.process(name).unwrap();
    let beautified_name = processed.beautify().unwrap();
    let normalized_name = processed.normalized;

    assert_eq!(normalized_name, "🅰🅱.eth");
    assert_eq!(beautified_name, "🅰️🅱️.eth");
}
