//! Helper structs to read a YAML test definition
use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

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

    pub fn from_yaml_file(yaml_file: &Path) -> Result<TestDefinition> {
        let file_contents =
            std::fs::read_to_string(yaml_file).context("Failed to read test definition file")?;
        let test_def: TestDefinition = serde_yaml::from_str(&file_contents).unwrap();
        Ok(test_def)
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
