use std::time::Instant;

fn main() {
    let now = Instant::now();
    let name = std::iter::repeat("$Sand-#️⃣🇪🇨")
        .take(10)
        .collect::<Vec<_>>()
        .join("");
    let processor = ens_normalize_rs::Processor::default();
    let size = 1000;
    for _ in 0..size {
        let _name = processor.process(&name).unwrap();
    }
    // Total time to process 1000 names: 4.755187667s
    println!("Total time to process {size} names: {:?}", now.elapsed());
}
