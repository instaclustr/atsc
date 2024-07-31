use crate::csv::Sample;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
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
            Error::UpdateForPointError(sample) => {
                write!(f, "updating for point failed, sample: {:?}", sample)
            }
            Error::UnknownError => write!(f, "unknown error occurred"),
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
    pub fn append_samples(&mut self, samples: &[Sample]) -> Result<(), Error> {
        for sample in samples {
            // For solution simplification it generates only 1 WavBrro and 1 VSRI
            let ts = day_elapsed_seconds(sample.timestamp / 1000);
            self.vsri
                .update_for_point(ts)
                .map_err(|_| Error::UpdateForPointError(sample.clone()))?;

            self.wbro.add_sample(sample.value);
        }

        Ok(())
    }

    /// Creates default metric from the existing samples
    pub fn from_samples(samples: &[Sample]) -> Result<Self, Error> {
        let mut metric = Metric::default();
        metric.append_samples(samples)?;
        Ok(metric)
    }

    /// Flushes underlying WavBrro formatted metrics to the file at path
    pub fn flush_wavbrro(&self, path: &Path) {
        self.wbro.to_file(path)
    }

    /// Flushes underlying VSRI to the file at path
    pub fn flush_indexes(&self, path: &Path) -> Result<(), std::io::Error> {
        self.vsri.flush_to(path)
    }

    /// Returns vector of Samples by iterating over data inside underlying WavBrro
    /// and getting timestamp for each of data point from VSRI.
    pub fn get_samples(self) -> Vec<Sample> {
        self.wbro
            .get_samples()
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let ts = self.vsri.get_time(i as i32);
                Sample::new(ts.unwrap() as i64, *value)
            })
            .collect()
    }
}
