pub mod categorical;
pub mod numerical;
pub use numerical::NumericalTaxonomy;
pub use categorical::{CategoricalTaxonomy, CategoricalHierarchy};
use std::collections::HashMap;
use polars::prelude::*;

use crate::data::QIType;
use crate::data::QuasiIdentifiers;

#[derive(Clone)]
pub struct TaxonomyManager {
    pub numerical_taxonomies: HashMap<String, NumericalTaxonomy>,
    pub categorical_taxonomies: HashMap<String, CategoricalTaxonomy>
}

impl TaxonomyManager {
    pub fn new() -> Self {
        Self {
            numerical_taxonomies: HashMap::new(),
            categorical_taxonomies: HashMap::new(),
        }
    }

    pub fn build_from_qis(
        df: &DataFrame,
        qis: &QuasiIdentifiers
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manager = Self::new();

        for qi in &qis.0 {
            match &qi.qi_type {
                QIType::Numerical { leaf_bucket_size } => {
                    println!("\n--- {} ---", &qi.column_name);
                    let col = df.column(&qi.column_name);
                    let series = col?.i64()?;

                    let min = series.min().unwrap_or(0);
                    let max = series.max().unwrap_or(0);
                    println!("Range: {} to {}", min, max);

                    let taxonomy = NumericalTaxonomy::create_from_data_range(
                        &qi.column_name,
                        min,
                        max,
                        *leaf_bucket_size,
                        3,
                    );

                    taxonomy.print_numerical_taxanomy_tree();
                    manager.numerical_taxonomies.insert(qi.column_name.clone(), taxonomy);
                }   
                QIType::Categorical { path_to_json_hierarchy } => {
                    println!("\n--- {} ---", &qi.column_name);
                    let hierarchy = load_hierarchy_from_json(path_to_json_hierarchy)?;

                    let taxonomy = CategoricalTaxonomy::create_from_hierarchy(
                        &qi.column_name,
                        &hierarchy,
                    )?;

                    taxonomy.print_categorical_taxanomy_tree();
                    manager.categorical_taxonomies.insert(qi.column_name.clone(), taxonomy);
                }
            } 
        }

        Ok(manager)
    }
}

fn load_hierarchy_from_json(path: &str) -> Result<CategoricalHierarchy, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(path)?;

    Ok(serde_json::from_str(&json)?)
}