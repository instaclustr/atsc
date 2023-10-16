use bincode::{Decode, Encode};
use std::{collections::BinaryHeap, cmp::Ordering};
use rustfft::{FftPlanner, num_complex::Complex};
use crate::utils::error::calculate_error;

use super::BinConfig;
use log::{error, debug, warn, info};

const CONSTANT_COMPRESSOR_ID: u8 = 15;

/// Struct to store frequencies, since bincode can't encode num_complex Complex format, this one is compatible
// This could be a Generic to support f64, integers, etc...
#[derive(Encode, Decode, Debug, Copy, Clone)]
pub struct FrequencyPoint {
    /// Frequency position
    pos: u16, // This is the reason that frame size is limited to 65535, probably enough
    freq_real: f32,
    freq_img: f32
}

impl FrequencyPoint {
    pub fn new(real: f32, img: f32) -> Self {
        FrequencyPoint { pos: 0, freq_real: real, freq_img: img }
    }

    pub fn with_position(real: f32, img: f32, pos: u16) -> Self {
        FrequencyPoint { pos, freq_real: real, freq_img: img }
    }

    pub fn from_complex(complex: Complex<f32>) -> Self {
        FrequencyPoint { pos: 0, freq_real: complex.re, freq_img: complex.im }
    }

    pub fn from_complex_with_position(complex: Complex<f32>, pos: u16) -> Self {
        FrequencyPoint { pos, freq_real: complex.re, freq_img: complex.im }
    }

    pub fn to_complex(self) -> Complex<f32> {
        Complex{re: self.freq_real, im: self.freq_img}
    }

    pub fn to_inv_complex(self) -> Complex<f32> {
        Complex{re: self.freq_real, im: self.freq_img * -1.0}
    }

    /// To allow to use rust std structures
    fn partial_cmp(&self, other: &Self) -> Ordering {
        let c1 = Complex{re: self.freq_real, im: self.freq_img};
        let c2 = Complex{re: other.freq_real, im: other.freq_img};
        if self == other { Ordering::Equal }
        else if c1.norm() > c2.norm()  { return Ordering::Greater;}
        else { return Ordering::Less;}

    }
}

// This is VERY specific for this use case, DO NOT RE-USE! This NORM comparison is false for complex numbers
impl PartialEq for FrequencyPoint {
    fn eq(&self, other: &Self) -> bool {
        let c1 = Complex{re: self.freq_real, im: self.freq_img};
        let c2 = Complex{re: other.freq_real, im: other.freq_img};
        c1.norm() == c2.norm()
    }
}

impl Eq for FrequencyPoint {}

impl PartialOrd for FrequencyPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.partial_cmp(other))
    }
}

impl Ord for FrequencyPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
    }
}

/// FFT Compressor. Applies FFT to a signal, picks the N best frequencies, discards the rest. Always LOSSY
#[derive(Encode, Decode, PartialEq, Debug)]
pub struct FFT {
    /// Compressor ID
    pub id: u8,
    /// Stored frequencies
    pub frequencies: Vec<FrequencyPoint>,
    /// The maximum numeric value of the points in the frame
    pub max_value: f32,  
    /// The minimum numeric value of the points in the frame
    pub min_value: f32,  
}

impl FFT {
    /// Creates a new instance of the Constant compressor with the size needed to handle the worst case
    pub fn new(frame_size: usize, min: f64, max: f64) -> Self {
        debug!("FFT compressor");
        FFT {
            id: CONSTANT_COMPRESSOR_ID,
            frequencies: Vec::with_capacity(frame_size),
            /// The maximum numeric value of the points in the frame
            max_value: FFT::f64_to_f32(max),  
            /// The minimum numeric value of the points in the frame
            min_value: FFT::f64_to_f32(min),  
            }
    }

    // TODO: Move this to the optimizer?
    fn f64_to_f32(x: f64) -> f32 {
        let y = x as f32;
        if !(x.is_finite() && y.is_finite()) {
            // PANIC? Error?
            error!("f32 overflow during conversion");
        }
        y
    }

    /// Rounds a number to the specified number of decimal places
    // TODO: Move this into utils? I think this will be helpfull somewhere else.
    fn round(&self, x: f32, decimals: u32) -> f64 {
        let y = 10i32.pow(decimals) as f64;
        let out = (x as f64 * y).round() / y;
        if out > self.max_value as f64 { return self.max_value as f64; }
        if out < self.min_value as f64 { return self.min_value as f64; }
        out
    }

    // Converts an f64 vec to an Vec of Complex F32
    fn optimize(data: &[f64]) -> Vec<Complex<f32>> {
        data.iter()
            .map(|x| Complex{re: FFT::f64_to_f32(*x), im: 0.0f32})
            .collect()
    }

    /// Removes the smallest frequencies from `buffer` until `max_freq` remain
    fn fft_trim(buffer: &mut Vec<Complex<f32>>, max_freq: usize) ->  Vec<FrequencyPoint> {
        // We need half + 1 frequencies at most, due to the mirrored nature of FFT (signal is always real!)
        // and the first one being the dc component
        let size = (buffer.len() / 2) + 1;
        buffer.truncate(size);
        let mut freq_vec = Vec::with_capacity(max_freq);
        if max_freq == 1 {
            freq_vec.push(FrequencyPoint::from_complex_with_position(buffer[0], 0));
            return freq_vec 
            }
        // More than 1 frequency needed, get the biggest frequencies now.
        // Move from the buffer into Frequency Vectors
        let tmp_vec: Vec<FrequencyPoint> = buffer.iter().enumerate()
                      .map(|(pos, &f)| FrequencyPoint::from_complex_with_position(f, pos as u16))
                      .collect();
        // This part, is because Binary heap is very good at "give me the top N elements"
        let mut heap = BinaryHeap::from(tmp_vec);
        // Now that we have it, let's pop the elements we need!
        for _ in 0..max_freq {
            if let Some(item) = heap.pop() {freq_vec.push(item)}
        }
        freq_vec
    }

    /// Compress data via FFT. 
    /// This picks a set of data, computes the FFT, and stores the most relevant frequencies, dropping
    /// the remaining ones.
    pub fn compress(&mut self, data: &[f64], max_freq: usize) {
        // First thing, always try to get the data len as a power of 2. 
        let v = data.len();
        if !v.is_power_of_two() {
            warn!("Slow FFT, data segment is not a power of 2!");
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(v);
        let mut buffer = FFT::optimize(data);
        // The data is processed in place, it gets back to the buffer
        fft.process(&mut buffer);
        self.frequencies = FFT::fft_trim(&mut buffer, max_freq);
    }

    /// Decompresses data
    pub fn decompress(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (fft, _) = bincode::decode_from_slice(data, config).unwrap();
        fft
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    /// Gets the full sized array with the frequencies mirrored
    fn get_mirrored_freqs(&self, len: usize) -> Vec<Complex<f32>> {
        // Because we are dealing with Real inputs, we only store half the frequencies, but 
        // we need all for the ifft
        let mut data = vec![Complex{re: 0.0f32, im: 0.0f32}; len];
        for f in &self.frequencies {
            let pos = f.pos as usize;
            data[pos] = f.to_complex();
            // fo doesn't mirror
            if pos == 0 { continue; }
            // Mirror and invert the imaginary part
            data[len-pos] = f.to_inv_complex()
        }
        data
    }

    /// Returns an array of data
    /// Runs the ifft, and push residuals into place and/or adjusts max and mins accordingly
    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        // Vec to process the ifft
        let mut data = self.get_mirrored_freqs(frame_size);
        // Plan the ifft
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(frame_size);
        // run the ifft
        fft.process(&mut data);
        // We need this for normalization
        let len = frame_size as f32;
        // We only need the real part
        // TODO: Only 1 decimal place is sketchy!
        let out_data = data.iter()
                           .map(|&f| self.round(f.re/len, 1))
                           .collect();
        out_data
    }
}

/// Compresses a data segment via FFT.
pub fn fft(data: &[f64], max_freqs: usize, min: f64, max: f64) -> Vec<u8> {
    info!("Initializing FFT Compressor");
    // Initialize the compressor
    let mut c = FFT::new(data.len(), min, max);
    // Convert the data
    c.compress(data, max_freqs);
    // Convert to bytes
    c.to_bytes()
}

/// Compress targeting a specific max error allowed. This is very computational intensive,
/// as the FFT will be calculated over and over until the specific error threshold is achived.
/// `max_freqs` is used as a starting point for the calculation
pub fn fft_allowed_error(data: &[f64], max_freqs: usize, min: f64, max: f64, allowed_error: f64) -> Vec<u8> {
    let frame_size = data.len();
    let compressed_data = fft(data, max_freqs, min, max);
    let mut out = FFT::decompress(&compressed_data).to_data(frame_size);
    let mut e = calculate_error(&data.to_vec(), &out).unwrap();
    let mut i = 1;

    while e > allowed_error {
        // Calculate the FFT only once.
        let fft_result = fft(data, max_freqs + i, min, max);
        out = FFT::decompress(&fft_result).to_data(frame_size);
        e = calculate_error(&data.to_vec(), &out).unwrap();
        i += 1;
    }

    compressed_data
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(fft(&vector1, 2, 1.0, 5.0), [15, 2, 0, 0, 0, 152, 65, 0, 0, 0, 0, 4, 0, 0, 96, 192, 102, 144, 138, 64, 0, 0, 160, 64, 0, 0, 128, 63]);
    }

    #[test]
    fn test_to_lossess_data() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let compressed_data = fft(&vector1, frame_size, 1.0, 5.0);
        let out = FFT::decompress(&compressed_data).to_data(frame_size);
        assert_eq!(vector1, out);
    }

    #[test]
    fn test_to_lossy_data() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let lossy_vec = vec![1.1, 1.0, 1.1, 1.0, 2.1, 1.0, 1.1, 1.0, 3.1, 1.0, 1.1, 4.9];
        let frame_size = vector1.len();
        let compressed_data = fft(&vector1, 6, 1.0, 5.0);
        let out = FFT::decompress(&compressed_data).to_data(frame_size);
        assert_eq!(lossy_vec, out);
    }

    #[test]
    fn test_to_allowed_error() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let compressed_data = fft_allowed_error(&vector1, 1, 1.0, 5.0, 0.01);
        let out = FFT::decompress(&compressed_data).to_data(frame_size);
        let e = calculate_error(&vector1, &out).unwrap();
        assert!(e <= 0.01);
    }
}
