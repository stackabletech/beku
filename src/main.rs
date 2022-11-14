mod test_definition;
use anyhow::{Context, Ok, Result};
use minijinja::{context, Environment, Source};
use std::{fs::DirEntry, path::PathBuf};
use test_definition::TestDefinition;

fn main() -> Result<()> {
    // read the test definition file
    let file_contents = std::fs::read_to_string("test-definition.yaml")
        .context("Failed to read test definition file")?;
    let test_def: TestDefinition = serde_yaml::from_str(&file_contents).unwrap();
    let dimensions = test_def.dimensions_as_map();
    let templates_dir: PathBuf = ["templates", "kuttl"].iter().collect();
    let base_dir = "tests"; // the base output directory
    std::fs::create_dir(base_dir)
        .with_context(|| format!("Failed to create directory: {}", base_dir))?;
    // generate output
    for test in test_def.tests.iter() {
        let test_dir = test.test_dir(base_dir);
        // load templates for the test
        let mut source = Source::new();
        let mut files_to_template: Vec<String> = Vec::new();
        let mut files_to_copy: Vec<DirEntry> = Vec::new();
        let test_source_dir = templates_dir.join(test.name.clone());
        for entry in std::fs::read_dir(test_source_dir.as_path())
            .with_context(|| format!("Could not read: {}", test_source_dir.display()))?
        {
            let entry = entry.context("Something wrong with the entry")?;
            if entry.path().to_string_lossy().ends_with(".j2") {
                let template = std::fs::read_to_string(entry.path())
                    .with_context(|| format!("Failed to read path {}", entry.path().display()))?;
                let x = entry.file_name();
                let y = x.to_str().unwrap().to_string();
                let template_name = y.strip_suffix(".j2").unwrap();
                source.add_template(template_name, &template).unwrap();
                files_to_template.push(template_name.to_string());
            } else {
                files_to_copy.push(entry);
            }
        }
        let mut env = Environment::new();
        env.set_source(source);
        // create the files
        std::fs::create_dir(test_dir.clone())?;
        // all scenarios
        for scenario in test.generate_scenarios(&dimensions).iter() {
            let scenario_dir =
                scenario.scenario_dir(&test_dir.to_str().context("Failed to convert to string")?);
            std::fs::create_dir(scenario_dir.clone()).with_context(|| {
                format!("Failed to create directory: {}", scenario_dir.display())
            })?;
            // copy files that don't need templating
            for file in files_to_copy.iter() {
                let new_file_name = scenario_dir.join(file.file_name());
                std::fs::copy(file.path(), new_file_name).with_context(|| format!("Failed to copy path: {}", file.path().display()))?;
            }
            // template files and write them
            for file in files_to_template.iter() {
                println!("{:?}", scenario);
                let rendered_file_content = env
                    .get_template(&file)
                    .with_context(|| format!("Failed to retrieve template for: {}", &file))?
                    .render(context!(test_scenario => scenario))
                    .with_context(|| format!("Failed to render template: {}", file))?;
                let out_path = scenario_dir.join(file);
                std::fs::write(out_path.as_path(), rendered_file_content).with_context(|| {
                    format!("Failed to write templated file to: {}", out_path.display())
                })?;
            }
        }
    }
    Ok(())
}
