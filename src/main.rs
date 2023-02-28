use anyhow::{bail, Context, Result};
use args::Arguments;
use cad_import::{loader::Manager, structure::CADData};
use log::{error, info, LevelFilter};
use std::{fs::File, path::Path, process::ExitCode};
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

/// Tries to return the extension from the given file path
///
/// # Arguments
/// * `file_path` - The filepath from which the extension will be extracted.
fn get_extension(file_path: &Path) -> Result<String> {
    match file_path.extension() {
        Some(ext) => {
            let ext: String = ext.to_string_lossy().to_string();
            Ok(ext)
        }
        None => {
            bail!("Input file {:?} has no extension", file_path);
        }
    }
}

/// Tries to load the cad data from the given path
///
/// # Arguments
/// * `file_path` - The path to load the CAD data from.
fn load_cad_data(file_path: &Path) -> Result<CADData> {
    let ext = get_extension(file_path)?;

    let manager = Manager::new();
    match manager.get_loader_by_extension(&ext) {
        Some(loader) => {
            let mut f = File::open(file_path)
                .context(format!("Failed opening input file {:?}", file_path))?;
            let cad_data = loader
                .read_file(&mut f)
                .context(format!("Failed reading input file {:?}", file_path))?;

            Ok(cad_data)
        }
        None => {
            bail!("Cannot find loader for the input file {:?}", file_path);
        }
    }
}

/// The central entry point for starting the program
fn run_program(args: Arguments) -> Result<()> {
    // load cad data
    info!("Load '{}'...", args.input_file.to_string_lossy());
    let cad_data = load_cad_data(&args.input_file)?;
    info!("Load '{}'...DONE", args.input_file.to_string_lossy());

    let renderer = Renderer::new(cad_data);
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
