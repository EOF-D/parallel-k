use std::fs::File;

use polars::prelude::*;
use parallel_k::data::{Dataset, QuasiIdentifiers, QIType};

#[path = "./util/util.rs"]
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = util::clean_adult_csv("adult.csv".into())?;

    unsafe { std::env::set_var("POLARS_FMT_MAX_COLS", df.width().to_string()); }

    println!("{df}");

    let mut outf = File::create("adult_clean.csv").expect("could not create file");
    CsvWriter::new(&mut outf)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut df)?;

    // just age numerical qi for phase 1
    let qis = QuasiIdentifiers::from_column_names(&[
        ("age", QIType::Numerical { leaf_bucket_size: 5 }),
        ("workclass", QIType::Categorical { path_to_json_hierarchy: "taxonomies/workclass.json".into() }),
        ("education", QIType::Categorical { path_to_json_hierarchy: "taxonomies/education.json".into() }),
    ]);

    let dataset = Dataset::build(&df, &qis)?;

    Ok(())
}
