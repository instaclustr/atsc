use std::cmp;

/// Calculates the mean squared error between two vectors.
///
/// # Arguments
///
/// * `vec1` - The first vector of f64 values.
/// * `vec2` - The second vector of f64 values.
///
/// # Returns
///
/// The mean squared error, or an error message if the vector lengths are different.
fn calculate_error(vec1: &Vec<f64>, vec2: &Vec<f64>) -> Result<f64, &'static str> {
    if vec1.len() != vec2.len() {
        return Err("Vector lengths are not the same.");
    }

    let min_length = cmp::min(vec1.len(), vec2.len());
    let squared_error: f64 = (0..min_length)
        .map(|i| (vec1[i] - vec2[i]).powi(2))
        .sum();
    Ok(squared_error / min_length as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_error() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vector2 = vec![1.5, 2.5, 2.8, 3.7, 4.9];

        match calculate_error(&vector1, &vector2) {
            Ok(error) => println!("The mean squared error between vector1 and vector2 is: {:.2}", error),
            Err(msg) => panic!("Error: {}", msg),
        }
    }
}
