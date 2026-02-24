//! Error metrics used for codec selection.

use crate::error::{Error, Result};

/// Normalised Root Mean Square Error.
///
/// NRMSE = `sqrt(MSE) / (max - min)`.
///
/// # Errors
/// Returns `Err(Error::LengthMismatch)` if the input slices differ in length.
#[must_use = "handle the Result to observe metric errors"]
pub fn nrmse(original: &[f64], reconstructed: &[f64]) -> Result<f64> {
    if original.len() != reconstructed.len() {
        return Err(Error::LengthMismatch {
            a: original.len() as u64,
            b: reconstructed.len() as u64,
        });
    }
    if original.is_empty() {
        return Err(Error::EmptyData);
    }

    let mut min = original[0];
    let mut max = original[0];
    if !min.is_finite() {
        return Err(Error::InvalidInput);
    }

    let mut sum_sq = 0.0f64;
    for (&o, &r) in original.iter().zip(reconstructed) {
        if !o.is_finite() || !r.is_finite() {
            return Err(Error::InvalidInput);
        }
        if o < min {
            min = o;
        }
        if o > max {
            max = o;
        }
        let d = o - r;
        sum_sq += d * d;
    }

    let mse = sum_sq / original.len() as f64;
    let rmse = mse.sqrt();
    let denom = max - min;

    if denom == 0.0 {
        if mse == 0.0 {
            return Ok(0.0);
        }
        return Ok(f64::INFINITY);
    }

    Ok(rmse / denom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nrmse_rejects_length_mismatch() {
        let err = nrmse(&[1.0, 2.0], &[1.0]).unwrap_err();
        assert!(matches!(err, Error::LengthMismatch { a: 2, b: 1 }));
    }

    #[test]
    fn nrmse_constant_perfect_reconstruction_is_zero() {
        let v = vec![std::f64::consts::PI; 128];
        assert_eq!(nrmse(&v, &v).unwrap(), 0.0);
    }

    #[test]
    fn nrmse_constant_imperfect_is_infinite() {
        let a = vec![5.0; 8];
        let mut b = a.clone();
        b[0] = 4.0;
        assert!(nrmse(&a, &b).unwrap().is_infinite());
    }

    #[test]
    fn nrmse_basic() {
        let a = [0.0, 10.0];
        let b = [0.0, 9.0];
        // MSE = (0^2 + 1^2)/2 = 0.5, RMSE = sqrt(0.5)
        // denom = 10
        let got = nrmse(&a, &b).unwrap();
        let expected = (0.5f64).sqrt() / 10.0;
        assert!((got - expected).abs() < 1e-12);
    }
}
