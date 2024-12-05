use ens_normalize_rs::EnsNameNormalizer;

fn main() {
    let normalizer = EnsNameNormalizer::default();

    let name = "Nàme‍🧙‍♂.eth";
    let result = normalizer.tokenize(name);

    for token in result.tokens {
        if token.is_disallowed() {
            println!("disallowed: {:?}", token.as_string());
        }
    }
}
