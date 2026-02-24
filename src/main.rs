#![allow(dead_code)]
#![allow(unused_must_use)]

use clap::{Parser, Subcommand};
use dotenv_codegen::dotenv;
use std::{
    env,
    path::PathBuf,
    process::Command,
    time::{Duration, SystemTime},
};
use uuid::*;
use FileSorterX::*;

/*
Made by Xanthus
Check out my other works at https://github.com/Xanthus58
Email me at 'business@xanthus.uk'
You can see more information on my website https://xanthus.uk
*/

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sorts the files in the current directory
    #[command(subcommand)]
    command: Option<Commands>,

    /// Disables telemetry
    #[arg(short, long, default_value_t = false)]
    telemetry: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Sorts files based on file extension matching our database
    Sort {
        /// The input directory
        #[arg(short, long)]
        inputdir: String,

        /// The output directory
        #[arg(short, long)]
        outputdir: String,

        /// Number of directory levels (1-3)
        #[arg(short, long, default_value_t = 2)]
        nesting_level: u8,

        /// Use alternative sorting directory name
        #[arg(short, long, default_value_t = false)]
        use_alt: bool,

        /// Verbose mode
        #[arg(short, long, default_value_t = false)]
        verbose: bool,

        /// Generates a log file
        #[arg(short, long, default_value_t = false)]
        log: bool,
    },
    /// Creates a specified amount of files
    Create {
        /// The amount of files to create
        #[arg(short, long)]
        amount: u32,
    },
    /// Sorts files based on custom file extensions
    Customsort {
        /// The input directory
        #[arg(short, long)]
        inputdir: String,

        /// The output directory
        #[arg(short, long)]
        outputdir: String,

        /// The file extension to sort
        #[arg(short, long)]
        extension: String,

        /// Verbose mode
        #[arg(short, long, default_value_t = false)]
        verbose: bool,

        /// Generates a log file
        #[arg(short, long, default_value_t = false)]
        log: bool,
    },
    /// Updates FileSorterX to the latest version based on the github repo
    Update {},
    /// Note: Only run in a new empty directory. Runs a benchmark test
    Benchmark {},
}

fn main() {
    let cli = Cli::parse();

    let start = SystemTime::now();
    match &cli.command {
        Some(Commands::Sort {
            inputdir,
            outputdir,
            nesting_level,
            use_alt,
            verbose,
            log,
        }) => {
            let in_dir = PathBuf::from(inputdir);
            let out_dir = PathBuf::from(outputdir);

            if !in_dir.is_dir() {
                panic!("Provided path is not a directory: '{:?}'", in_dir)
            }

            sort_files(in_dir, out_dir, *nesting_level, *use_alt, *verbose, *log);
            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Time taken: {:?}", duration);

            if cli.telemetry {
                collect_telemetry(
                    inputdir.to_string(),
                    outputdir.to_string(),
                    &nesting_level.to_string(),
                    &use_alt.to_string(),
                    &verbose.to_string(),
                    &log.to_string(),
                    "N/A".to_string(),
                    "N/A",
                    "Sort Files",
                    duration,
                );
            }
        }
        Some(Commands::Customsort {
            inputdir,
            outputdir,
            extension,
            verbose,
            log,
        }) => {
            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            custom_sort(inputdir, outputdir, extension, *verbose, *log);
            if cli.telemetry {
                collect_telemetry(
                    inputdir.to_string(),
                    outputdir.to_string(),
                    "N/A",
                    "N/A",
                    &verbose.to_string(),
                    &log.to_string(),
                    extension.to_string(),
                    "N/A",
                    "Custom Sort",
                    duration,
                );
            }
        }
        Some(Commands::Create { amount }) => {
            create_files(amount + 1);
            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Time taken: {:?}", duration);

            if cli.telemetry {
                collect_telemetry(
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A".to_string(),
                    amount.to_string().as_str(),
                    "Create Files",
                    duration,
                );
            }
        }
        Some(Commands::Update { .. }) => {
            update_filesorterx().expect("Failed to update FileSorterX");

            if cli.telemetry {
                collect_telemetry(
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A".to_string(),
                    "N/A",
                    "Update",
                    Duration::from_secs(0),
                );
            }
        }
        Some(Commands::Benchmark { .. }) => {
            let time = benchmark();
            println!("Time Taken: {:?}", time);
            if cli.telemetry {
                collect_telemetry(
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A".to_string(),
                    "N/A",
                    "Benchmark",
                    time,
                );
            }
        }
        None => println!("No command provided. Use 'filesorterx --help' for more information."),
    }
}

fn collect_telemetry(
    inputdir: String,
    outputdir: String,
    nesting_level: &str,
    use_alt: &str,
    verbose: &str,
    log: &str,
    extension: String,
    amount: &str,
    cmd: &str,
    time: Duration,
) {
    let id = Uuid::new_v4();
    let os = env::consts::OS;
    let token = dotenv!("TELEMETRY_TOKEN");
    let mut command = String::new();

    command.push_str("'UUID: ");
    command.push_str(&id.to_string());
    command.push_str(" | OS: ");
    command.push_str(os);
    command.push_str(" | Command: ");
    command.push_str(cmd);
    command.push_str(" | Inputdir: ");
    command.push_str(&inputdir);
    command.push_str(" | OutputDir: ");
    command.push_str(&outputdir);
    command.push_str(" | Nesting Level: ");
    command.push_str(nesting_level);
    command.push_str(" | Use Alt: ");
    command.push_str(use_alt);
    command.push_str(" | Verbose: ");
    command.push_str(verbose);
    command.push_str(" | Logging: ");
    command.push_str(log);
    command.push_str(" | Extension: ");
    command.push_str(&extension);
    command.push_str(" | Amount: ");
    command.push_str(amount);
    command.push_str(" | Time Taken: ");
    command.push_str(&time.as_secs_f64().to_string());
    command.push_str(" | FileSorterX Version: ");
    command.push_str(env!("CARGO_PKG_VERSION"));
    command.push('\'');

    Command::new("curl")
        .arg("-A")
        .arg(command)
        .arg(token)
        .output()
        .expect("Failed to execute command");

    println!("Telemetry ID: {}", id);
    println!("Pleaes use this ID when reporting any issues.");
}
