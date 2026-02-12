use kvdb::VecDB;
use std::time::Instant;
use tempfile::NamedTempFile;

fn random_vector(dim: usize, seed: u64) -> Vec<f32> {
    // Simple LCG pseudo-random generator (no external dep needed)
    let mut state = seed;
    (0..dim)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            // Map to [-1.0, 1.0]
            ((state >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

#[test]
fn test_save_load_10k_vectors_and_search() {
    let dim = 786;
    let num_vectors = 100_000;
    let num_searches = 100;

    println!("\n=== Persistence E2E Test ===");
    println!("Vectors: {}, Dimensions: {}, Searches: {}\n", num_vectors, dim, num_searches);

    // Phase 1: Create DB and insert 100K vectors
    let start = Instant::now();
    let mut db = VecDB::new();
    for i in 0..num_vectors {
        let vec = random_vector(dim, i as u64);
        db.insert(format!("vec_{}", i), vec).unwrap();
    }
    let insert_time = start.elapsed();
    assert_eq!(db.count(), num_vectors);
    println!("Phase 1 - Insert {} vectors: {:.3}s ({:.0} inserts/s)",
        num_vectors, insert_time.as_secs_f64(),
        num_vectors as f64 / insert_time.as_secs_f64());

    // Phase 2: Save to file
    let start = Instant::now();
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_str().unwrap();
    db.save(path).unwrap();
    let save_time = start.elapsed();
    let file_size = std::fs::metadata(path).unwrap().len();
    println!("Phase 2 - Save to disk: {:.3}s (file size: {:.2} MB)",
        save_time.as_secs_f64(), file_size as f64 / 1_048_576.0);

    // Phase 3: Drop current DB
    let start = Instant::now();
    drop(db);
    let drop_time = start.elapsed();
    println!("Phase 3 - Drop DB: {:.3}s", drop_time.as_secs_f64());

    // Phase 4: Load from file
    let start = Instant::now();
    let loaded_db = VecDB::load(path).unwrap();
    let load_time = start.elapsed();
    assert_eq!(loaded_db.count(), num_vectors);
    println!("Phase 4 - Load from disk: {:.3}s", load_time.as_secs_f64());

    // Phase 5: Run 100 random searches
    let start = Instant::now();
    for i in 0..num_searches {
        let query = random_vector(dim, (num_vectors + i) as u64);
        let results = loaded_db.search(query, 10).unwrap();

        assert_eq!(results.len(), 10);
        // Verify results are sorted by score descending
        for w in results.windows(2) {
            assert!(w[0].2 >= w[1].2, "Results not sorted by score");
        }
    }
    let search_time = start.elapsed();
    println!("Phase 5 - {} searches: {:.3}s (avg {:.3}ms/search)\n",
        num_searches, search_time.as_secs_f64(),
        search_time.as_secs_f64() / num_searches as f64 * 1000.0);
}