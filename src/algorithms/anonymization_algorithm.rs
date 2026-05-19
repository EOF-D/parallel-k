use crate::data::{dataset::Dataset, qi::QuasiIdentifiers};

/// Error produced by the anonymization algorithm(s).
///
/// Send + Sync so errors can be returned from worker threads.
pub type AlgorithmError = Box<dyn std::error::Error + Send + Sync>;

/// Trait for anonymization algorithms.
///
/// Pluggable k-anonymity algorithm, e.g, Mondrian, Incognito, etc.
pub trait AnonymizationAlgorithm: Send + Sync {
    /// Name for the algorithm, e.g., "Mondrian", "Incognito", etc. (for debugging, logging, etc.)
    fn name(&self) -> &str;

    /// Anonymize the dataset so that each group of rows sharing the same QIs has at least k rows.
    ///
    /// # Parameters
    /// - `k`: The k in k-anonymity, i.e., the minimum number of rows that must share the same QI values.
    /// - `dataset`: The input dataset to be anonymized.
    /// - `quasi_identifiers`: The list of column names that are considered quasi-identifiers.
    ///
    /// # Returns
    /// - `Ok(Dataset)`: The anonymized dataset.
    /// - `Err(AlgorithmError)`: An error if the anonymization process fails, e.g., if the dataset is empty, etc.
    fn anonymize(
        &self,
        k: u32,
        dataset: Dataset,
        quasi_identifiers: QuasiIdentifiers,
    ) -> Result<Dataset, AlgorithmError>;
}
