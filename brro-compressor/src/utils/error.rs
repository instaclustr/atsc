use std::cmp;

/// Calculates the root mean squared error (RMSE) between two vectors.
///
/// # Arguments
///
/// * `vec1` - The first vector of f64 values.
/// * `vec2` - The second vector of f64 values.
///
/// # Returns
///
/// Returns the RMSE, which is the square root of the mean squared error, or an error message
/// if the vector lengths are different.
fn calculate_error(vec1: &Vec<f64>, vec2: &Vec<f64>) -> Result<f64, &'static str> {
    // Check if the lengths of the input vectors are the same.
    if vec1.len() != vec2.len() {
        return Err("Vector lengths are not the same.");
    }

    // Calculate the minimum length of the two vectors.
    let min_length = cmp::min(vec1.len(), vec2.len());

    // Calculate the squared error for each corresponding pair of elements and sum them.
    let squared_error: f64 = (0..min_length)
        .map(|i| (vec1[i] - vec2[i]).powi(2))
        .sum();

    // Calculate the square root of the mean of squared errors and return it.
    Ok(squared_error.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_approx_eq;

    #[test]
    fn test_error_calculation() {
        // Test case 1: Vectors of equal length with known error
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.5, 2.5, 3.5];
        
        // Assert that the calculated RMSE is approximately 0.5
        assert_approx_eq!(calculate_error(&vec1, &vec2).unwrap(), 0.5, epsilon = 1e-6);

        // Test case 2: Vectors of different length (error case)
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.5, 2.5];
        
        // Assert that an error is returned due to different vector lengths.
        assert!(calculate_error(&vec1, &vec2).is_err());
    }
}

fn main() {
    let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let vector2 = vec![1.5, 2.5, 2.8, 3.7, 4.9];
    
    match calculate_error(&vector1, &vector2) {
        Ok(error) => println!("The root mean squared error between vector1 and vector2 is: {:.2}", error),
        Err(msg) => eprintln!("Error: {}", msg),
    }
}
