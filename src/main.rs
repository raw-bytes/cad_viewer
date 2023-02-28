use anyhow::{Context, Result};
use args::Arguments;
use log::{error, info, LevelFilter};
use std::process::ExitCode;
use viewer::Viewer;

use crate::viewer::Renderer;

mod args;
mod viewer;

/// Initializes the program logging
fn initialize_logging() {
    simple_logging::log_to(std::io::stdout(), LevelFilter::Debug);
}

/// Prints the usage of the program
fn print_usage() {
    println!("cad_viewer <INPUT>\n");
    println!("INPUT: The path to the input file");
}

/// The central entry point for starting the program
fn run_program(args: Arguments) -> Result<()> {
    let renderer = Renderer::new(&args.input_file);
    let viewer =
        Viewer::new("Simple CAD Viewer", renderer).context("Failed initializing the viewer")?;

    info!("Start viewer...");
    viewer.run()?;
    info!("Viewer closed");
    Ok(())
}

fn main() -> ExitCode {
    initialize_logging();
    let args = match args::Arguments::parse_args() {
        Ok(args) => args,
        Err(err) => {
            print_usage();
            error!("Failed to parse program arguments: {}", err);
            return ExitCode::FAILURE;
        }
    };

    args.print_to_log();

    match run_program(args) {
        Err(err) => {
            error!("Failed due to {}", err);
            ExitCode::FAILURE
        }
        _ => {
            info!("CLOSED");
            ExitCode::SUCCESS
        }
    }
}
