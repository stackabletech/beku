use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};

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

impl Test {
    pub fn test_dir(&self, base_dir: &str) -> Box<std::path::Path> {
        let path: std::path::PathBuf = vec![base_dir, &self.name].iter().collect();
        path.into_boxed_path()
    }

    pub fn generate_scenarios(
        &self,
        dimensions: &HashMap<String, Vec<String>>,
    ) -> Vec<TestScenario> {
        let combinations: Vec<Vec<String>> = self
            .dimensions
            .iter()
            .map(|d| dimensions.get(d).unwrap().clone())
            .multi_cartesian_product()
            .collect();

        let mut scenarios = Vec::new();
        for c in combinations {
            let values: HashMap<String, String> = self
                .dimensions
                .clone()
                .into_iter()
                .zip(c.clone().into_iter())
                .collect();
            let scenario = TestScenario::new(self.name.clone(), values);
            scenarios.push(scenario);
        }
        scenarios
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestScenario {
    test_name: String,
    pub values: HashMap<String, String>,
}

impl TestScenario {
    pub fn new(test_name: String, values: HashMap<String, String>) -> Self {
        Self { test_name, values }
    }

    pub fn scenario_dir(&self, base_dir: &str) -> Box<std::path::Path> {
        let path: std::path::PathBuf = vec![base_dir, &self.scenario_name()].iter().collect();
        path.into_boxed_path()
    }

    pub fn scenario_name(&self) -> String {
        let mut parts = vec![self.test_name.clone()];
        for (k, v) in self.values.iter() {
            parts.push(format!("_{}-{}", k, v));
        }
        parts.join("")
    }
}

impl Display for TestScenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TODO")
    }
}
