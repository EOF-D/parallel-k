use crate::{
    algorithms::anonymization_algorithm::{AlgorithmError, AnonymizationAlgorithm},
    data::{dataset::Dataset, qi::QuasiIdentifiers},
};

#[derive(Default)]
pub struct Mondrian {}

impl AnonymizationAlgorithm for Mondrian {
    fn name(&self) -> &str {
        "Mondrian"
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
