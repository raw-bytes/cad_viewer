use anyhow::{bail, Context, Result};
use log::info;
use std::{env, path::PathBuf, str::FromStr};

/// The program arguments
pub struct Arguments {
    pub input_file: PathBuf,
}

impl Arguments {
    /// Parses the program arguments.
    pub fn parse_args() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        let args = &args[1..];

        if args.len() != 1 {
            bail!("Invalid number of program arguments");
        }

        let input_file = PathBuf::from_str(&args[0]).context("Failed to parse input file path")?;

        Ok(Self { input_file })
    }

    /// Prints all arguments into the log
    pub fn print_to_log(&self) {
        info!("Input File: {}", self.input_file.to_string_lossy());
    }
}
