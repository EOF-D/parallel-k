#[derive(Clone, Debug)]
pub struct QuasiIdentifier {
    pub column_name: std::string::String,
    pub qi_type: QIType
}

#[derive(Clone, Debug)]
pub enum QIType {
    Numerical { leaf_bucket_size: i64 },
    Categorical { path_to_json_hierarchy: String }
}

#[derive(Clone)]
pub struct QuasiIdentifiers(pub Vec<QuasiIdentifier>/* or maybe a polars series?*/);
impl QuasiIdentifiers {
    pub fn from_csv() -> Self { todo!() }
    
    pub fn from_column_names(names: &[(&str, QIType)]) -> Self { 
        let qis = names.iter()
            .map(|(name, qi_type)| QuasiIdentifier {
                column_name: name.to_string(),
                qi_type: qi_type.clone(),
            })
            .collect();
        Self(qis)
    }
}