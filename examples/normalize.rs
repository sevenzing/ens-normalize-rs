use ens_normalize_rs::Processor;

fn main() {
    let processor = Processor::default();
    for name in ["🅰🅱🅲", "⛹️‍♀"] {
        let tokens = processor.tokenize(name).unwrap();
        println!("{:?}", tokens);
        let result = processor.process(name).unwrap();
        println!("{:?}", result);
    }
}
