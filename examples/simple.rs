use ens_normalize_rs::Processor;

fn main() {
    let processor = Processor::default();
    let name = "🅰🅱🅲.eth";
    let processed = processor.process(name).unwrap();
    println!("normalized: {}", processed.normalized);
    println!("beautified: {}", processed.beautify().unwrap());
}
