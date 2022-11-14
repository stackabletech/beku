mod test_definition;
mod test_generation;
use anyhow::{Ok, Result};
use std::path::Path;
use test_definition::TestDefinition;
use test_generation::TestGeneration;

fn main() -> Result<()> {
    // read the test definition file
    let test_def = TestDefinition::from_yaml_file(&Path::new("test-definition.yaml"))?;
    let test_gen =
        TestGeneration::new(&Path::new("tests"), &Path::new("templates/kuttl"), test_def);
    test_gen.generate()?;

    Ok(())
}
