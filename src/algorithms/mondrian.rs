use polars::prelude::*;

use crate::{algorithms::anonymization_algorithm::AnonymizationAlgorithm, data::{dataset::Dataset, qi::QuasiIdentifiers}};

pub struct Mondrian {}
impl AnonymizationAlgorithm for Mondrian {
    fn anonymize(k: u32, dataset: Dataset, quasi_identifiers: QuasiIdentifiers) -> Dataset {
        todo!()
    }
}
