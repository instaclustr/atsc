/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::cmp;

#[derive(Default, Debug, Clone)]
pub enum ErrorMethod {
    Mse,
    Nmse,
    Mae,
    #[default]
    Mape,
    Smape,
}

impl ErrorMethod {
    pub fn error(self, original: &[f64], generated: &[f64]) -> f64 {
        match self {
            ErrorMethod::Mse => error_mse(original, generated),
            ErrorMethod::Nmse => error_nmsqe(original, generated),
            ErrorMethod::Mae => error_mae(original, generated),
            ErrorMethod::Mape => error_mape(original, generated),
            ErrorMethod::Smape => error_smape(original, generated),
        }
    }
}

/// This function calculates the error between 2 arrays of f64. The results are from 0 to ..
/// Being 0, no error, 1 - 100% error and so on.
/// This uses the default function to calculte it.
pub fn calculate_error(original: &[f64], generated: &[f64]) -> f64 {
    ErrorMethod::error(ErrorMethod::default(), original, generated)
}

/// This function calculates the error between 2 arrays of f64. The results are from 0 to ..
/// Being 0, no error, 1 - 100% error and so on.
/// This uses the provided method to calculte it.
pub fn calculate_error_method(original: &[f64], generated: &[f64], method: ErrorMethod) -> f64 {
    ErrorMethod::error(method, original, generated)
}
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
pub fn error_mse(vec1: &[f64], vec2: &[f64]) -> f64 {
    if vec1.len() != vec2.len() {
        panic!("Can't compute error! Arrays with different lenghts.")
    }

    let min_length = cmp::min(vec1.len(), vec2.len());
    let squared_error: f64 = (0..min_length).map(|i| (vec1[i] - vec2[i]).powi(2)).sum();
    squared_error / min_length as f64
}

/// Computes the Normalized Mean Square Error between 2 signals
/// # Panics:
/// When the 2 arrays don't have the same size
pub fn error_nmsqe(original: &[f64], generated: &[f64]) -> f64 {
    if original.len() != generated.len() {
        panic!("Can't compute error! Arrays with different lenghts.")
    }

    let squared_error: f64 = original
        .iter()
        .zip(generated.iter())
        .map(|(original, generated)| (generated - original).powi(2))
        .sum();
    let original_square_sum: f64 = original.iter().map(|original| original.powi(2)).sum();
    squared_error / original_square_sum
}

/// Computes the Mean Absolute Error between 2 signals
/// # Panics:
/// When the 2 arrays don't have the same size
pub fn error_mae(original: &[f64], generated: &[f64]) -> f64 {
    if original.len() != generated.len() {
        panic!("Can't compute error! Arrays with different lenghts.")
    }

    let abs_error: f64 = original
        .iter()
        .zip(generated.iter())
        .map(|(original, generated)| (generated - original).abs())
        .sum();
    abs_error / original.len() as f64
}

/// Computes the Mean Absolute Percentage Error between 2 signals
/// # Panics:
/// When the 2 arrays don't have the same size
pub fn error_mape(original: &[f64], generated: &[f64]) -> f64 {
    if original.len() != generated.len() {
        panic!("Can't compute error! Arrays with different lenghts.")
    }

    let abs_error: f64 = original
        .iter()
        .zip(generated.iter())
        .map(|(original, generated)| ((generated - original) / original).abs())
        .sum();
    // TODO: NaN needs to be handled
    abs_error / original.len() as f64
}

/// Computes the Symmetric Mean Absolute Percentage Error between 2 signals
/// # Panics:
/// When the 2 arrays don't have the same size
/// Output between 0 and 1 (1 is 100% error)
pub fn error_smape(original: &[f64], generated: &[f64]) -> f64 {
    if original.len() != generated.len() {
        panic!("Can't compute error! Arrays with different lenghts.")
    }

    let mut sum_up = 0.0;
    let mut sum_down = 0.0;
    for (&value_original, &value_generated) in original.iter().zip(generated) {
        if value_generated == 0.0 && value_original == 0.0 {
            continue;
        }
        sum_up += (value_generated - value_original).abs();
        sum_down += value_original.abs() + value_generated.abs();
    }
    (sum_up / sum_down) / original.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_error() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vector2 = vec![2.5, 4.0, 6.0, 8.0, 10.0];

        assert_eq!(error_mse(&vector1, &vector1), 0.0);
        assert_eq!(error_mse(&vector1, &vector2), 11.25);
    }

    #[test]
    fn test_calculate_nmsqe() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vector2 = vec![2.5, 4.0, 6.0, 8.0, 10.0];

        assert_eq!(error_nmsqe(&vector1, &vector1), 0.0);
        assert_eq!(error_nmsqe(&vector1, &vector2), 1.0227272727272727);
    }

    #[test]
    fn test_calculate_mae() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vector2 = vec![2.5, 4.0, 6.0, 8.0, 10.0];

        assert_eq!(error_mae(&vector1, &vector1), 0.0);
        assert_eq!(error_mae(&vector1, &vector2), 3.1);
    }

    #[test]
    fn test_calculate_mape() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vector2 = vec![2.5, 4.0, 6.0, 8.0, 10.0];
        let vector3 = vec![1.0];
        let vector4 = vec![1.1];

        assert_eq!(error_mape(&vector1, &vector1), 0.0);
        assert_eq!(error_mape(&vector1, &vector2), 1.1);
        assert!(error_mape(&vector3, &vector4) < 0.101);
    }

    #[test]
    fn test_calculate_smape() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vector2 = vec![2.5, 4.0, 6.0, 8.0, 10.0];
        let vector3 = vec![1.0];
        let vector4 = vec![1.1];
        let vector5 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0,
        ];

        assert_eq!(error_smape(&vector1, &vector1), 0.0);
        assert_eq!(error_smape(&vector5, &vector5), 0.0);
        assert!(error_smape(&vector1, &vector2) < 0.353);
        assert!(error_smape(&vector3, &vector4) < 0.101);
        assert!(error_smape(&[1.0], &[2.0]) > 0.33);
    }
}
