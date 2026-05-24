use std::collections::HashMap;
use std::path::PathBuf;

use polars::prelude::*;

pub fn clean_adult_csv(path: PathBuf) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let mut df = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default()
            .with_try_parse_dates(true)
        )
        .try_into_reader_with_file_path(Some(path))?
        .finish()?;

    // replace qmarks with nowe
    for i in 0..df.width() {
        if let Some(can) = df[i].dtype().can_cast_to(&DataType::String) && can {
            df.try_apply_at_idx(i, |series| {
                let cast = series.str()?;
                cast.set(&cast.equal("?"), None)
            })?;
        }
    };

    df = df.drop_nulls::<String>(None)?;

    df = df.unique_stable(None, UniqueKeepStrategy::First, None)?;

    df = df.lazy().with_column(
        col("income").replace(
            lit(Series::new("from".into(), &["<=50K", ">50K"])),
            lit(Series::new("to".into(), &[0i64, 1i64])),
        ).cast(DataType::Int64)
    ).collect()?;

    Ok(df)
}

/// Verify k-anonymity property.
///
/// # Parameters
/// - `df`: The DataFrame to verify.
/// - `qi_columns`: The list of quasi-identifier column names to consider for grouping.
/// - `k`: The k value for k-anonymity.
pub fn verify_k_anonymity(df: &DataFrame, qi_columns: &[&str], k: usize) -> bool {
    if df.height() == 0 {
        return true;
    }

    let mut groups: HashMap<String, usize> = HashMap::new();

    for row in 0..df.height() {
        let mut key = String::new();
        for &col_name in qi_columns {
            let val = df
                .column(col_name)
                .expect("QI column must exist in DataFrame")
                .as_series()
                .expect("column must be a Series")
                .str()
                .expect("QI column must be String dtype after anonymization")
                .get(row)
                .unwrap_or("null");

            key.push_str(val);
            key.push('|');
        }

        *groups.entry(key).or_insert(0) += 1;
    }

    groups.values().all(|&count| count >= k)
}
