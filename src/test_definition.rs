//! Helper structs to read a YAML test definition
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestDefinition {
    pub dimensions: Vec<Dimension>,
    pub tests: Vec<Test>,
}

impl TestDefinition {
    pub fn dimensions_as_map(&self) -> HashMap<String, Vec<String>> {
        let mut hashmap = HashMap::new();
        for dimension in self.dimensions.iter() {
            hashmap.insert(dimension.name.clone(), dimension.values.clone());
        }
        hashmap
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Dimension {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Test {
    pub name: String,
    pub dimensions: Vec<String>,
}
