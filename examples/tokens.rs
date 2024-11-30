use ens_normalize_rs::Processor;

fn main() {
    let processor = Processor::default();

    let name = "Nàme‍🧙‍♂.eth";
    let result = processor.tokenize(name).unwrap();

    for token in result.tokens {
        if token.is_disallowed() {
            println!("disallowed: {:?}", token.as_string());
        }
    }
}
