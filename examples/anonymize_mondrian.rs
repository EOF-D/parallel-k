use std::fs::File;

use parallel_k::{
    algorithms::mondrian::Mondrian,
    anonymize::Anonymizer,
    data::{
        dataset::Dataset,
        qi::{QIType, QuasiIdentifiers},
    },
};

use polars::prelude::*;

#[path = "./util/util.rs"]
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let df = util::clean_adult_csv("adult.csv".into())?;

    let qis = QuasiIdentifiers::from_column_names(&[
        ("age", QIType::Numerical { leaf_bucket_size: 5 }),

        ("workclass", QIType::Categorical {
            path_to_json_hierarchy: "taxonomies/workclass.json".into(),
        }),

        ("education", QIType::Categorical {
            path_to_json_hierarchy: "taxonomies/education.json".into(),
        }),
    ]);

    let dataset = Dataset::build(&df, &qis)?;
    let anonymizer = Anonymizer::new(dataset, qis);

    let result = anonymizer.run(&Mondrian::default(), 100)
        .expect("Mondrian k=100 failed");

    println!(
        "[Mondrian k=100] {} rows -> {} anonymized, {} suppressed ({:.2?})",
        result.rows_original,
        result.anonymized_dataset.df.height(),
        result.rows_suppressed,
        result.duration,
    );

    println!("{}", result.anonymized_dataset.df.head(Some(15)));

    let verified = util::verify_k_anonymity(&result.anonymized_dataset.df, &["age", "workclass", "education"], 100);
    println!("k=100 anonymity satisfied: {verified}");

    let mut out = result.anonymized_dataset.df;
    let mut file = File::create("output.csv")?;
    CsvWriter::new(&mut file)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut out)?;
    println!("wrote output.csv ({} rows)", out.height());

    Ok(())
}
