use polars::prelude::*;

#[derive(Clone)]
pub struct Dataset {
    pub df: DataFrame
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

}
