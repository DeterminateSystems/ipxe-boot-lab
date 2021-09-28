use std::path::{Path, PathBuf};

use structopt::StructOpt;

fn validate_is_file(path: String) -> Result<(), String> {
    if Path::new(&path).is_file() {
        return Ok(());
    }

    Err(format!("'{}' was not a file", path))
}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// The path to the metadata of the machine being emulated
    #[structopt(required = true, validator = validate_is_file)]
    pub meta_file: PathBuf,
    /// Whether or not to run in non-interactive mode (will print everything to stdout and will not
    /// accept any input)
    #[structopt(long, conflicts_with = "manual")]
    pub non_interactive: bool,
    /// Whether or not to manually set up screen sessions
    #[structopt(long, conflicts_with = "non-interactive")]
    pub manual: bool,
}
