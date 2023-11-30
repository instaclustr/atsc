use crate::compressor::{constant::constant, fft::fft, polynomial::{polynomial, PolynomialType}};
use std::thread;

/// Enum to represent the decision between compressors.
#[derive(PartialEq, Debug)]
enum CompressionDecision {
    Constant,
    Fft,
    Polynomial,
}

impl CompressionDecision {
    /// Function to perform compression and make a decision based on the results.
    pub fn compress_and_decide() -> Result<(), Box<dyn std::error::Error>> {
        // Sample data for testing
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // Clone data for each compressor
        let data_constant = data.clone();
        let data_fft = data.clone();
        let data_polynomial = data.clone();

        // Create threads for each compressor
        let thread_constant = thread::spawn(move || constant(&data_constant));
        let thread_fft = thread::spawn(move || fft(&data_fft));
        let thread_polynomial = thread::spawn(move || polynomial(&data_polynomial, PolynomialType::Polynomial));

        // Wait for threads to finish and collect their results with error handling
        let result_constant = thread_constant.join().map_err(|e| format!("Constant thread error: {:?}", e))?;
        let result_fft = thread_fft.join().map_err(|e| format!("FFT thread error: {:?}", e))?;
        let result_polynomial = thread_polynomial.join().map_err(|e| format!("Polynomial thread error: {:?}", e))?;

        // Use the decision logic to determine the compression decision
        let decision = match (result_constant.len(), result_fft.len(), result_polynomial.len()) {
            (constant_len, fft_len, poly_len) if constant_len < fft_len && constant_len < poly_len => CompressionDecision::Constant,
            (_, fft_len, poly_len) if fft_len < poly_len => CompressionDecision::Fft,
            _ => CompressionDecision::Polynomial,
        };

        // Use the decision to perform further actions
        match decision {
            CompressionDecision::Constant => {
                println!("Selected Constant Compressor");
            }
            CompressionDecision::Fft => {
                println!("Selected FFT Compressor");
            }
            CompressionDecision::Polynomial => {
                println!("Selected Polynomial Compressor");
            }
        }

        Ok(())
    }

}
fn get_compression_decision(
    result_constant: &[f64],
    result_fft: &[f64],
    result_polynomial: &[f64],
) -> CompressionDecision {
    match (result_constant.len(), result_fft.len(), result_polynomial.len()) {
        (constant_len, fft_len, poly_len) if constant_len < fft_len && constant_len < poly_len => CompressionDecision::Constant,
        (_, fft_len, poly_len) if fft_len < poly_len => CompressionDecision::Fft,
        _ => CompressionDecision::Polynomial,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_comparison_logic_constant_wins() {
        let result_constant = vec![1.0, 2.0, 3.0];
        let result_fft = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result_polynomial = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let decision = get_compression_decision(&result_constant, &result_fft, &result_polynomial);
        assert_eq!(decision, CompressionDecision::Constant);
    }

    #[test]
    fn test_comparison_logic_fft_wins() {
        let result_constant = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result_fft = vec![1.0, 2.0, 3.0];
        let result_polynomial = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let decision = get_compression_decision(&result_constant, &result_fft, &result_polynomial);
        assert_eq!(decision, CompressionDecision::Fft);
    }

    #[test]
    fn test_comparison_logic_polynomial_wins() {
        let result_constant = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result_fft = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result_polynomial = vec![1.0, 2.0, 3.0];
        let decision = get_compression_decision(&result_constant, &result_fft, &result_polynomial);
        assert_eq!(decision, CompressionDecision::Polynomial);
    }
    
}