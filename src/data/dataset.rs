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
