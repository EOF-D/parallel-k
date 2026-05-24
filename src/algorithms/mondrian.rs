use std::collections::{HashMap, HashSet};

use polars::prelude::*;

use crate::{
    algorithms::anonymization_algorithm::{AlgorithmError, AnonymizationAlgorithm},
    data::{dataset::Dataset, qi::{QIType, QuasiIdentifier, QuasiIdentifiers}},
};

/// Mondrian k-Anonymity algorithm.
#[derive(Default)]
pub struct Mondrian {}

impl AnonymizationAlgorithm for Mondrian {
    fn name(&self) -> &str {
        "Mondrian"
    }

    /// Anonymize the dataset so that each group of rows sharing the same QIs has at least k rows.
    ///
    /// # Parameters
    /// - `k`: The k in k-anonymity, i.e., the minimum number of rows that must share the same QI values.
    /// - `dataset`: The input dataset to be anonymized.
    /// - `quasi_identifiers`: The list of column names that are considered quasi-identifiers.
    ///
    /// # Returns
    /// - `Ok(Dataset)`: The anonymized dataset.
    /// - `Err(AlgorithmError)`: An error if the anonymization process fails, e.g., if the dataset is empty, etc.
    fn anonymize(
        &self,
        k: u32,
        dataset: Dataset,
        quasi_identifiers: QuasiIdentifiers,
    ) -> Result<Dataset, AlgorithmError> {
        if k < 2 {
            return Err(format!("k must be at least 2, got k = {k}").into());
        }

        let k = k as usize;
        let n = dataset.df.height();

        if k > n {
            let empty = Self::empty_df_like(&dataset.df)?;
            return Ok(Dataset::from_anonymized(empty, quasi_identifiers, dataset.taxonomies));
        }

        let mut leaves: Vec<Vec<usize>> = Vec::new();
        Self::partition(k, &dataset, (0..n).collect(), &mut leaves);

        let output = Self::build_output_df(&dataset, &leaves)?;
        Ok(Dataset::from_anonymized(output, quasi_identifiers, dataset.taxonomies))
    }
}

impl Mondrian {
    /// Recursively partition `rows`.
    ///
    /// Partitions with fewer than `k` rows are suppressed. Partitions where no
    /// allowable cut exists become leaves.
    ///
    /// # Parameters
    /// - `k`: The minimum partition size.
    /// - `dataset`: The dataset containing the DataFrame and QI metadata.
    /// - `rows`: The row indices of the current partition.
    /// - `leaves`: Accumulator of completed leaf partitions.
    fn partition(k: usize, dataset: &Dataset, rows: Vec<usize>, leaves: &mut Vec<Vec<usize>>) {
        if rows.len() < k {
            return;
        }

        if let Some((left, right)) = Self::try_split(k, dataset, &rows) {
            Self::partition(k, dataset, left, leaves);
            Self::partition(k, dataset, right, leaves);
        } else {
            leaves.push(rows);
        }
    }

    /// Find the best allowable cut across all QI dimensions.
    ///
    /// # Parameters
    /// - `k`: The minimum partition size.
    /// - `dataset`: The dataset containing the DataFrame and QI metadata.
    /// - `rows`: The row indices of the current partition to split.
    ///
    /// # Returns
    /// - `Some((left, right))`: Row indices for the left and right sub partitions.
    /// - `None`: No valid cut exists.
    fn try_split(k: usize, dataset: &Dataset, rows: &[usize]) -> Option<(Vec<usize>, Vec<usize>)> {
        let mut spans: Vec<(usize, f64)> = dataset.qis.0
            .iter()
            .enumerate()
            .map(|(i, qi)| (i, Self::normalized_span(dataset, qi, rows)))
            .collect();

        spans.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (dim_idx, _) in spans {
            let qi = &dataset.qis.0[dim_idx];
            if let Some((left, right)) = Self::split_on_dim(dataset, qi, rows)
                && left.len() >= k && right.len() >= k
            {
                return Some((left, right));
            }
        }

        None
    }

    /// Compute the normalized span of `qi` over `rows`.
    ///
    /// # Parameters
    /// - `dataset`: The dataset containing the DataFrame and taxonomy metadata.
    /// - `qi`: The quasi-identifier dimension to measure.
    /// - `rows`: The row indices of the current partition.
    ///
    /// # Returns
    /// - `f64`: The normalized span in `[0.0, 1.0]`, or `0.0` if the span cannot be computed.
    fn normalized_span(dataset: &Dataset, qi: &QuasiIdentifier, rows: &[usize]) -> f64 {
        match &qi.qi_type {
            QIType::Numerical { .. } => {
                let tax = match dataset.taxonomies.numerical_taxonomies.get(&qi.column_name) {
                    Some(t) => t,
                    None => return 0.0,
                };

                let global_range = (tax.max_val - tax.min_val) as f64;
                if global_range == 0.0 {
                    return 0.0;
                }

                let col = match dataset.df.column(&qi.column_name) {
                    Ok(c) => c,
                    Err(_) => return 0.0,
                };

                let ca = match col.as_series().expect("column must be a Series").i64() {
                    Ok(c) => c,
                    Err(_) => return 0.0,
                };

                match Self::numerical_min_max(ca, rows) {
                    Some((low, high)) => (high - low) as f64 / global_range,
                    None => 0.0,
                }
            }

            QIType::Categorical { .. } => {
                let tax = match dataset.taxonomies.categorical_taxonomies.get(&qi.column_name) {
                    Some(t) => t,
                    None => return 0.0,
                };

                let total_leaves = tax.nodes.values().filter(|n| n.level == 0).count();
                if total_leaves == 0 {
                    return 0.0;
                }

                let col = match dataset.df.column(&qi.column_name) {
                    Ok(c) => c,
                    Err(_) => return 0.0,
                };

                let ca = match col.as_series().expect("column must be a Series").str() {
                    Ok(c) => c,
                    Err(_) => return 0.0,
                };

                let distinct: HashSet<&str> = rows.iter().filter_map(|&r| ca.get(r)).collect();
                distinct.len() as f64 / total_leaves as f64
            }
        }
    }

    /// Split `rows` on `qi`, returning `(left, right)` or `None` if impossible.
    ///
    /// # Parameters
    /// - `dataset`: The dataset containing the DataFrame and QI metadata.
    /// - `qi`: The quasi-identifier dimension to split on.
    /// - `rows`: The row indices of the current partition.
    ///
    /// # Returns
    /// - `Some((left, right))`: Row indices for the left and right sub partitions.
    /// - `None`: The dimension cannot be split (e.g., only one distinct value).
    fn split_on_dim(
        dataset: &Dataset,
        qi: &QuasiIdentifier,
        rows: &[usize],
    ) -> Option<(Vec<usize>, Vec<usize>)> {
        match &qi.qi_type {
            QIType::Numerical { .. } => {
                let col = dataset.df.column(&qi.column_name).ok()?;
                let ca = col.as_series().expect("column must be a Series").i64().ok()?;

                let mut vals: Vec<i64> = rows.iter().filter_map(|&r| ca.get(r)).collect();
                vals.sort_unstable();

                if vals.is_empty() {
                    return None;
                }

                let median = vals[(vals.len() - 1) / 2];

                let mut left = Vec::new();
                let mut right = Vec::new();
                for &r in rows {
                    match ca.get(r) {
                        Some(v) if v <= median => left.push(r),
                        Some(_) => right.push(r),
                        None => {}
                    }
                }

                if left.is_empty() || right.is_empty() { None } else { Some((left, right)) }
            }

            QIType::Categorical { .. } => {
                let col = dataset.df.column(&qi.column_name).ok()?;
                let ca = col.as_series().expect("column must be a Series").str().ok()?;

                let distinct: Vec<String> = {
                    let mut set = HashSet::new();
                    for &r in rows {
                        if let Some(v) = ca.get(r) {
                            set.insert(v.to_string());
                        }
                    }

                    let mut v: Vec<String> = set.into_iter().collect();
                    v.sort();
                    v
                };

                if distinct.len() < 2 {
                    return None;
                }

                let mid = distinct.len() / 2;
                let right_set: HashSet<&str> = distinct[mid..].iter().map(String::as_str).collect();

                let mut left = Vec::new();
                let mut right = Vec::new();
                for &r in rows {
                    match ca.get(r) {
                        Some(v) if right_set.contains(v) => right.push(r),
                        Some(_) => left.push(r),
                        None => {}
                    }
                }

                if left.is_empty() || right.is_empty() { None } else { Some((left, right)) }
            }
        }
    }

    /// Build the anonymized output DataFrame from the completed leaf partition set.
    ///
    /// # Parameters
    /// - `dataset`: The original dataset containing the DataFrame and QI metadata.
    /// - `leaves`: The completed leaf partitions produced by `Self::partition`.
    ///
    /// # Returns
    /// - `Ok(DataFrame)`: The anonymized DataFrame with generalized QI columns.
    /// - `Err(AlgorithmError)`: An error if column access or DataFrame construction fails.
    fn build_output_df(dataset: &Dataset, leaves: &[Vec<usize>]) -> Result<DataFrame, AlgorithmError> {
        if leaves.is_empty() {
            return Self::empty_df_like(&dataset.df);
        }

        let mut row_order: Vec<(usize, usize)> = leaves
            .iter()
            .enumerate()
            .flat_map(|(leaf_idx, rows)| rows.iter().map(move |&r| (r, leaf_idx)))
            .collect();

        row_order.sort_by_key(|&(orig, _)| orig);

        let mut labels: HashMap<(usize, usize), String> = HashMap::new();
        for (leaf_idx, rows) in leaves.iter().enumerate() {
            for (qi_idx, qi) in dataset.qis.0.iter().enumerate() {
                labels.insert((leaf_idx, qi_idx), Self::generalize_leaf(dataset, qi, rows)?);
            }
        }

        let total = row_order.len();
        let mut out_columns: Vec<Column> = Vec::new();

        for col in dataset.df.columns() {
            let col_name = col.name().to_string();
            if let Some(qi_idx) = dataset.qis.0.iter().position(|q| q.column_name == col_name) {
                let values: Vec<String> = row_order
                    .iter()
                    .map(|&(_, leaf_idx)| {
                        labels
                            .get(&(leaf_idx, qi_idx))
                            .cloned()
                            .expect("every (leaf, qi) pair has a label")
                    })
                    .collect();

                out_columns.push(Series::new(col_name.as_str().into(), values).into());
            } else {
                let series = col.as_series().expect("column must be a Series");
                let idx: Vec<IdxSize> = row_order.iter().map(|&(r, _)| r as IdxSize).collect();
                let gathered = series.take(&IdxCa::from_vec("".into(), idx))?;

                out_columns.push(gathered.into_column());
            }
        }

        Ok(DataFrame::new(total, out_columns)?)
    }

    /// Compute the generalized label for `qi` over a single leaf partition.
    ///
    /// # Parameters
    /// - `dataset`: The dataset containing the DataFrame and taxonomy metadata.
    /// - `qi`: The quasi-identifier dimension to generalize.
    /// - `rows`: The row indices of the leaf partition.
    ///
    /// # Returns
    /// - `Ok(String)`: The generalized label for the partition.
    /// - `Err(AlgorithmError)`: An error if the taxonomy is missing or LCA cannot be found.
    fn generalize_leaf(
        dataset: &Dataset,
        qi: &QuasiIdentifier,
        rows: &[usize],
    ) -> Result<String, AlgorithmError> {
        match &qi.qi_type {
            QIType::Numerical { .. } => {
                let col = dataset.df.column(&qi.column_name)?;
                let ca = col.as_series().expect("column must be a Series").i64()?;

                match Self::numerical_min_max(ca, rows) {
                    Some((low, high)) if low == high => Ok(format!("{low}")),
                    Some((low, high)) => Ok(format!("{low}-{high}")),
                    None => Ok("*".into()),
                }
            }

            QIType::Categorical { .. } => {
                let tax = dataset.taxonomies
                    .categorical_taxonomies
                    .get(&qi.column_name)
                    .ok_or_else(|| format!("no taxonomy for column '{}'", qi.column_name))?;

                let col = dataset.df.column(&qi.column_name)?;
                let ca = col.as_series().expect("column must be a Series").str()?;

                let distinct: Vec<&str> = rows
                    .iter()
                    .filter_map(|&r| ca.get(r))
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();

                tax.find_lca(&distinct).ok_or_else(|| {
                    format!("find_lca returned None for column '{}' values {distinct:?}", qi.column_name).into()
                })
            }
        }
    }

    /// Compute the min and max of a numerical column over a subset of rows.
    ///
    /// # Parameters
    /// - `ca`: The chunked array to scan.
    /// - `rows`: The row indices to consider.
    ///
    /// # Returns
    /// - `Some((min, max))`: The inclusive range of values found.
    /// - `None`: All rows were null or `rows` is empty.
    fn numerical_min_max(ca: &Int64Chunked, rows: &[usize]) -> Option<(i64, i64)> {
        let mut low = i64::MAX;
        let mut high = i64::MIN;

        for &r in rows {
            if let Some(v) = ca.get(r) {
                low = low.min(v);
                high = high.max(v);
            }
        }

        if low > high { None } else { Some((low, high)) }
    }

    /// Return an empty DataFrame with the same column names and dtypes as `df`.
    ///
    /// # Parameters
    /// - `df`: The source DataFrame whose schema is copied.
    ///
    /// # Returns
    /// - `Ok(DataFrame)`: An empty DataFrame with identical schema.
    /// - `Err(AlgorithmError)`: An error if DataFrame construction fails.
    fn empty_df_like(df: &DataFrame) -> Result<DataFrame, AlgorithmError> {
        let cols: Vec<Column> = df
            .columns()
            .iter()
            .map(|c| Series::new_empty(c.name().clone(), c.dtype()).into_column())
            .collect();

        Ok(DataFrame::new(0, cols)?)
    }
}
