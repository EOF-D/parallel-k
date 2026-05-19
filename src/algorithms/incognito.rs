use crate::{
    algorithms::anonymization_algorithm::{AlgorithmError, AnonymizationAlgorithm},
    data::{dataset::Dataset, qi::QuasiIdentifiers},
};

#[derive(Default)]
pub struct Incognito {}

impl AnonymizationAlgorithm for Incognito {
    fn name(&self) -> &str {
        "Incognito"
    }

    fn anonymize(
        &self,
        k: u32,
        dataset: Dataset,
        quasi_identifiers: QuasiIdentifiers,
    ) -> Result<Dataset, AlgorithmError> {
        todo!()
    }
}
