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

    // pub fn find_lca(&self, values: &[String]) -> Option<String> {
    //     // to-do
    //     None
    // }

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



