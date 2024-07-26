use crate::csv::Sample;
use crate::metric::Error::UpdateForPointError;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use vsri::{day_elapsed_seconds, Vsri};
use wavbrro::wavbrro::WavBrro;

/// Metric is responsible for generating WavBrro and VSRI from parsed Samples
#[derive(Default)]
pub struct Metric {
    /// Metric data itself
    pub wbro: WavBrro,
    /// Metric indexes
    pub vsri: Vsri,
}

#[derive(Debug)]
pub enum Error {
    UpdateForPointError(Sample),
    UnknownError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateForPointError(sample) => {
                write!(f, "updating for point failed, sample: {:?}", sample)
            }
            _ => write!(f, "unknown error occurred"),
        }
    }
}

impl std::error::Error for Error {}

impl Metric {
    /// Creates new WavBrro instance
    pub fn new(wbro: WavBrro, vsri: Vsri) -> Self {
        Metric { wbro, vsri }
    }

    /// Appends samples to the metric
    pub fn append_samples(&mut self, samples: &Vec<Sample>) -> Result<(), Error> {
        for sample in samples {
            // For solution simplification it generates only 1 WavBrro and 1 VSRI
            let ts = day_elapsed_seconds(sample.timestamp / 1000);
            let result = self.vsri.update_for_point(ts);
            if result.is_err() {
                return Err(UpdateForPointError(sample.clone()));
            }

            self.wbro.add_sample(sample.value);
        }

        Ok(())
    }

    /// Creates default metric from the existing samples
    pub fn from_samples(samples: &Vec<Sample>) -> Result<Self, Error> {
        let mut metric = Metric::default();
        match metric.append_samples(samples) {
            Ok(_) => Ok(metric),
            Err(err) => Err(err),
        }
    }

    /// Flushes underlying WavBrro formatted metrics to the file at path
    pub fn flush_wavbrro(&self, path: &String) {
        let mut file_path = PathBuf::from(path);
        file_path.set_extension("wbro");
        self.wbro.to_file(&file_path)
    }

    /// Flushes underlying VSRI to the file at path
    pub fn flush_indexes(&self, path: &String) {
        let mut file_path = PathBuf::from(path);
        file_path.set_extension("vsri");
        self.vsri
            .flush_to(&file_path)
            .expect("Failed to write indexes!")
    }

    /// Returns vector of Samples by iterating over data inside underlying WavBrro
    /// and getting timestamp for each of data point from VSRI
    pub fn get_samples(&self) -> Vec<Sample> {
        let mut samples = Vec::new();
        let values = self.wbro.clone().get_samples();
        for (i, value) in values.iter().enumerate() {
            let ts = self.vsri.get_time(i as i32);
            samples.push(Sample::new(ts.unwrap() as i64, *value));
        }
        samples
    }
}
