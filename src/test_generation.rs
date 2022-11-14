use std::{collections::HashMap, path::Path};

use minijinja::{context, Environment, Template};
use serde::Serialize;

use crate::test_definition::TestDefinition;
use anyhow::{Ok, Result};
use itertools::Itertools;

pub struct TestGeneration {
    tests: Vec<Test>,
}

impl TestGeneration {
    pub fn new(out_dir: &Path, template_base_dir: &Path, test_defintion: TestDefinition) -> Self {
        let dimension_values = test_defintion.dimensions_as_map();
        let mut tests = Vec::new();
        for test_def in test_defintion.tests.iter() {
            let mut test = Test::new(out_dir, &template_base_dir, &test_def.name);
            test.add_scenarios_for_values(&test_def.dimensions, &dimension_values);
            tests.push(test);
        }
        Self { tests }
    }

    pub fn generate(&self) -> Result<()> {
        for test in self.tests.iter() {
            test.generate()?;
        }
        Ok(())
    }
}

struct Test {
    out_dir: Box<Path>,
    template_dir: Box<Path>,
    test_name: String,
    test_scenerarios: Vec<TestScenario>,
}

impl Test {
    pub fn new(out_base_dir: &Path, template_base_dir: &Path, test_name: &str) -> Self {
        Self {
            out_dir: out_base_dir.join(test_name).into(),
            template_dir: template_base_dir.join(test_name).into(),
            test_name: test_name.to_string(),
            test_scenerarios: Vec::new(),
        }
    }

    fn add_scenarios_for_values(
        &mut self,
        used_dimensions: &Vec<String>,
        dimension_values: &HashMap<String, Vec<String>>,
    ) {
        let combinations: Vec<Vec<String>> = used_dimensions
            .iter()
            .map(|d| dimension_values.get(d).unwrap().clone())
            .multi_cartesian_product()
            .collect();

        for c in combinations {
            let values: HashMap<String, String> = used_dimensions
                .clone()
                .into_iter()
                .zip(c.clone().into_iter())
                .collect();
            let scenario = TestScenario::new(&self.out_dir, &self.test_name, values);
            self.test_scenerarios.push(scenario);
        }
    }

    fn generate(&self) -> Result<()> {
        std::fs::create_dir(self.out_dir.clone())?;
        for s in self.test_scenerarios.iter() {
            s.create_base_directory()?;
        }
        for entry in walkdir::WalkDir::new(self.template_dir.clone()) {
            let entry = entry.unwrap(); // TODO fix unwrap
            let no_prefix_path = entry.path().strip_prefix(self.template_dir.clone())?;
            if no_prefix_path.to_string_lossy().is_empty() {
                continue;
            }
            if entry.path().is_dir() {
                for s in self.test_scenerarios.iter() {
                    s.create_directory(&no_prefix_path)?;
                }
            } else if entry.path().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.ends_with(".j2") {
                    let template = std::fs::read_to_string(entry.path())?;
                    let rendered_filename = no_prefix_path
                        .to_str()
                        .unwrap()
                        .strip_suffix(".j2")
                        .unwrap()
                        .to_string();
                    let mut env = Environment::new();
                    env.add_template(&rendered_filename, &template)?;
                    let template = env.get_template(&rendered_filename)?;
                    for s in self.test_scenerarios.iter() {
                        s.render_template(&template, Path::new(&rendered_filename))?;
                    }
                } else {
                    for s in self.test_scenerarios.iter() {
                        s.copy_file(entry.path(), no_prefix_path)?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct TestScenario {
    out_dir: Box<Path>,
    values: HashMap<String, String>,
}

impl TestScenario {
    pub fn new(
        test_base_dir: &Box<Path>,
        test_name: &str,
        dimension_values: HashMap<String, String>,
    ) -> Self {
        let mut parts = vec![test_name.to_string()];
        for (k, v) in dimension_values.iter() {
            let rest = format!("_{}-{}", k, v);
            parts.push(rest);
        }
        let scenario_dir_name = parts.join("");
        Self {
            out_dir: test_base_dir.join(scenario_dir_name).into(),
            values: dimension_values,
        }
    }

    pub fn create_base_directory(&self) -> Result<()> {
        std::fs::create_dir(self.out_dir.clone())?;
        Ok(())
    }

    pub fn create_directory(&self, path: &Path) -> Result<()> {
        std::fs::create_dir(self.out_dir.join(path))?;
        Ok(())
    }

    pub fn copy_file(&self, full_source_path: &Path, partial_target_path: &Path) -> Result<()> {
        std::fs::copy(full_source_path, self.out_dir.join(partial_target_path))?;
        Ok(())
    }

    pub fn render_template(&self, template: &Template, partial_output_path: &Path) -> Result<()> {
        let rendered = template.render(context!(test_scenario => self))?;
        std::fs::write(self.out_dir.join(partial_output_path), rendered)?;
        Ok(())
    }
}
