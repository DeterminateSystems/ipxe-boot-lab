use std::path::{Path, PathBuf};
use std::str::FromStr;

use structopt::StructOpt;

const DRIVERS: &[&str; 3] = &["read-only", "tmux", "manual"];

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
    /// The driver to use
    #[structopt(long, possible_values = DRIVERS, case_insensitive = true, default_value = "tmux")]
    pub driver: Driver,
}

#[derive(Debug, PartialEq)]
pub enum Driver {
    ReadOnly,
    Tmux,
    Manual,
}

impl FromStr for Driver {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = match s {
            "read-only" => Driver::ReadOnly,
            "tmux" => Driver::Tmux,
            "manual" => Driver::Manual,
            _ => return Err(format!("not one of {:?}", DRIVERS)),
        };

        Ok(ret)
    }
}
