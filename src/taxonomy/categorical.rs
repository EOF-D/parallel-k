use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/**
   Categorical taxanomy Stucts
   and impl
*/
#[derive(Clone, Debug)]
pub struct CategoricalNode {
   pub id: String,
   pub value: String,
   pub level: usize,
   pub children: Vec<String>,
   pub parent: Option<String>
}

#[derive(Clone, Debug)]
pub struct CategoricalTaxonomy {
   pub nodes: HashMap<String, CategoricalNode>,
   pub col_name: String,
   pub root_id: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoricalHierarchy {
   pub col_name: String,
   pub children: Vec<CategoricalHierarchy>,
}

impl CategoricalHierarchy {
    pub fn new(col_name: &str) -> Self {
        Self {
            col_name: col_name.to_string(),
            children: vec![],
        }
    }

    pub fn with_children(mut self, children: Vec<CategoricalHierarchy>) -> Self {
        self.children = children;
        self
    }
}

impl CategoricalTaxonomy {
    pub fn create_from_hierarchy(
        col_name: &str,
        hierarchy: &CategoricalHierarchy,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut nodes = HashMap::new();

        let root_id = Self::build_tree(
            &mut nodes,
            col_name,
            hierarchy,
            None,
            0
        );

        let max_level = nodes.values().map(|n| n.level).max().unwrap_or(0);
        for node in nodes.values_mut() {
            node.level = max_level - node.level;
        }

        Ok(Self {
            nodes,
            col_name: col_name.to_string(),
            root_id,
        })
    }

    // Recursive build with based on hierarchy def
    pub fn build_tree(
        nodes: &mut HashMap<String, CategoricalNode>,
        col_name: &str,
        hierarchy: &CategoricalHierarchy,
        parent_id: Option<String>,
        level: usize
    ) -> String {
        let node_id = format!("{}_{}_{}", col_name, level, hierarchy.col_name);

        let children_node_ids: Vec<String> = hierarchy
            .children
            .iter()
            .map(|child| {
                Self::build_tree(
                    nodes,
                    col_name,
                    child,
                    Some(node_id.clone()),
                    level + 1
                )
            })
            .collect();
        
        let node = CategoricalNode {
            id: node_id.clone(),
            value: hierarchy.col_name.clone(),
            level,
            children: children_node_ids,
            parent: parent_id,
        };
    
        nodes.insert(node_id.clone(), node);
        node_id
    }

    /// Find the lowest common ancestor of the given values in this taxonomy.
    ///
    /// # Parameters
    /// - `values`: A slice of string values to find the LCA for.
    ///
    /// # Returns
    /// - `Some(String)`: The value of the lowest common ancestor if found.
    /// - `None`: If the input slice is empty or if any value is not found.
    pub fn find_lca(&self, values: &[&str]) -> Option<String> {
        if values.is_empty() {
            return None;
        }

        // For each value, find its node and collect all ancestor node IDs.
        let mut ancestor_sets: Vec<std::collections::HashSet<String>> = Vec::new();

        for &value in values {
            // Find the node whose value field matches.
            let start_node = self.nodes.values().find(|n| n.value == value)?;

            let mut ancestors: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut current_id = start_node.id.clone();

            loop {
                ancestors.insert(current_id.clone());
                let current_node = self.nodes.get(&current_id)
                    .expect("every node id stored in the tree must exist in the nodes map");

                match &current_node.parent {
                    Some(parent_id) => current_id = parent_id.clone(),
                    None => break,
                }
            }

            ancestor_sets.push(ancestors);
        }

        // Intersect all ancestor sets to find common ancestors.
        let common: std::collections::HashSet<String> = ancestor_sets
            .into_iter()
            .reduce(|acc, set| acc.intersection(&set).cloned().collect())?;

        // LCA is the common ancestor with the minimum level.
        let lca_id = common
            .into_iter()
            .min_by_key(|id| {
                self.nodes.get(id)
                    .expect("every id in ancestor set must exist in nodes map")
                    .level
            })?;

        Some(
            self.nodes.get(&lca_id)
                .expect("lca_id must exist in nodes map")
                .value
                .clone(),
        )
    }

    pub fn print_categorical_taxanomy_tree(&self) {
        if let Some(root_node) = self.nodes.get(&self.root_id) {
            self.print_catgeorical_taxonomy_recurse(&self.root_id, root_node, 0);
        } else {
            for (id, node) in self.nodes.iter().filter(|(_, n)| n.level == 0) {
                self.print_catgeorical_taxonomy_recurse(id, node, 0);
            }
        }
    }

    pub fn print_catgeorical_taxonomy_recurse(&self, id: &String, node: &CategoricalNode, depth: usize) {
        let indent = "  ".repeat(depth);

        println!("{}[Level {}] {} (ID: {})", indent, node.level, node.value, id);

        for child_id in &node.children {
            if let Some(child_node) = self.nodes.get(child_id) {
                self.print_catgeorical_taxonomy_recurse(child_id, child_node, depth + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_taxonomy() -> CategoricalTaxonomy {
        let hierarchy = CategoricalHierarchy {
            col_name: "Root".to_string(),
            children: vec![
                CategoricalHierarchy {
                    col_name: "Mid1".to_string(),
                    children: vec![
                        CategoricalHierarchy::new("LeafA"),
                        CategoricalHierarchy::new("LeafB"),
                    ],
                },
                CategoricalHierarchy {
                    col_name: "Mid2".to_string(),
                    children: vec![
                        CategoricalHierarchy::new("LeafC"),
                    ],
                },
            ],
        };

        CategoricalTaxonomy::create_from_hierarchy("col", &hierarchy)
            .expect("test taxonomy must build without error")
    }

    #[test]
    fn test_find_lca_single_value_returns_self() {
        let tax = build_test_taxonomy();
        assert_eq!(tax.find_lca(&["LeafA"]), Some("LeafA".to_string()));
        assert_eq!(tax.find_lca(&["Root"]), Some("Root".to_string()));
        assert_eq!(tax.find_lca(&["Mid1"]), Some("Mid1".to_string()));
    }

    #[test]
    fn test_find_lca_two_siblings_returns_parent() {
        let tax = build_test_taxonomy();

        // LeafA and LeafB share parent Mid1.
        assert_eq!(tax.find_lca(&["LeafA", "LeafB"]), Some("Mid1".to_string()));
    }

    #[test]
    fn test_find_lca_all_leaves_returns_root() {
        let tax = build_test_taxonomy();
        assert_eq!(tax.find_lca(&["LeafA", "LeafB", "LeafC"]), Some("Root".to_string()));
    }

    #[test]
    fn test_find_lca_empty_returns_none() {
        let tax = build_test_taxonomy();
        assert_eq!(tax.find_lca(&[]), None);
    }

    #[test]
    fn test_find_lca_unknown_value_returns_none() {
        let tax = build_test_taxonomy();
        assert_eq!(tax.find_lca(&["None"]), None);
        assert_eq!(tax.find_lca(&["LeafA", "Random"]), None);
    }
}
