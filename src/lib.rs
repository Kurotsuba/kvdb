//! # KVDB - A Simple Vector Database
//!
//! KVDB is a learning project implementing a simple in-memory vector database.
//! Vectors are automatically L2-normalized on insertion and searched using
//! dot product similarity (equivalent to cosine similarity for normalized vectors).
//!
//! ## Example
//!
//! ```
//! use kvdb::VecDB;
//!
//! let mut db = VecDB::new();
//!
//! // Insert vectors
//! db.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
//! db.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
//! db.insert("vec3".to_string(), vec![0.7, 0.7, 0.0]).unwrap();
//!
//! // Search for similar vectors
//! let results = db.search(vec![1.0, 0.0, 0.0], 2).unwrap();
//! assert_eq!(results[0].0, "vec1"); // Most similar vector
//! ```

pub mod vector;
mod db;

// Re-export VecDB as the primary public API
pub use db::VecDB;
