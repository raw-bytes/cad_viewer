use log::{error, LevelFilter};

mod args;

/// Initializes the program logging
fn initialize_logging() {
    simple_logging::log_to(std::io::stdout(), LevelFilter::Info);
}

/// Prints the usage of the program
fn print_usage() {
    println!("cad_viewer <INPUT>\n");
    println!("INPUT: The path to the input file");
}

fn main() {
    initialize_logging();
    let args = match args::Arguments::parse_args() {
        Ok(args) => args,
        Err(err) => {
            print_usage();
            error!("Failed to parse program arguments: {}", err);
            std::process::exit(exitcode::USAGE);
        }
    };

    args.print_to_log();
}
