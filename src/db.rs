//! The database module
//! Provide CRUD method for the vector database

use crate::vector::{dot_product, l2_norm};
use serde::{Serialize, Deserialize};
use std:: { 
    fs::File,
    io::{
        BufReader,
        BufWriter,
    }
};

#[derive(Serialize, Deserialize)]
pub struct VecDB {
    ids: Vec<String>,
    vectors: Vec<f32>,
    dimension: Option<usize>,
}

impl VecDB {
    /// Creates a new empty vector database instance.
    ///
    /// The database starts with no dimension constraint. The dimension will be
    /// set automatically on the first insert operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use kvdb::VecDB;
    ///
    /// let db = VecDB::new();
    /// assert_eq!(db.count(), 0);
    /// ```
    pub fn new() -> VecDB {
        VecDB { ids: Vec::new(), vectors: Vec::new(), dimension: None }
    }

    /// Inserts or updates a vector in the database.
    ///
    /// The vector is automatically L2-normalized before storage. If the ID already
    /// exists, the existing vector is updated. If the database is empty, the
    /// dimension is set based on the first vector inserted.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the vector
    /// * `vector` - Vector to insert (will be normalized)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Success message indicating insertion or update
    /// * `Err(String)` - Error if dimension mismatch or normalization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use kvdb::VecDB;
    ///
    /// let mut db = VecDB::new();
    ///
    /// // Insert a new vector
    /// let result = db.insert("vec1".to_string(), vec![3.0, 4.0]);
    /// assert!(result.is_ok());
    ///
    /// // Update existing vector
    /// let result = db.insert("vec1".to_string(), vec![1.0, 0.0]);
    /// assert!(result.unwrap().contains("Updated"));
    ///
    /// // Dimension mismatch error
    /// let result = db.insert("vec2".to_string(), vec![1.0, 2.0, 3.0]);
    /// assert!(result.is_err());
    /// ```
    pub fn insert(&mut self, id: String, vector: Vec<f32>) -> Result<String, String> {
        let dim = vector.len();
        match self.dimension {
            None => {
                self.dimension = Some(dim);
            }
            Some(d) => {
                if dim != d {
                    return Err("Different dimension".to_string());
                } 
            }
        }

        
        let norm_vec = l2_norm(&vector);
        match norm_vec {
            Ok(res) => {
                // Check if ID exists and update instead
                if let Some(index) = self.ids.iter().position(|x| x == &id) {
                    // Update existing vector
                    let start = index * dim;
                    self.vectors.splice(start..start+dim, res.iter().cloned());
                    return Ok(format!("Updated vector with id: {}", id));
                }
                self.ids.push(id);
                self.vectors.extend(res);
            }
            Err(msg) => return Err(msg),
        }

        Ok("Inserted to database with id".to_string())
    }

    /// Searches for the k most similar vectors to the query vector.
    ///
    /// The query vector is normalized and compared against all stored vectors using
    /// dot product similarity (equivalent to cosine similarity for normalized vectors).
    /// Results are returned in descending order of similarity.
    ///
    /// # Arguments
    ///
    /// * `query` - Query vector (will be normalized)
    /// * `top_k` - Number of results to return
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(String, Vec<f32>, f32)>)` - Vector of tuples containing:
    ///   - ID of the vector
    ///   - The normalized vector
    ///   - Similarity score (0.0 to 1.0)
    /// * `Err(String)` - Error if database is empty, dimension mismatch, or normalization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use kvdb::VecDB;
    ///
    /// let mut db = VecDB::new();
    /// db.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
    /// db.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
    /// db.insert("vec3".to_string(), vec![0.7, 0.7, 0.0]).unwrap();
    ///
    /// // Search for vectors similar to [1.0, 0.0, 0.0]
    /// let results = db.search(vec![1.0, 0.0, 0.0], 2).unwrap();
    /// assert_eq!(results.len(), 2);
    /// assert_eq!(results[0].0, "vec1"); // Most similar
    /// assert!((results[0].2 - 1.0).abs() < 0.01); // Similarity ~1.0
    /// ```
    pub fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<(String, Vec<f32>, f32)>, String> {
        if self.dimension.is_none() {
            return Err("Empty database".to_string());
        } else if query.len() != self.dimension.unwrap() {
            return Err("Wrong query dimension".to_string());
        }

        let norm_q = match l2_norm(&query) {
            Ok(res) => res,
            Err(msg) => {
                return Err(msg)
            },
        };

        if top_k >= self.ids.len(){
            let mut remain = Vec::new();
            for i in 0..self.ids.len() {
                remain.push(self.get_vector(i));
            }

            let result = self.ids.iter()
                .zip(remain.iter())
                .map(|(i, v)| (i.clone(), v.to_vec(), dot_product(v, &norm_q).unwrap()))
                .collect();

            return Ok(result);
        }

        let mut dps: Vec<(usize, f32)> = vec![(top_k-1, f32::NEG_INFINITY); top_k];
        for i in 0..self.ids.len() {
            let sim = dot_product(self.get_vector(i), &norm_q).unwrap();
            let insert_index = dps.partition_point(|&x| x.1 > sim);
            dps.insert(insert_index, (i, sim));
            dps.truncate(top_k);
        }

        let result = dps.iter()
            .map(|(i, dp)| (self.ids[*i].clone(), self.get_vector(*i).to_vec(), *dp))
            .collect();

        Ok(result)


    }

    /// Retrieves a vector by its ID.
    ///
    /// Returns the normalized vector associated with the given ID, or `None`
    /// if the ID doesn't exist or the database is empty.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the vector to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(Vec<f32>)` - The normalized vector if found
    /// * `None` - If the ID doesn't exist or database is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use kvdb::VecDB;
    ///
    /// let mut db = VecDB::new();
    /// db.insert("vec1".to_string(), vec![3.0, 4.0]).unwrap();
    ///
    /// // Get existing vector
    /// let vec = db.get("vec1");
    /// assert!(vec.is_some());
    /// assert_eq!(vec.unwrap().len(), 2);
    ///
    /// // Try to get non-existent vector
    /// let vec = db.get("vec2");
    /// assert!(vec.is_none());
    /// ```
    pub fn get(&self, id: &str) -> Option<Vec<f32>> {
        if self.dimension.is_none() {
            return None;
        }

        for i in 0..self.ids.len() {
            if self.ids[i] == id {
                return Some(self.get_vector(i).to_vec());
            }
        }        

        None
    }

    /// Deletes a vector from the database by its ID.
    ///
    /// Removes both the ID and the associated vector data from the flat array storage.
    /// After deletion, the remaining vectors maintain their correct indices.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the vector to delete
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Success message if deletion succeeded
    /// * `Err(String)` - Error if database is empty or ID not found
    ///
    /// # Examples
    ///
    /// ```
    /// use kvdb::VecDB;
    ///
    /// let mut db = VecDB::new();
    /// db.insert("vec1".to_string(), vec![1.0, 2.0]).unwrap();
    /// db.insert("vec2".to_string(), vec![3.0, 4.0]).unwrap();
    ///
    /// // Delete existing vector
    /// let result = db.delete("vec1");
    /// assert!(result.is_ok());
    /// assert!(db.get("vec1").is_none());
    ///
    /// // Try to delete non-existent vector
    /// let result = db.delete("vec3");
    /// assert!(result.is_err());
    /// ```
    pub fn delete(&mut self, id: &str) -> Result<String, String> {
        if self.dimension.is_none() {
            return Err("Cannot delete on empty database".to_string());
        }

        for i in 0..self.ids.len() {
            if self.ids[i] == id {
                self.vectors.splice(
                    (i * self.dimension.unwrap())..((i+1) * self.dimension.unwrap()),
                    std::iter::empty()
                );
                self.ids.remove(i);
                return Ok("Success Delete".to_string());
            }
        }

        Err("ID not found".to_string())
    }

    /// Returns all vectors in the database with their IDs.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (ID, normalized vector)
    ///
    /// # Examples
    ///
    /// ```
    /// use kvdb::VecDB;
    ///
    /// let mut db = VecDB::new();
    /// db.insert("vec1".to_string(), vec![1.0, 0.0]).unwrap();
    /// db.insert("vec2".to_string(), vec![0.0, 1.0]).unwrap();
    ///
    /// let all_vectors = db.list();
    /// assert_eq!(all_vectors.len(), 2);
    /// ```
    pub fn list(&self) -> Vec<(String, Vec<f32>)> {
        (0..self.ids.len())
            .map(|i| (self.ids[i].clone(), self.get_vector(i).to_vec()))
            .collect()
    }

    /// Returns the number of vectors in the database.
    pub fn count(&self) -> usize {
        self.ids.len()
    }

    /// Retrieves a vector slice from the flat array by index.
    ///
    /// This is a private helper function that efficiently slices the flat vector
    /// array to return a reference to the vector at the given index. The vectors
    /// are stored contiguously as: `[v1_d1, v1_d2, ..., v2_d1, v2_d2, ...]`
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the vector (0-based)
    ///
    /// # Returns
    ///
    /// A slice reference to the vector at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if the dimension is `None` or if the index is out of bounds.
    fn get_vector(&self, index: usize) -> &[f32] {
        let start = index * self.dimension.unwrap();
        &self.vectors[start..start+self.dimension.unwrap()]
    }

    /// Saves the database to a file using bincode serialization.
    ///
    /// All vectors, IDs, and dimension metadata are serialized into a compact
    /// binary format and written to disk using buffered I/O.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to save the database to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Database saved successfully
    /// * `Err(String)` - Error if file creation or serialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kvdb::VecDB;
    ///
    /// let mut db = VecDB::new();
    /// db.insert("vec1".to_string(), vec![1.0, 2.0, 3.0]).unwrap();
    /// db.save("my_database.db").unwrap();
    /// ```
    pub fn save(&self, path: &str) -> Result<(), String> {
        let file = File::create(path)
            .map_err(|e| format!("Fail to create file for saving '{}': {}", path, e))?;

        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)
            .map_err(|e| format!("Serialization failed: {}", e))?;
    
        Ok(())
    }

    /// Loads a database from a file previously saved with [`save`](VecDB::save).
    ///
    /// Deserializes the binary file back into a fully functional `VecDB` instance
    /// with all vectors, IDs, and dimension metadata restored.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to load the database from
    ///
    /// # Returns
    ///
    /// * `Ok(VecDB)` - The loaded database
    /// * `Err(String)` - Error if file not found, cannot be opened, or deserialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kvdb::VecDB;
    ///
    /// let db = VecDB::load("my_database.db").unwrap();
    /// println!("Loaded {} vectors", db.count());
    /// ```
    pub fn load(path: &str) -> Result<Self, String> {
        if !std::path::Path::new(path).exists() {
            return Err("File not found!".to_string());
        }

        let file = File::open(path)
            .map_err(|e| format!("Fail to create file for saving '{}': {}", path, e))?;

        let reader = BufReader::new(file);

        let db: VecDB = bincode::deserialize_from(reader)
            .map_err(|e| format!("Deserialization failed: {}", e))?;

        Ok(db)
    
    }
}

#[cfg(test)]
mod db_test {
    use super::*;

    #[test]
    fn test_insert_single_vector() {
        let mut db = VecDB::new();
        let result = db.insert("vec1".to_string(), vec![1.0, 2.0, 3.0]);

        assert!(result.is_ok());
        assert_eq!(db.ids.len(), 1);
        assert_eq!(db.ids[0], "vec1");
        assert_eq!(db.dimension, Some(3));
        // Vectors should be normalized, so length should be 3 floats
        assert_eq!(db.vectors.len(), 3);
    }

    #[test]
    fn test_insert_multiple_vectors() {
        let mut db = VecDB::new();

        db.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        db.insert("vec3".to_string(), vec![0.0, 0.0, 1.0]).unwrap();

        assert_eq!(db.ids.len(), 3);
        assert_eq!(db.vectors.len(), 9); // 3 vectors × 3 dimensions
    }

    #[test]
    fn test_insert_dimension_mismatch() {
        let mut db = VecDB::new();

        db.insert("vec1".to_string(), vec![1.0, 2.0, 3.0]).unwrap();
        let result = db.insert("vec2".to_string(), vec![1.0, 2.0]); // Wrong dimension

        assert!(result.is_err());
        assert_eq!(db.ids.len(), 1); // Only first vector inserted
    }

    #[test]
    fn test_get_vector() {
        let mut db = VecDB::new();

        db.insert("vec1".to_string(), vec![3.0, 4.0]).unwrap();
        db.insert("vec2".to_string(), vec![5.0, 12.0]).unwrap();

        // Get first vector (normalized [3,4] = [0.6, 0.8])
        let v1 = db.get_vector(0);
        assert_eq!(v1.len(), 2);
        assert!((v1[0] - 0.6).abs() < 1e-5);
        assert!((v1[1] - 0.8).abs() < 1e-5);

        // Get second vector (normalized [5,12] = [~0.38, ~0.92])
        let v2 = db.get_vector(1);
        assert_eq!(v2.len(), 2);
        assert!((v2[0] - 0.384615).abs() < 1e-5);
        assert!((v2[1] - 0.923077).abs() < 1e-5);
    }

    #[test]
    fn test_search_basic() {
        let mut db = VecDB::new();

        // Insert three vectors
        db.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        db.insert("vec3".to_string(), vec![0.7, 0.7, 0.0]).unwrap();

        // Search for vector close to vec1
        let results = db.search(vec![1.0, 0.0, 0.0], 2).unwrap();

        assert_eq!(results.len(), 2);
        // First result should be vec1 (exact match, similarity = 1.0)
        assert_eq!(results[0].0, "vec1");
        assert!((results[0].2 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_search_returns_top_k() {
        let mut db = VecDB::new();

        db.insert("vec1".to_string(), vec![1.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0]).unwrap();
        db.insert("vec3".to_string(), vec![0.5, 0.5]).unwrap();

        // Request top 2 out of 3
        let results = db.search(vec![1.0, 1.0], 2).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_empty_database() {
        let db = VecDB::new();

        let result = db.search(vec![1.0, 2.0], 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_dimension_mismatch() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0, 3.0]).unwrap();

        let result = db.search(vec![1.0, 2.0], 1); // Wrong dimension
        assert!(result.is_err());
    }

    // ========== Get Tests ==========

    #[test]
    fn test_get_existing_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![3.0, 4.0]).unwrap();

        let result = db.get("vec1");
        assert!(result.is_some());

        let vec = result.unwrap();
        assert_eq!(vec.len(), 2);
        // Should be normalized [3,4] -> [0.6, 0.8]
        assert!((vec[0] - 0.6).abs() < 1e-5);
        assert!((vec[1] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_get_nonexistent_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0]).unwrap();

        let result = db.get("vec2");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_from_empty_database() {
        let db = VecDB::new();

        let result = db.get("vec1");
        assert!(result.is_none());
    }


    // ========== Delete Tests ==========

    #[test]
    fn test_delete_existing_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0]).unwrap();
        db.insert("vec2".to_string(), vec![3.0, 4.0]).unwrap();

        let result = db.delete("vec1");
        assert!(result.is_ok());

        // Verify vec1 is gone
        assert!(db.get("vec1").is_none());

        // Verify vec2 is still there
        assert!(db.get("vec2").is_some());

        // Verify database size
        assert_eq!(db.ids.len(), 1);
        assert_eq!(db.vectors.len(), 2); // 1 vector × 2 dimensions
    }

    #[test]
    fn test_delete_nonexistent_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0]).unwrap();

        let result = db.delete("vec2");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "ID not found");

        // Original data should be intact
        assert_eq!(db.ids.len(), 1);
    }

    #[test]
    fn test_delete_from_empty_database() {
        let mut db = VecDB::new();

        let result = db.delete("vec1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot delete on empty database");
    }

    #[test]
    fn test_delete_first_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        db.insert("vec3".to_string(), vec![0.0, 0.0, 1.0]).unwrap();

        db.delete("vec1").unwrap();

        // Verify vec1 is gone
        assert!(db.get("vec1").is_none());

        // Verify vec2 and vec3 are still accessible
        let v2 = db.get("vec2").unwrap();
        assert!((v2[1] - 1.0).abs() < 1e-5);

        let v3 = db.get("vec3").unwrap();
        assert!((v3[2] - 1.0).abs() < 1e-5);

        // Verify counts
        assert_eq!(db.ids.len(), 2);
        assert_eq!(db.vectors.len(), 6); // 2 vectors × 3 dimensions
    }

    #[test]
    fn test_delete_middle_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0]).unwrap();
        db.insert("vec3".to_string(), vec![1.0, 1.0]).unwrap();

        db.delete("vec2").unwrap();

        // Verify vec2 is gone
        assert!(db.get("vec2").is_none());

        // Verify vec1 and vec3 are still correct
        assert!(db.get("vec1").is_some());
        assert!(db.get("vec3").is_some());

        assert_eq!(db.ids.len(), 2);
        assert_eq!(db.vectors.len(), 4); // 2 vectors × 2 dimensions
    }

    #[test]
    fn test_delete_last_vector() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0, 3.0]).unwrap();
        db.insert("vec2".to_string(), vec![4.0, 5.0, 6.0]).unwrap();
        db.insert("vec3".to_string(), vec![7.0, 8.0, 9.0]).unwrap();

        db.delete("vec3").unwrap();

        // Verify vec3 is gone
        assert!(db.get("vec3").is_none());

        // Verify vec1 and vec2 are still there
        assert!(db.get("vec1").is_some());
        assert!(db.get("vec2").is_some());

        assert_eq!(db.ids.len(), 2);
        assert_eq!(db.vectors.len(), 6); // 2 vectors × 3 dimensions
    }

    #[test]
    fn test_delete_all_vectors_sequentially() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0]).unwrap();
        db.insert("vec2".to_string(), vec![3.0, 4.0]).unwrap();

        db.delete("vec1").unwrap();
        assert_eq!(db.ids.len(), 1);

        db.delete("vec2").unwrap();
        assert_eq!(db.ids.len(), 0);
        assert_eq!(db.vectors.len(), 0);
    }

    #[test]
    fn test_insert_after_delete() {
        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 2.0]).unwrap();
        db.delete("vec1").unwrap();

        // Should be able to insert again with same ID
        let result = db.insert("vec1".to_string(), vec![3.0, 4.0]);
        assert!(result.is_ok());

        let vec = db.get("vec1").unwrap();
        // Normalized [3,4] = [0.6, 0.8]
        assert!((vec[0] - 0.6).abs() < 1e-5);
    }

    // ========== Save/Load Tests ==========

    #[test]
    fn test_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let path_str = path.to_str().unwrap();

        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        db.insert("vec3".to_string(), vec![0.0, 0.0, 1.0]).unwrap();

        db.save(path_str).unwrap();

        let loaded = VecDB::load(path_str).unwrap();
        assert_eq!(loaded.count(), 3);
        assert_eq!(loaded.dimension, Some(3));

        // Verify vectors are preserved
        let v1 = loaded.get("vec1").unwrap();
        assert!((v1[0] - 1.0).abs() < 1e-5);
        assert!((v1[1] - 0.0).abs() < 1e-5);

        let v2 = loaded.get("vec2").unwrap();
        assert!((v2[1] - 1.0).abs() < 1e-5);

        let v3 = loaded.get("vec3").unwrap();
        assert!((v3[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_save_and_load_empty_db() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.db");
        let path_str = path.to_str().unwrap();

        let db = VecDB::new();
        db.save(path_str).unwrap();

        let loaded = VecDB::load(path_str).unwrap();
        assert_eq!(loaded.count(), 0);
        assert_eq!(loaded.dimension, None);
    }

    #[test]
    fn test_load_nonexistent_file() {
        match VecDB::load("nonexistent_file.db") {
            Err(e) => assert!(e.contains("File not found")),
            Ok(_) => panic!("Expected error for nonexistent file"),
        }
    }

    #[test]
    fn test_save_load_preserves_search() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("search.db");
        let path_str = path.to_str().unwrap();

        let mut db = VecDB::new();
        db.insert("vec1".to_string(), vec![1.0, 0.0]).unwrap();
        db.insert("vec2".to_string(), vec![0.0, 1.0]).unwrap();
        db.insert("vec3".to_string(), vec![0.7, 0.7]).unwrap();

        db.save(path_str).unwrap();
        let loaded = VecDB::load(path_str).unwrap();

        // Search on loaded db should return same results
        let results = loaded.search(vec![1.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "vec1");
        assert!((results[0].2 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_save_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("overwrite.db");
        let path_str = path.to_str().unwrap();

        // Save first version
        let mut db = VecDB::new();
        db.insert("old".to_string(), vec![1.0, 0.0]).unwrap();
        db.save(path_str).unwrap();

        // Save second version to same path
        let mut db2 = VecDB::new();
        db2.insert("new1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        db2.insert("new2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        db2.save(path_str).unwrap();

        // Load should get the second version
        let loaded = VecDB::load(path_str).unwrap();
        assert_eq!(loaded.count(), 2);
        assert!(loaded.get("old").is_none());
        assert!(loaded.get("new1").is_some());
        assert!(loaded.get("new2").is_some());
    }
}