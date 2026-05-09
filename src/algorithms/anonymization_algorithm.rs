use crate::data::{dataset::Dataset, qi::QuasiIdentifiers};

pub trait AnonymizationAlgorithm {
    fn anonymize(k: u32, dataset: Dataset, quasi_identifiers: QuasiIdentifiers) -> Dataset;
}
