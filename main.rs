use clap::Parser;
use std::fs;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    #[arg(long, num_args(0..), default_value(""))]
    exclude: String,
    /// The path to the file to read
    #[arg(long)]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    println!("exclude: {:?}, path: {:?}", args.exclude, args.path);

    match fs::read_dir(args.path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => println!("{:?}", entry.path()),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error reading path: {}", e),
    }
}
