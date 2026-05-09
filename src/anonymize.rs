use std::{ops::Range};

use crate::{algorithms::anonymization_algorithm::AnonymizationAlgorithm, data::{dataset::Dataset, qi::QuasiIdentifiers}};


type AnonymizerResult = Result<AnonymizerOutput, Box<dyn std::error::Error>>;
pub struct AnonymizerOutput {
    pub anonymized_dataset: Dataset,
    // metrics, other info
}

pub struct Anonymizer {
    dataset: Dataset,
    quasi_identifiers: QuasiIdentifiers
}

impl Anonymizer {
    pub fn new(dataset: Dataset, quasi_identifiers: QuasiIdentifiers) -> Self {
        Self { dataset, quasi_identifiers }
    }

    pub fn run_single<A: AnonymizationAlgorithm>(&self, k: u32) -> AnonymizerResult {
        // A::anonymize(k, self.dataset.clone(), self.quasi_identifiers.clone())?
        todo!()
    }

    pub fn run_parallel<A: AnonymizationAlgorithm>(&self, k_range: Range<u32>) -> AnonymizerResult {
        // A::anonymize(k, self.dataset.clone(), self.quasi_identifiers.clone())?
        todo!()
    }
}
