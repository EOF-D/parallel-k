use std::collections::HashMap;

/**
    Numerical taxanomy Stucts
    and impl
*/
#[derive(Clone, Debug)]
pub struct NumericalNode {
    pub id: String,
    pub range: (i64, i64),
    pub level: usize,
    pub children: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NumericalTaxonomy {
    pub nodes: HashMap<String, NumericalNode>,
    // These min and max will be for
    // dynamically creating the intervals
    pub min_val: i64,
    pub max_val: i64,
    pub col_name: String
}

impl NumericalTaxonomy {
    pub fn create_from_data_range(
        col_name: &str,
        min_val: i64,
        max_val: i64,
        leaf_bucket_size: i64,
        grouping_factor: usize,
    ) -> Self {
        let mut nodes = HashMap::new();

        // building bottom-up
        let leaf_nodes = Self::create_leaf_level(
            &mut nodes,
            col_name,
            min_val,
            max_val,
            leaf_bucket_size,
        );

        let mut current_level_nodes = leaf_nodes;
        let mut current_level = 0;

        while current_level_nodes.len() > 1 {
            current_level += 1;

            current_level_nodes = Self::create_parent_level(
                &mut nodes,
                col_name,
                &current_level_nodes,
                grouping_factor,
                current_level
            )
        }

        if let Some(root_id) = current_level_nodes.first() {
            if let Some(root) = nodes.get_mut(root_id) {
                root.level = current_level + 1;
            }
        }
        Self {
            nodes,
            min_val,
            max_val,
            col_name: col_name.to_string(),
        }
    }

    pub fn create_leaf_level(
        nodes: &mut HashMap<String, NumericalNode>,
        col_name: &str,
        min_val: i64,
        max_val: i64,
        bucket_size: i64,
    ) -> Vec<String> {
        let mut leaf_node_ids = Vec::new();
        let mut current = min_val;
        while current <= max_val {
            let bucket_end = (current + bucket_size - 1).min(max_val);
            let id = format!("{}_{}_{}_{}", col_name, 0, current, bucket_end);
            leaf_node_ids.push(id.clone());
            nodes.insert(
                id,
                NumericalNode {
                    id: leaf_node_ids.last().unwrap().clone(),
                    range: (current, bucket_end),
                    level: 0,
                    children: vec![],
                    parent: None,
                },
            );
            current = bucket_end + 1;
        }
        leaf_node_ids
    }

    pub fn create_parent_level(
        nodes: &mut HashMap<String, NumericalNode>,
        col_name: &str,
        child_ids: &[String],
        group_size: usize,
        level: usize,
    ) -> Vec<String> {
        let mut parent_ids = Vec::new();
        for chunk in child_ids.chunks(group_size) {
            if chunk.is_empty() {
                continue;
            }
            // Get min and max from children
            let min_range = nodes[&chunk[0]].range.0;
            let max_range = nodes[&chunk[chunk.len() - 1]].range.1;
            let parent_id = format!("{}_{}_{}_{}", col_name, level, min_range, max_range);
            parent_ids.push(parent_id.clone());
            // Update children to point to this parent
            for child_id in chunk {
                if let Some(child) = nodes.get_mut(child_id) {
                    child.parent = Some(parent_id.clone());
                }
            }
            // Create parent node
            nodes.insert(
                parent_id,
                NumericalNode {
                    id: parent_ids.last().unwrap().clone(),
                    range: (min_range, max_range),
                    level,
                    children: chunk.iter().cloned().collect(),
                    parent: None,
                },
            );
        }
        parent_ids
    }

    pub fn print_numerical_taxanomy_tree(&self){
        println!("{:?}", self);
    }
}



