use std::{
    collections::{HashMap, VecDeque},
    io::Read,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
};

use clarg::{Arg, ArgMap, ArgParser};
use sha2::{Digest, Sha256, digest::generic_array::functional::FunctionalSequence};

#[cfg(target_os = "windows")]
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;

fn main() {
    let args = setup();
    let file_hashmap = check_duplicates(args);
    print_results(file_hashmap);
}

/// Execute the logic that searches for duplicate files.
/// This function calculates a hash of each file. When duplicates are found,
/// a list of files is stored per each hash.
fn check_duplicates(args: ArgMap) -> HashMap<String, Vec<PathBuf>> {
    let path = args.get_raw("path").expect("Invalid path");
    let mut file_hashmap = HashMap::new();
    let mut directory_queue = VecDeque::new();

    // Visit the folder passed.
    if let Err(err) = walk_directory(path, &mut directory_queue, &mut file_hashmap, &args) {
        eprintln!("Error walking directory: `{path}` {err}");
    } else {
        // We may need to run recursively
        if args.get::<bool>("recurse").is_ok() {
            while !directory_queue.is_empty() {
                let tip = directory_queue.pop_front();
                if let Some(directory) = tip {
                    if let Err(err) =
                        walk_directory(&directory, &mut directory_queue, &mut file_hashmap, &args)
                    {
                        eprintln!(
                            "Error walking directory: `{}` {err}",
                            directory.to_string_lossy()
                        );
                    }
                }
            }
        }
    }
    file_hashmap
}

/// Prints the results of the execution including all duplicates found if any.
fn print_results(file_hashmap: HashMap<String, Vec<PathBuf>>) {
    let mut duplicates_found = false;
    println!("Went through: {} unique files", file_hashmap.len());

    for (_, file_list) in file_hashmap {
        if file_list.len() > 1 {
            duplicates_found = true;
            println!("------- Multiple Entries Found -------");
            for (index, file) in file_list.iter().enumerate() {
                println!("{:>5} -> `{}`", index + 1, file.to_string_lossy());
            }
            println!("--------------------------------------");
        }
    }

    if !duplicates_found {
        println!("No duplicates found with hash comparison method.");
    }
}

/// Set up, and parse arguments for the CLI.
fn setup() -> ArgMap {
    ArgParser::new("Find duplicate files.")
        .arg(Arg::string(
            "path",
            Some('p'),
            true,
            "Directory being analyzed",
        ))
        .arg(Arg::boolean("recurse", Some('r'), "Run recursively"))
        .arg(Arg::boolean("include-hidden", None, "Include hidden."))
        .parse()
}

/// Walk a given directory.
/// # Arguments
/// `path` the directory being analyzed
/// `to_visit_queue` queue to store all directories found. Used in recursive execution.
/// `file_hash_map`  map storing all hashes and files analyzed.
/// `config` map or arguments passed to the CLI
fn walk_directory(
    path: impl AsRef<Path>,
    to_visit_queue: &mut VecDeque<PathBuf>,
    file_hash_map: &mut HashMap<String, Vec<PathBuf>>,
    config: &ArgMap,
) -> std::io::Result<()> {
    let directory_iterator = std::fs::read_dir(path)?;
    let include_hidden = config.has_arg("include-hidden");
    for dir_item in directory_iterator.flatten() {
        let item_path = dir_item.path();

        // Check if hidden files are to be ignored
        if let Ok(meta) = item_path.metadata() {
            if meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 && !include_hidden {
                continue;
            }
        }

        if item_path.is_dir() {
            to_visit_queue.push_back(item_path);
        } else {
            let hash = get_file_hash(&item_path)?;
            let file_list = file_hash_map.entry(hash).or_insert(Vec::new());
            file_list.push(item_path);
        }
    }

    Ok(())
}

/// Determine the hash for a given file
fn get_file_hash(path: &PathBuf) -> std::io::Result<String> {
    let mut buffer = [0; 4096];
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    loop {
        let read_bytes = file.read(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }
        hasher.update(&buffer[..read_bytes]);
    }
    Ok(hasher.finalize().map(|byte| format!("{:x}", byte)).join(""))
}
