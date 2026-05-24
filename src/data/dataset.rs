use polars::prelude::*;
use crate::data::qi::QuasiIdentifiers;
use crate::taxonomy::TaxonomyManager;

#[derive(Clone)]
pub struct Dataset {
    pub df: DataFrame,
    pub qis: QuasiIdentifiers,
    pub taxonomies: TaxonomyManager,
}

impl Dataset {
    // ease of use functions for basics
    // (when data was already preprocessed/cleaned)
    pub fn from_csv() -> Self { todo!() }
    pub fn from_json() -> Self { todo!() }
    // ...etc


    // more specific, consumer wants to provide dataframe after they
    // do stuff with it
    pub fn from_dataframe(_df: DataFrame) -> Self { todo!() }


    /// Creates a Dataset from an already anonymized DataFrame and pre-built taxonomies.
    ///
    /// Prefer `Dataset::build` when starting from raw data where taxonomies still need to be derived.
    ///
    /// # Parameters
    /// - `df`: The anonymized DataFrame containing the data.
    /// - `qis`: The quasi-identifiers associated with the dataset.
    /// - `taxonomies`: The pre-built TaxonomyManager containing the taxonomies for the dataset.
    ///
    /// # Returns
    /// - `Dataset`: The constructed dataset.
    pub fn from_anonymized(df: DataFrame, qis: QuasiIdentifiers, taxonomies: TaxonomyManager) -> Self {
        Self { df, qis, taxonomies }
    }

    // build taxonomies
    pub fn build(
        df: &DataFrame,
        qis: &QuasiIdentifiers
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let taxonomies = TaxonomyManager::build_from_qis(df, qis)?;

        Ok(Self {
            df: df.clone(),
            qis: qis.clone(),
            taxonomies,
        })
    }
}
