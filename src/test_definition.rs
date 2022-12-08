//! Helper structs to read a YAML test definition file.
//! The structure is defined with structs and parsed with serde_yaml.
use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

/// The root struct of a test defintion file.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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

/// A test dimension that can take multiple values. Often this would be a dependency and its versions,
/// i.e. "zookeeper" and "3.7.0", "3.7.1", "3.8.0" etc.
/// A test case that uses this dimension will be rendered with all of the dimensions values.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Dimension {
    pub name: String,
    pub values: Vec<String>,
}

/// A kuttl test. It has a name and a list of dimensions that it needs.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Test {
    pub name: String,
    pub dimensions: Vec<String>,
}
