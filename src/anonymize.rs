use std::ops::Range;
use std::time::{Duration, Instant};

use rayon::prelude::*;

use crate::{
    algorithms::anonymization_algorithm::{AlgorithmError, AnonymizationAlgorithm},
    data::{dataset::Dataset, qi::QuasiIdentifiers},
};

/// Result of a single anonymization run, includes utility metrics.
pub struct AnonymizerOutput {
    /// Name of the algorithm that produced this result.
    pub algorithm_name: String,

    /// The anonymized dataset.
    pub anonymized_dataset: Dataset,

    /// Row count of the original input.
    pub rows_original: usize,

    /// Number of rows suppressed during anonymization.
    pub rows_suppressed: usize,

    /// Time spent inside the algorithm's `anonymize` call.
    pub duration: Duration,
}

/// Result of an anonymization run, either successful with output or an error.
type AnonymizerResult = Result<AnonymizerOutput, AlgorithmError>;

/// Main struct for orchestrating anonymization runs with different algorithms and parameters.
pub struct Anonymizer {
    /// The original dataset to be anonymized.
    dataset: Dataset,

    /// The list of column names that are considered quasi-identifiers.
    quasi_identifiers: QuasiIdentifiers,
}

impl Anonymizer {
    pub fn new(dataset: Dataset, quasi_identifiers: QuasiIdentifiers) -> Self {
        Self {
            dataset,
            quasi_identifiers,
        }
    }

    /// Run a single algorithm against the dataset.
    ///
    /// # Parameters
    /// - `algorithm`: The anonymization algorithm to run.
    /// - `k`: The k in k-anonymity, i.e., the minimum number of rows that must share the same QI values.
    ///
    /// # Returns
    /// - `Ok(AnonymizerOutput)`: The output of the anonymization run, including the anonymized dataset and utility metrics.
    /// - `Err(AlgorithmError)`: An error if the anonymization processing fails.
    pub fn run(&self, algorithm: &dyn AnonymizationAlgorithm, k: u32) -> AnonymizerResult {
        let rows_original = self.dataset.df.height();

        // Get wall-clock time for the anonymization process, excluding any setup or post-processing.
        let start = Instant::now();
        let anonymized =
            algorithm.anonymize(k, self.dataset.clone(), self.quasi_identifiers.clone())?;
        let duration = start.elapsed();

        let rows_suppressed = rows_original.saturating_sub(anonymized.df.height());

        Ok(AnonymizerOutput {
            algorithm_name: algorithm.name().to_string(),
            anonymized_dataset: anonymized,
            rows_original,
            rows_suppressed,
            duration,
        })
    }

    /// Run algorithms across a range of k values in parallel.
    ///
    /// # Parameters
    /// - `k_range`: The range of k values to evaluate (e.g., `2..10`).
    /// - `algorithms`: The anonymization algorithms to run for each k value.
    ///
    /// # Returns
    /// - `Ok(Vec<AnonymizerOutput>)`: A list of anonymization outputs.
    /// - `Err(AlgorithmError)`: An error if any algorithm fails for any k value.
    pub fn run_parallel(
        &self,
        k_range: Range<u32>,
        algorithms: &[&dyn AnonymizationAlgorithm],
    ) -> Result<Vec<AnonymizerOutput>, AlgorithmError> {
        k_range
            .into_par_iter()
            .map(|k| {
                algorithms
                    .iter()
                    .map(|&alg| self.run(alg, k))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|per_k| per_k.into_iter().flatten().collect())
    }
}
