mod test_definition;
mod test_generation;
use anyhow::{Context, Ok, Result};
use minijinja::{context, Environment, Source};
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};
use test_definition::TestDefinition;
use test_generation::TestGeneration;

// TODO we need to use walkdir
// we can't build all the templates first, and then go through each scenario anymore
// it would be better to go through each file and place it in all the scenarios immediately

// walk dir
// if directory:
//   for every

fn main() -> Result<()> {
    // read the test definition file

    let file_contents = std::fs::read_to_string("test-definition.yaml")
        .context("Failed to read test definition file")?;
    let test_def: TestDefinition = serde_yaml::from_str(&file_contents).unwrap();
    let test_gen =
        TestGeneration::new(&Path::new("tests"), &Path::new("templates/kuttl"), test_def);
    test_gen.generate()?;

    Ok(())
}
