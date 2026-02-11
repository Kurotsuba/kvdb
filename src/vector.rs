//! This is the vector math module
//! Provide L2 normalization and dot product

/// L2 Normalization
/// norm_vec = vec / ||vec||
/// Zero vector cannot be normalized
pub fn l2_norm(vector: &[f32]) -> Result<Vec<f32>, String> {
    if vector.is_empty() {
        return Err("Cannot normalize an empty vector".to_string());
    }

    let norm = vector.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt();

    if norm == 0.0 {
        return Err("Cannot normalize a zero vector".to_string());
    }

    let normed_vec = vector.iter()
        .map(|x| x / norm)
        .collect();

    Ok(normed_vec)

}

/// Dot Product
/// dot_prod = sum(a[i] * b[i]) for i = 0..a.len()
/// Can only process vectors with same dimensions
pub fn dot_product(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err("Different dimentions".to_string());
    }

    let dot_prod = left.iter()
        .zip(right.iter())
        .map(|(x, y)| x * y)
        .sum();

    Ok(dot_prod)
}

#[cfg(test)]
mod vector_test {
    use super::*;

    // ========== L2 Normalization Tests ==========

    #[test]
    fn test_l2_norm_basic() {
        // Test case: [3.0, 4.0] should normalize to [0.6, 0.8]
        // Because ||[3,4]|| = sqrt(9+16) = 5
        let vector = vec![3.0, 4.0];
        let result = l2_norm(&vector).unwrap();

        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.6).abs() < 1e-6);
        assert!((result[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_l2_norm_is_unit_length() {
        // Verify that normalized vector has length 1
        let vector = vec![1.0, 2.0, 3.0, 4.0];
        let result = l2_norm(&vector).unwrap();

        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_l2_norm_single_element() {
        let vector = vec![5.0];
        let result = l2_norm(&vector).unwrap();

        assert_eq!(result.len(), 1);
        assert!((result[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_l2_norm_negative_values() {
        // Test with negative values: [-3.0, 4.0]
        let vector = vec![-3.0, 4.0];
        let result = l2_norm(&vector).unwrap();

        assert!((result[0] - (-0.6)).abs() < 1e-6);
        assert!((result[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_l2_norm_zero_vector_error() {
        // Zero vector should return an error
        let vector = vec![0.0, 0.0, 0.0];
        let result = l2_norm(&vector);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot normalize a zero vector");
    }

    #[test]
    fn test_l2_norm_empty_vector() {
        // Empty vector should return error (0 norm)
        let vector = vec![];
        let result = l2_norm(&vector);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot normalize an empty vector");
    }

    // ========== Dot Product Tests ==========

    #[test]
    fn test_dot_product_basic() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        // Expected: 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        let result = dot_product(&a, &b).unwrap();

        assert!((result - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_orthogonal() {
        // Orthogonal vectors should have dot product = 0
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let result = dot_product(&a, &b).unwrap();

        assert!((result - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_dimension_mismatch() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0];  // Different dimension

        let result = dot_product(&a, &b);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Different dimentions");
    }

    #[test]
    fn test_dot_product_zero_vectors() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = dot_product(&a, &b).unwrap();

        assert!((result - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_empty_vectors() {
        let a = vec![];
        let b = vec![];
        let result = dot_product(&a, &b).unwrap();

        assert!((result - 0.0).abs() < 1e-6);
    }

    // ========== Integration Test ==========

    #[test]
    fn test_normalize_then_dot_product() {
        // End-to-end test: normalize two vectors then compute similarity
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.7, 0.7, 0.0];

        let n1 = l2_norm(&v1).unwrap();
        let n2 = l2_norm(&v2).unwrap();

        let similarity = dot_product(&n1, &n2).unwrap();

        // v2 normalized is ~[0.707, 0.707, 0]
        // dot product with [1,0,0] should be ~0.707
        assert!((similarity - 0.707).abs() < 0.001);
    }
}