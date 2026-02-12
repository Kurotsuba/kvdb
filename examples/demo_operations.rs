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
    let dim = 768;
    let path = "demo.db";

    // Load existing demo.db
    println!("Loading '{}'...", path);
    let start = Instant::now();
    let mut db = VecDB::load(path).expect("Failed to load demo.db. Run `cargo run --release --example gen_demo` first.");
    println!("Loaded {} vectors in {:.3}s\n", db.count(), start.elapsed().as_secs_f64());

    // === Phase 1: 10 searches (before modifications) ===
    println!("=== Phase 1: 10 Searches (before modifications) ===\n");
    let search_queries: Vec<Vec<f32>> = (0..10)
        .map(|i| random_vector(dim, 900_000 + i))
        .collect();

    for (i, query) in search_queries.iter().enumerate() {
        let start = Instant::now();
        let results = db.search(query.clone(), 5).unwrap();
        let elapsed = start.elapsed();

        println!("Search {}/10 ({:.3}ms):", i + 1, elapsed.as_secs_f64() * 1000.0);
        for (rank, (id, _vec, score)) in results.iter().enumerate() {
            println!("  {}. {} (score: {:.6})", rank + 1, id, score);
        }
        println!();
    }

    // === Phase 2: 10 inserts ===
    println!("=== Phase 2: 10 Inserts ===\n");
    let count_before = db.count();
    for i in 0..10 {
        let id = format!("new_vec_{}", i);
        let vec = random_vector(dim, 800_000 + i);
        let start = Instant::now();
        let result = db.insert(id.clone(), vec).unwrap();
        let elapsed = start.elapsed();
        println!("Insert {}/10: {} - {} ({:.3}ms)", i + 1, id, result, elapsed.as_secs_f64() * 1000.0);
    }
    println!("\nCount: {} -> {}\n", count_before, db.count());

    // === Phase 3: 10 deletes ===
    println!("=== Phase 3: 10 Deletes ===\n");
    let count_before = db.count();
    for i in 0..10 {
        let id = format!("vec_{}", i * 1000); // Delete vec_0, vec_1000, vec_2000, ...
        let start = Instant::now();
        let result = db.delete(&id);
        let elapsed = start.elapsed();
        match result {
            Ok(msg) => println!("Delete {}/10: {} - {} ({:.3}ms)", i + 1, id, msg, elapsed.as_secs_f64() * 1000.0),
            Err(err) => println!("Delete {}/10: {} - Error: {} ({:.3}ms)", i + 1, id, err, elapsed.as_secs_f64() * 1000.0),
        }
    }
    println!("\nCount: {} -> {}\n", count_before, db.count());

    // === Phase 4: 10 searches (after modifications) ===
    println!("=== Phase 4: 10 Searches (after modifications) ===\n");
    // Use same queries as Phase 1 for comparison
    for (i, query) in search_queries.iter().enumerate() {
        let start = Instant::now();
        let results = db.search(query.clone(), 5).unwrap();
        let elapsed = start.elapsed();

        println!("Search {}/10 ({:.3}ms):", i + 1, elapsed.as_secs_f64() * 1000.0);
        for (rank, (id, _vec, score)) in results.iter().enumerate() {
            println!("  {}. {} (score: {:.6})", rank + 1, id, score);
        }
        println!();
    }

    println!("=== Summary ===");
    println!("Final vector count: {}", db.count());
}