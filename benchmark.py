#!/usr/bin/env python3
"""
Benchmark script for KVDB vector database.

This script:
1. Generates 1000 random 100-dimensional vectors
2. Inserts them into the database
3. Performs a search with a random query vector
4. Reports timing information
"""

import subprocess
import numpy as np
import time
import sys

# Configuration
NUM_VECTORS = 100000
DIMENSION = 786
BINARY_PATH = "./target/release/kvdb"

def generate_random_vector(dim):
    """Generate a random vector with given dimension."""
    return np.random.randn(dim).astype(np.float32)

def format_vector(vec):
    """Format vector as space-separated string."""
    return ' '.join(map(str, vec))

def main():
    print("=" * 60)
    print("KVDB Benchmark")
    print("=" * 60)
    print(f"Number of vectors: {NUM_VECTORS}")
    print(f"Vector dimension: {DIMENSION}")
    print()

    # Check if binary exists
    try:
        subprocess.run([BINARY_PATH, "count"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print(f"Error: Could not find binary at {BINARY_PATH}")
        print("Please build the project first: cargo build --release")
        sys.exit(1)

    # Generate random vectors
    print("Generating random vectors...")
    vectors = []
    for i in range(NUM_VECTORS):
        vec = generate_random_vector(DIMENSION)
        vectors.append((f"vec_{i}", vec))
    print(f"Generated {len(vectors)} random vectors")
    print()

    # Phase 1: Benchmark insertions only
    print("Phase 1: Benchmarking insertions...")
    insert_commands = []
    for vec_id, vec in vectors:
        cmd = f"insert {vec_id} {format_vector(vec)}\n"
        insert_commands.append(cmd)
    insert_commands.append("exit\n")

    start_insert = time.time()
    proc = subprocess.Popen(
        [BINARY_PATH],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    stdout_insert, stderr_insert = proc.communicate(''.join(insert_commands))
    insert_time = time.time() - start_insert

    if proc.returncode != 0:
        print("Error during insertion:")
        print(stderr_insert)
        sys.exit(1)

    print(f"  Inserted {NUM_VECTORS} vectors in {insert_time:.3f} seconds")
    print(f"  Throughput: {NUM_VECTORS / insert_time:.2f} inserts/sec")
    print()

    # Phase 2: Benchmark 100 searches
    print("Phase 2: Benchmarking 100 random searches...")
    print("  Generating query vectors...")
    query_vectors = [generate_random_vector(DIMENSION) for _ in range(100)]

    # Build commands: insert all vectors + run 100 searches
    all_commands = []
    for vec_id, vec in vectors:
        all_commands.append(f"insert {vec_id} {format_vector(vec)}\n")

    for query_vec in query_vectors:
        all_commands.append(f"search {format_vector(query_vec)} --k_top 10\n")

    all_commands.append("exit\n")

    # Run and measure total time
    start_total = time.time()
    proc = subprocess.Popen(
        [BINARY_PATH],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    stdout, stderr = proc.communicate(''.join(all_commands))
    total_time = time.time() - start_total

    if proc.returncode != 0:
        print("Error during search benchmark:")
        print(stderr)
        sys.exit(1)

    # Calculate search-only time by subtracting insert time from total
    search_time = total_time - insert_time
    avg_search_time = search_time / 100  # Average over 100 searches

    print(f"  Completed 100 searches in {search_time:.3f} seconds")
    print(f"  Average search time: {avg_search_time * 1000:.3f} ms")
    print(f"  Search throughput: {100 / search_time:.2f} searches/sec")
    print()

    # Summary
    print("=" * 60)
    print("Benchmark Summary")
    print("=" * 60)
    print(f"Database size: {NUM_VECTORS:,} vectors")
    print(f"Vector dimension: {DIMENSION}")
    print()
    print("Insertion Performance:")
    print(f"  Total time: {insert_time:.3f} seconds")
    print(f"  Throughput: {NUM_VECTORS / insert_time:.2f} inserts/sec")
    print(f"  Average per insert: {insert_time / NUM_VECTORS * 1000:.3f} ms")
    print()
    print("Search Performance (100 random searches):")
    print(f"  Total time: {search_time:.3f} seconds")
    print(f"  Average per search: {avg_search_time * 1000:.3f} ms")
    print(f"  Throughput: {100 / search_time:.2f} searches/sec")
    print()
    print(f"Total benchmark time: {total_time:.3f} seconds")
    print("=" * 60)

if __name__ == "__main__":
    main()