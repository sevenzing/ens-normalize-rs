const SIZE: usize = 100;
const NAME_LENGTH: usize = 1000;
const NAME: &str = "$Sand-#️⃣🇪🇨";

fn main() {
    let now = std::time::Instant::now();
    let name = std::iter::repeat(NAME)
        .take(NAME_LENGTH / NAME.len())
        .collect::<Vec<_>>()
        .join("");
    let processor = ens_normalize_rs::Processor::default();
    for _ in 0..SIZE {
        let _name = processor.process(&name).unwrap();
    }
    // Total time to process 100 names: 728.916542ms
    println!("Total time to process {SIZE} names: {:?}", now.elapsed());
}