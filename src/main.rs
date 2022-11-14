use serde::{Serialize, Deserialize};
use minijinja::{Environment, Source, context};
use std::fmt::{self, Display};
use std::collections::HashMap;
use std::fs::DirEntry;
use std::path::PathBuf;
use itertools::Itertools;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TestDefinition {
    dimensions: Vec<Dimension>,
    tests: Vec<Test>
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
struct Dimension {
    name: String,
    values: Vec<String>
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Test {
    name: String,
    dimensions: Vec<String>
}

impl Test {
    pub fn test_dir(&self, base_dir: &str) -> Box<std::path::Path> {
        let path: std::path::PathBuf = vec![base_dir, &self.name].iter().collect();
        path.into_boxed_path()
    }

    pub fn generate_scenarios(&self, dimensions: &HashMap<String, Vec<String>>) -> Vec<TestScenario> {
        let combinations: Vec<Vec<String>> = self.dimensions.iter()
            .map(|d| dimensions.get(d).unwrap().clone())
            .multi_cartesian_product().collect();
        
        let mut scenarios = Vec::new();
        for c in combinations {
            let values: HashMap<String, String> = self.dimensions.clone().into_iter().zip(c.clone().into_iter()).collect();
            let scenario = TestScenario::new(self.name.clone(), values);
            scenarios.push(scenario);
        }
        scenarios
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TestScenario {
    test_name: String,
    values: HashMap<String, String>
}

impl TestScenario {
    pub fn new(test_name: String, values: HashMap<String, String>) -> Self {
        Self {
            test_name,
            values
        }
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

fn main() {
    let file_contents = std::fs::read_to_string("test-definition.yaml").unwrap();
    let test_def: TestDefinition = serde_yaml::from_str(&file_contents).unwrap();
    let dimensions = test_def.dimensions_as_map();
    let base_dir = "tests";
    std::fs::create_dir(base_dir);
    for test in test_def.tests.iter() {
        let test_dir = test.test_dir(base_dir);
        // load templates for the test
        let mut source = Source::new();
        let mut files_to_template: Vec<String> = Vec::new();
        let mut files_to_copy: Vec<DirEntry> = Vec::new();
        for entry in std::fs::read_dir(vec!["templates", "kuttl", &test.name].join("/")).unwrap() {
            let entry = entry.unwrap();
            if entry.path().to_string_lossy().ends_with(".j2") {
                let template = std::fs::read_to_string(entry.path()).unwrap();
                let x = entry.file_name();
                let y = x.to_str();
                let z = y.unwrap();
                let a = z.to_string();
                let template_name = a.strip_suffix(".j2").unwrap();
                source.add_template(template_name, &template).unwrap();
                files_to_template.push(template_name.to_string());
            } else {
                files_to_copy.push(entry);
            }
        }
        let mut env = Environment::new();
        env.set_source(source);
        // create the files
        std::fs::create_dir(test_dir.clone());
        // all scenarios
        for scenario in test.generate_scenarios(&dimensions).iter() {
            let scenario_dir = scenario.scenario_dir(&test_dir.to_str().unwrap());
            std::fs::create_dir(scenario_dir.clone());
            for file in files_to_copy.iter() {
                let new_file_name = scenario_dir.join(file.file_name());
                std::fs::copy(file.path(), new_file_name);
            }
            for file in files_to_template.iter() {
                println!("{:?}", scenario);
                let rendered_file_content = env.get_template(&file)
                    .unwrap().render(context!(test_scenario => scenario)).unwrap();
                std::fs::write(scenario_dir.join(file), rendered_file_content).unwrap();
            }
        }
    }
}
