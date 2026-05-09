use parallel_k::{
    algorithms::{
        incognito::Incognito, mondrian::Mondrian
    }, 
    anonymize::Anonymizer,
    data::{
        dataset::Dataset,
        qi::QuasiIdentifiers
    }
};



#[path = "./util/util.rs"]
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let df = util::clean_adult_csv("adult.csv".into())?;

    let anonymizer = Anonymizer::new(
        Dataset::from_dataframe(df),
        QuasiIdentifiers::from_column_names(&["age", "workclass"])
    );

    let result = anonymizer.run_single::<Mondrian>(5)?;
    println!("{}", result.anonymized_dataset.df);

    let result = anonymizer.run_single::<Incognito>(5)?;
    println!("{}", result.anonymized_dataset.df);

    let result = anonymizer.run_parallel::</*probably need a macro*/>(5..10)?;
    println!("{}", result.anonymized_dataset.df);


    Ok(())
}
