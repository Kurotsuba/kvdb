# kvdb - Kurotsuba's (Kantan) Vector Database

A simple, learning-focused vector database implementation in Rust. **Kantan** (簡単) means "simple" in Japanese - this project focuses on understanding core vector database concepts without production complexities.

## Overview

kvdb is an educational project that implements a working vector database from scratch. It demonstrates fundamental concepts like vector normalization, similarity search, and persistent storage - perfect for learning how vector databases work under the hood.

**Current Status**: v2.0 - Persistent storage with bincode serialization

## Features

- **L2 Vector Normalization**: Automatic normalization on insertion
- **Cosine Similarity Search**: Using dot product on normalized vectors
- **Flat Array Storage**: Memory-efficient contiguous storage with excellent cache locality
- **Persistence**: Save/load databases to disk with bincode binary format
- **Library-First Architecture**: Core logic separated from interface for future extensibility
- **Comprehensive Testing**: Unit tests + end-to-end persistence tests

## Installation

### Prerequisites
- Rust (2024 edition)

### Build
```bash
git clone https://github.com/Kurotsuba/kvdb.git
cd kvdb

cargo build --release
cargo test
```

## Library Usage

```rust
use kvdb::VecDB;

let mut db = VecDB::new();

// Insert vectors (automatically L2-normalized)
db.insert("doc1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
db.insert("doc2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
db.insert("doc3".to_string(), vec![0.7, 0.7, 0.0]).unwrap();

// Search for k most similar vectors
let results = db.search(vec![1.0, 1.0, 0.0], 2).unwrap();
for (id, _vector, score) in results {
    println!("{}: similarity = {:.4}", id, score);
}

// Retrieve by ID
let vec = db.get("doc1").unwrap();

// Delete
db.delete("doc2").unwrap();

// Count
println!("{} vectors", db.count());

// Persist to disk
db.save("my_database.db").unwrap();

// Load from disk
let db = VecDB::load("my_database.db").unwrap();
```

## CLI / REPL

kvdb includes a command-line interface for interactive use.

### REPL Mode
```bash
./target/release/kvdb

kvdb> insert vec1 1.0 0.0 0.0
kvdb> insert vec2 0.0 1.0 0.0
kvdb> search 0.7 0.7 0.0 --k_top 2
kvdb> save my_data.db
kvdb> load my_data.db
kvdb> count
kvdb> get vec1
kvdb> delete vec1
kvdb> list
kvdb> help
kvdb> exit
```

### Single-Command Mode
```bash
# Usage: kvdb <db_path> <command> [args...]
# Automatically loads the database (or creates new), executes, and saves back.

./target/release/kvdb data.db insert vec1 1.0 2.0 3.0
./target/release/kvdb data.db search 1.0 2.0 3.0 --k_top 5
./target/release/kvdb data.db count
./target/release/kvdb data.db list
```

## Architecture

```
src/
├── lib.rs       # Public API (VecDB)
├── vector.rs    # Vector math (L2 norm, dot product)
├── db.rs        # Core database logic + persistence
├── cli.rs       # CLI parsing, REPL, command execution
└── main.rs      # Entry point
```

### Storage Strategy
- All vectors stored contiguously: `[v1_d1, v1_d2, ..., v2_d1, v2_d2, ...]`
- Parallel ID array: `["vec1", "vec2", ...]`
- Excellent memory locality for cache efficiency
- SIMD-friendly layout for future optimizations

### Persistence
- Bincode binary serialization via serde
- Buffered I/O for efficient read/write

## Performance

### Complexity
| Operation | Time | Space |
|-----------|------|-------|
| Insert | O(d) | O(d) |
| Search | O(n*d) | O(k) |
| Get | O(n) | O(d) |
| Delete | O(n*d) | O(1) |
| Save | O(n*d) | O(1) |
| Load | O(n*d) | O(n*d) |

Where n = number of vectors, d = dimension, k = top_k results.

### Benchmarks (100K vectors, 768-dim)
```
Insertion:  ~12,000 inserts/sec
Search:     ~22 searches/sec (brute-force)
Save:       ~0.3s (294 MB)
Load:       ~0.2s
```

## Examples

The `examples/` directory contains standalone scripts for demonstration:

- `gen_demo.rs` - Generate 100K random 768-d vectors and save to demo.db
- `demo_operations.rs` - Run search, insert, delete operations against demo.db
- `embed_wikipedia.rs` - Embed Wikipedia descriptions with BERT (BAAI/bge-base-en-v1.5, 768-dim) using candle
- `demo_semantic_search.rs` - Semantic search CLI over wikipedia.db
- `fetch_wikipedia.py` - Fetch 100K random Wikipedia pages (title + description)

```bash
# Generate demo database
cargo run --release --example gen_demo

# Run operations against it
cargo run --release --example demo_operations

# Semantic search (requires wikipedia.db from embed_wikipedia)
cargo run --release --example demo_semantic_search -- "famous physicist"
```

## Roadmap

### Completed
- [x] Core CRUD operations
- [x] L2 normalization + cosine similarity
- [x] REPL and CLI modes
- [x] Persistence (bincode serialization)

### TODO

**v3.0 - HTTP API**
- [ ] REST API with Axum/Actix
- [ ] JSON request/response
- [ ] Concurrent request handling

**v4.0 - Optimizations**
- [ ] HNSW indexing
- [ ] Product Quantization
- [ ] SIMD-accelerated dot product
- [ ] Parallel search with Rayon
- [ ] Memory-mapped file support

## License

MIT License - Copyright (c) 2026 Kurotsuba

See [LICENSE](LICENSE) for details.

---

*Kantan (簡単) - Simple, but complete enough to learn from.*