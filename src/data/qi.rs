#[derive(Clone)]
pub struct QuasiIdentifier {
    pub column_name: std::string::String,
}

#[derive(Clone)]
pub struct QuasiIdentifiers(pub Vec<QuasiIdentifier>/* or maybe a polars series?*/);
impl QuasiIdentifiers {
    pub fn from_csv() -> Self { todo!() }
    pub fn from_column_names(_names: &[&str]) -> Self { todo!() }
}
