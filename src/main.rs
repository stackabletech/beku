mod test_definition;
mod test_generation;
use anyhow::{Ok, Result};
use clap::Parser;
use std::path::Path;
use test_definition::TestDefinition;
use test_generation::TestGeneration;

/// Generate kuttl tests.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The test definition yaml file
    #[arg(short, long, default_value = "test-definition.yaml")]
    definition: String,

    /// Test kuttl test definition file
    #[arg(short, long, default_value = "kuttl-test.yaml.jinja2")]
    kuttl_test: String,

    /// The directory containing the templates
    #[arg(short, long, default_value = "templates")]
    templates: String,

    /// The output directory path
    #[arg(short, long, default_value = "out")]
    out: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // read the test definition file
    let test_def = TestDefinition::from_yaml_file(Path::new(&args.definition))?;
    // generate
    let test_gen = TestGeneration::new(
        Path::new(&args.out),
        Path::new(&args.templates),
        test_def,
        Path::new(&args.kuttl_test),
    );
    test_gen.generate()?;

    Ok(())
}
