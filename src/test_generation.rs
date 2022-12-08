//! Generates the kuttl test directories. The directory structure is:
//! root_dir/
//!   test_dir/
//!     test_scenario1/
//!     test_scenario2/
//!     ...
//!   test2/
//!     ...
use crate::test_definition::TestDefinition;
use anyhow::{Ok, Result};
use itertools::Itertools;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

pub struct TestGeneration {
    tests: Vec<Test>,
    kuttl_test: PathBuf,
    out_dir: PathBuf,
}

impl TestGeneration {
    /// out_dir: Where to generate the tests. For every named test a directory is created with
    /// the individual scenarios inside.
    pub fn new(
        out_dir: &Path,
        template_base_dir: &Path,
        test_definition: TestDefinition,
        kuttl_test: &Path,
    ) -> Self {
        let dimension_values = test_definition.dimensions_as_map();
        let mut tests = Vec::new();
        let mut tests_out_dir = PathBuf::from(out_dir);
        tests_out_dir.push(Path::new("tests"));
        for test_def in test_definition.tests.iter() {
            let mut test = Test::new(tests_out_dir.as_path(), template_base_dir, &test_def.name);
            test.add_scenarios_for_values(&test_def.dimensions, &dimension_values);
            tests.push(test);
        }
        Self {
            tests,
            kuttl_test: kuttl_test.into(),
            out_dir: out_dir.into(),
        }
    }

    pub fn generate(&self) -> Result<()> {
        std::fs::create_dir_all(self.out_dir.as_path())?;
        self.generate_kuttl_file()?;
        for test in self.tests.iter() {
            test.generate()?;
        }
        Ok(())
    }

    pub fn generate_kuttl_file(&self) -> Result<()> {
        let mut tera = Tera::default();
        tera.add_template_file(self.kuttl_test.as_path(), Some("kuttl-test"))?;

        let mut context = Context::new();
        let test_names: Vec<String> = self.tests.iter().map(|t| t.test_name.clone()).collect();
        context.insert("test_names", &test_names);

        let mut out_file_name = self.out_dir.clone();
        out_file_name.push("kuttl-test.yaml");
        let out_file = std::fs::File::create(out_file_name)?;
        tera.render_to("kuttl-test", &context, out_file)?;
        Ok(())
    }
}

/// A test. It contains the individual scenarios. They are all identical except for the
/// dimensions that the test uses. The cartesian product of the dimensions is created,
/// and for every combination a test scenario is created.
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
        used_dimensions: &[String],
        dimension_values: &HashMap<String, Vec<String>>,
    ) {
        let combinations: Vec<Vec<String>> = used_dimensions
            .iter()
            .map(|d| dimension_values.get(d).unwrap().clone())
            .multi_cartesian_product()
            .collect();

        for c in combinations {
            let values: HashMap<String, String> = used_dimensions
                .to_owned()
                .clone()
                .into_iter()
                .zip(c.clone().into_iter())
                .collect();
            let scenario = TestScenario::new(&self.out_dir, &self.test_name, values);
            self.test_scenerarios.push(scenario);
        }
    }

    fn generate(&self) -> Result<()> {
        // create the directory for this test and all the scenario directories inside
        std::fs::create_dir_all(self.out_dir.clone())?;
        for s in self.test_scenerarios.iter() {
            s.create_base_directory()?;
        }
        let mut tera = Tera::default();
        for entry in walkdir::WalkDir::new(self.template_dir.clone()) {
            let entry = entry.unwrap(); // TODO fix unwrap
            let no_prefix_path = entry.path().strip_prefix(self.template_dir.clone())?;
            if no_prefix_path.to_string_lossy().is_empty() {
                continue;
            }
            if entry.path().is_dir() {
                for s in self.test_scenerarios.iter() {
                    s.create_directory(no_prefix_path)?;
                }
            } else if entry.path().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.ends_with(".j2") {
                    tera.add_template_file(entry.path(), None)?;
                    let rendered_filename = no_prefix_path
                        .to_str()
                        .unwrap()
                        .strip_suffix(".j2")
                        .unwrap()
                        .to_string();
                    for s in self.test_scenerarios.iter() {
                        s.render_template(
                            &tera,
                            &entry.path().to_string_lossy(),
                            Path::new(&rendered_filename),
                        )?;
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

/// A test scenario is a particular instance of a test, with a specific set of dimension values.
struct TestScenario {
    out_dir: Box<Path>,
    context: Context,
}

impl TestScenario {
    pub fn new(
        test_base_dir: &Path,
        test_name: &str,
        dimension_values: HashMap<String, String>,
    ) -> Self {
        let mut parts = vec![test_name.to_string()];
        for (k, v) in dimension_values.iter() {
            let rest = format!("_{}-{}", k, v);
            parts.push(rest);
        }
        let scenario_dir_name = parts.join("");
        let mut context = Context::new();
        for (k, v) in dimension_values.iter() {
            context.insert(k, &v);
        }
        context.insert(
            "test_scenario",
            &HashMap::from([("values", dimension_values)]),
        );
        Self {
            out_dir: test_base_dir.join(scenario_dir_name).into(),
            context,
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

    pub fn render_template(
        &self,
        tera: &Tera,
        template_name: &str,
        partial_output_path: &Path,
    ) -> Result<()> {
        let out_file = std::fs::File::create(self.out_dir.join(partial_output_path))?;
        tera.render_to(template_name, &self.context, out_file)?;
        Ok(())
    }
}
