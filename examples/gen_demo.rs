use kvdb::VecDB;
use std::time::Instant;

fn random_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..dim)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((state >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

fn main() {
    let num_vectors = 100_000;
    let dim = 768;
    let path = "demo.db";

    println!("Generating {} random {}-d vectors...", num_vectors, dim);

    let start = Instant::now();
    let mut db = VecDB::new();
    for i in 0..num_vectors {
        let vec = random_vector(dim, i as u64);
        db.insert(format!("vec_{}", i), vec).unwrap();
        if (i + 1) % 10_000 == 0 {
            println!("  inserted {}/{}", i + 1, num_vectors);
        }
    }
    let insert_time = start.elapsed();
    println!("Insert: {:.3}s ({:.0} inserts/s)\n",
        insert_time.as_secs_f64(),
        num_vectors as f64 / insert_time.as_secs_f64());

    println!("Saving to '{}'...", path);
    let start = Instant::now();
    db.save(path).unwrap();
    let save_time = start.elapsed();
    let file_size = std::fs::metadata(path).unwrap().len();
    println!("Save: {:.3}s (file size: {:.2} MB)",
        save_time.as_secs_f64(),
        file_size as f64 / 1_048_576.0);

    println!("\nDone! Load it with: kvdb> load demo.db");
}