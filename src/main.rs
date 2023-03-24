use anyhow::{bail, Context, Result};
use args::Arguments;
use cad_import::{loader::Manager, structure::CADData};
use log::{error, info, LevelFilter};
use std::{fs::File, path::Path, process::ExitCode};
use viewer::Viewer;

use crate::viewer::Renderer;

mod args;
mod gpu_data;
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

/// Tries to find the mime types for the given file based on the file extension.
///
/// # Arguments
/// * `input_file` - The input file whose extension will be used
fn determine_mime_types(manager: &Manager, input_file: &Path) -> Result<Vec<String>> {
    match input_file.extension() {
        Some(ext) => match ext.to_str() {
            Some(ext) => Ok(manager.get_mime_types_for_extension(ext)),
            None => {
                bail!("Input file has invalid extension");
            }
        },
        None => {
            bail!("Input file has no extension");
        }
    }
}

/// Tries to load the cad data from the given path
///
/// # Arguments
/// * `file_path` - The path to load the CAD data from.
fn load_cad_data(file_path: &Path) -> Result<CADData> {
    let manager = Manager::new();

    let mime_types = determine_mime_types(&manager, file_path)?;

    for mime_type in mime_types.iter() {
        match manager.get_loader_by_mime_type(mime_type.as_str()) {
            Some(loader) => {
                let cad_data = loader
                    .read_file(file_path, mime_type)
                    .context(format!("Failed reading input file {:?}", file_path))?;

                return Ok(cad_data);
            }
            None => {}
        }
    }

    bail!("Cannot find loader for the input file {:?}", file_path);
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
