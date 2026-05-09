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
