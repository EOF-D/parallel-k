use std::fs::File;

use polars::prelude::*;

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

    Ok(())
}
