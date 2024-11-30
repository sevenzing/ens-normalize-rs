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
