mod config;
#[cfg(test)]
mod tests;

use config::EXTENSIONS;
use rayon::prelude::*;
use rand::Rng;
use self_update::cargo_crate_version;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

pub fn create_files(amount: u32) {
    // Use parallel processing for faster file creation
    (1..amount).into_par_iter().for_each(|file| {
        let mut file_name = String::new();
        file_name.push_str(&file.to_string());
        file_name.push('.');
        let mut rng = rand::thread_rng();
        let random_extension = EXTENSIONS[rng.gen_range(0..EXTENSIONS.len())].0;
        file_name.push_str(random_extension);
        
        if let Err(e) = fs::File::create(&file_name) {
            eprintln!("Error creating file {}: {}", file_name, e);
        }
    });
}

pub fn custom_sort(
    input_directory: &str,
    output_directory: &str,
    extension: &str,
    verbose: bool,
    log: bool,
) {
    // Set up the directories
    let input_directory = Path::new(input_directory);
    let output_directory = Path::new(output_directory);

    // Get all the files in the input directory
    let files = fs::read_dir(input_directory).unwrap();

    // Use parallel processing for better performance
    files.par_iter().for_each(|file| {
        if let Ok(file) = file {
            let file = file.path();
            let file_name = match file.file_name() {
                Some(file_name) => file_name,
                None => return,
            };

            match file.extension() {
                Some(ext) if ext == extension => {
                    if let Err(e) = fs::create_dir_all(output_directory) {
                        eprintln!("Error creating directory {}: {}", output_directory.display(), e);
                        return;
                    }
                    let output_file = output_directory.join(file.file_name().unwrap());
                    if let Err(e) = fs::rename(file.clone(), output_file) {
                        eprintln!("Error moving file {}: {}", file.display(), e);
                        return;
                    }
                }
                _ => return,
            }

            if verbose {
                println!("Moved file: {:?} to {:?}", file, output_directory);
            }

            if log {
                if let Err(e) = write_logfile(
                    file.as_os_str(),
                    output_directory,
                    input_directory.to_str().unwrap(),
                ) {
                    eprintln!("Error writing log: {}", e);
                }
            }
        }
    });
}

/// # Usage
/// ```markdown
/// (ext, (type, alt, sorted_dir)),
///
/// ("gif", ("image", Some("animated"), None)),
/// ("qt", ("video", None, Some("quicktime"))),
/// ("mp4", ("video", None, None)),
///
/// nesting_level, use_alt => gif, qt, mp4
///
/// 1, false => "image", "video", "video"
/// 2, false => "image/gif", "video/quicktime", "video/mp4"
/// 3, false => "image/gif", "video/quicktime", "video/mp4"
///
/// 1, true => "image", "video", "video"
/// 2, true => "image/animated", "video/quicktime", "video/mp4"
/// 3, true => "image/animated/gif", "video/quicktime", "video/mp4"
/// ```
pub fn get_subdir_by_extension(ext: &str, nesting_level: u8, use_alt: bool) -> PathBuf {
    if !(1..=3).contains(&nesting_level) {
        panic!("Nesting level is out of range.");
    }

    let extensions: HashMap<&str, (&str, Option<&str>, Option<&str>)> =
        HashMap::from(config::EXTENSIONS);

    let ext_data = match extensions.get(ext) {
        None => return PathBuf::from("other"),
        Some(e) => e,
    };

    let mut path = PathBuf::from(ext_data.0);

    match (nesting_level, use_alt) {
        (1, _) => {} // Do nothing
        (2, true) => {
            path.push(ext_data.1.unwrap_or(ext_data.2.unwrap_or(ext))); // use alt, then use sorted_dir, then use provided ext.
        }
        (3, true) => {
            if ext_data.1.is_some() {
                path.push(ext_data.1.unwrap())
            }
            path.push(ext_data.2.unwrap_or(ext));
        }
        (_, false) => {
            // 2 or 3
            // If sorted_dir is present in config, use it, otherwise fallback to provided one.
            path.push(ext_data.2.unwrap_or(ext));
        }
        _ => {
            panic!(
                "{} | get_subdir_by_extension() | nesting_level: {nesting_level}, use_alt: {use_alt}",
                file!()
            )
        }
    }

    path
}

pub fn write_logfile(file_name: &OsStr, moveto_directory: &Path, input_directory: &str) -> bool {
    let logdir = Path::new(input_directory).join("sorter-logs/");
    fs::create_dir_all(logdir.clone()).unwrap();
    let mut logfile = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(logdir.to_str().unwrap().to_owned() + "sorter.log")
        .expect("create failed");

    logfile
        .write_all(format!("{:?}", file_name).as_bytes())
        .expect("write failed");
    logfile
        .write_all(" Moved to ".as_bytes())
        .expect("write failed");
    logfile
        .write_all(format!("{:?}\n", moveto_directory.display()).as_bytes())
        .expect("write failed");

    true
}

pub fn sort_files(
    in_dir: PathBuf,
    out_dir: PathBuf,
    nesting_level: u8,
    use_alt: bool,
    verbose: bool,
    log: bool,
) -> std::io::Result<()> {
    // Collect all files first for better performance
    let entries: Vec<_> = fs::read_dir(in_dir.clone())?.collect();
    
    // Use parallel processing for better performance on multi-core systems
    entries.par_iter().for_each(|entry| {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(f) => f,
                None => return,
            };
            let ext = match path.extension() {
                Some(e) => e,
                None => return,
            };

            let moveto_directory = out_dir.join(get_subdir_by_extension(
                ext.to_str().unwrap(),
                nesting_level,
                use_alt,
            ));
            
            // Create directories if they don't exist
            if let Err(e) = fs::create_dir_all(&moveto_directory) {
                eprintln!("Error creating directory {}: {}", moveto_directory.display(), e);
                return;
            }
            
            // Move file with error handling
            if let Err(e) = fs::rename(&path, moveto_directory.join(path.file_name().unwrap())) {
                eprintln!("Error moving file {}: {}", path.display(), e);
                return;
            }

            if verbose {
                println!("{:?} moved to {:?}", file_name, moveto_directory.display());
            }

            if log {
                let log_dir = "sorter-logs";
                if let Err(e) = fs::create_dir_all(log_dir) {
                    eprintln!("Error creating log directory {}: {}", log_dir, e);
                    return;
                }
                if let Err(e) = write_logfile(file_name, &moveto_directory, in_dir.to_str().unwrap()) {
                    eprintln!("Error writing log: {}", e);
                }
            }
        }
    });

    Ok(())
}

pub fn update_filesorterx() -> Result<(), Box<dyn (std::error::Error)>> {
    println!("Updating FileSorterX to the latest version...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("xanthus58")
        .repo_name("FileSorterX")
        .bin_name("github")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(())
}

pub fn benchmark() -> Duration {
    let files = fs::read_dir(".");
    if files.is_ok() && files.unwrap().count() > 0 {
        println!("Please run benchmark in an empty directory.");
        return Duration::from_secs(0);
    }

    let startbench = SystemTime::now();
    create_files(10001);
    
    // Use parallel processing for faster sorting
    let result = sort_files(".".into(), "./benchmark".into(), 3, false, false, false);
    if let Err(e) = result {
        eprintln!("Benchmark failed: {}", e);
    }
    
    if let Err(e) = std::fs::remove_dir_all("./benchmark") {
        eprintln!("Failed to remove benchmark directory: {}", e);
    }
    
    let endbench = SystemTime::now();
    endbench.duration_since(startbench).unwrap()
}
