# kvdb - Kurotsuba's (Kantan) Vector Database

A simple, learning-focused vector database implementation in Rust. **Kantan** (簡単) means "simple" in Japanese - this project focuses on understanding core vector database concepts without production complexities.

## Overview

kvdb is an educational project that implements a working vector database from scratch. It demonstrates fundamental concepts like vector normalization, similarity search, and efficient storage - perfect for learning how vector databases work under the hood.

**Current Status**: v1.0 - In-memory implementation with CLI interface

## Features

### ✅ Implemented
- **L2 Vector Normalization**: Automatic normalization on insertion
- **Cosine Similarity Search**: Using dot product on normalized vectors
- **Flat Array Storage**: Memory-efficient contiguous storage with excellent cache locality
- **REPL & CLI Modes**: Interactive shell or single-command (not meaningful before persistency) execution
- **Library-First Architecture**: Core logic separated from interface for future extensibility
- **Comprehensive Testing**: Unit tests for all core operations

### Core Operations
- `insert` - Add vectors with unique IDs
- `search` - Find k most similar vectors
- `get` - Retrieve vector by ID
- `list` - Display all stored vectors
- `count` - Show database size
- `delete` - Remove vector by ID

## Architecture

### Design Principles
**Library-First Design** for easy interface swapping:

```
src/
├── lib.rs       # Public API (VecDB)
├── vector.rs    # Vector math (L2 norm, dot product)
├── db.rs        # Core database logic
├── cli.rs       # CLI command parsing
└── main.rs      # Entry point
```

This architecture enables:
- ✅ Testing core logic independently of CLI
- ✅ Easy addition of HTTP API, gRPC, or other interfaces
- ✅ Clean dependency boundaries
- ✅ Future Lambda deployment with minimal changes

### Storage Strategy
**Flat Array Storage** for optimal performance:
- All vectors stored contiguously: `[v1_d1, v1_d2, ..., v2_d1, v2_d2, ...]`
- Parallel ID array: `["vec1", "vec2", ...]`
- Excellent memory locality for cache efficiency
- SIMD-friendly layout for future optimizations

## Installation

### Prerequisites
- Rust 1.70+ (2024 edition)
- Python 3.8+ (for benchmarking)

### Build
```bash
# Clone the repository
git clone <your-repo-url>
cd kvdb

# Build release binary
cargo build --release

# Run tests
cargo test
```

## Usage

### Interactive REPL Mode
```bash
# Start REPL
./target/release/kvdb

kvdb> insert vec1 1.0 0.0 0.0
Inserted to database with id

kvdb> insert vec2 0.0 1.0 0.0
Inserted to database with id

kvdb> search 0.7 0.7 0.0 --k_top 2
Top 2 results:
1. ID: vec2, Score: 0.7071, Vector: [0.0, 1.0, 0.0]
2. ID: vec1, Score: 0.7071, Vector: [1.0, 0.0, 0.0]

kvdb> count
2

kvdb> exit
Goodbye!
```

### Single-Command Mode (*not yet meaningful*)
```bash
# Insert a vector
./target/release/kvdb insert vec1 1.0 2.0 3.0

# Search for similar vectors
./target/release/kvdb search 1.0 2.0 3.0 --k_top 5

# Get a specific vector
./target/release/kvdb get vec1

# Show all vectors
./target/release/kvdb list

# Delete a vector
./target/release/kvdb delete vec1
```

### Library Usage
```rust
use kvdb::VecDB;

fn main() {
    let mut db = VecDB::new();

    // Insert vectors (automatically normalized)
    db.insert("doc1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
    db.insert("doc2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
    db.insert("doc3".to_string(), vec![0.7, 0.7, 0.0]).unwrap();

    // Search for similar vectors
    let results = db.search(vec![1.0, 1.0, 0.0], 2).unwrap();

    for (id, vector, score) in results {
        println!("{}: similarity = {:.4}", id, score);
    }
}
```

## Benchmarking

Run performance benchmarks:

```bash
# Install Python dependencies
pip install numpy

# Run benchmark (100k vectors, 786 dimensions, 100 searches)
python3 benchmark.py
```

Example output:
```
Database size: 100,000 vectors
Vector dimension: 786

Insertion Performance:
  Total time: 8.234 seconds
  Throughput: 12,144.23 inserts/sec
  Average per insert: 0.082 ms

Search Performance (100 random searches):
  Average per search: 45.123 ms
  Throughput: 22.16 searches/sec
```

## Performance Characteristics

### Time Complexity
- **Insert**: O(d) - where d is vector dimension
- **Search**: O(n·d) - linear scan through n vectors
- **Get**: O(n) - linear search by ID
- **Delete**: O(n·d) - find + remove from flat array

### Space Complexity
- **Storage**: O(n·d) - flat array storage
- **Memory Overhead**: Minimal (just ID strings + dimension metadata)

### Current Limitations (v1)
- **No Persistence**: In-memory only, data lost on exit
- **Linear Search**: No indexing (HNSW, IVF, etc.)
- **Single-Threaded**: No concurrent operations
- **No Quantization**: Full precision f32 storage

## Roadmap

### 🚧 TODO

**v1.1 - Persistence**
- [ ] File-based storage (MessagePack/Bincode)
- [ ] Save/load operations
- [ ] Incremental updates

**v2.0 - HTTP API**
- [ ] REST API with Axum/Actix
- [ ] JSON request/response
- [ ] API documentation (OpenAPI)
- [ ] Concurrent request handling

**v3.0 - Optimizations**
- [ ] HNSW (Hierarchical Navigable Small World) indexing
- [ ] Product Quantization for compression
- [ ] SIMD-accelerated dot product
- [ ] Parallel search with Rayon
- [ ] Memory-mapped file support


## Learning Resources

This project demonstrates several key concepts:

1. **Vector Similarity Search**: Understanding cosine similarity via dot product
2. **Memory Layout**: Cache-friendly data structures
3. **Library Design**: Separation of concerns, clean APIs
4. **Rust Patterns**: Error handling, ownership, zero-cost abstractions
5. **Testing**: Unit tests, integration tests, benchmarking


## License

MIT License - feel free to use this for learning and experimentation.

## Acknowledgments

Built as a learning project to understand:
- Vector database internals
- Rust systems programming
- Performance optimization techniques
- Library design patterns

**Note**: This is an educational implementation. For production use, consider mature solutions like [Qdrant](https://qdrant.tech/), [Milvus](https://milvus.io/), or [Weaviate](https://weaviate.io/).

---

*Kantan (簡単) - Simple, but complete enough to learn from.* 🚀