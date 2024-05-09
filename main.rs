use clap::Parser;
use std::{
    fs,
    io::Write,
    process::exit,
    sync::{Arc, Mutex},
};

mod parser;
mod pot;
mod visitor;
mod walker;

/// Generate Gettext template files from Javascript/Typescript code.
#[derive(Parser)]
struct Cli {
    /// A list of patterns to exclude
    #[arg(long, num_args(0..), default_values_t = [
        "/.git/".to_string(),
        "/node_modules/".to_string(),
        "/__tests__/".to_string(),
        ".test.".to_string(),
        "/__mocks__/".to_string(),
        ".mock.".to_string(),
        ".story.".to_string(),
        ".cy.".to_string()
    ])]
    exclude: Vec<String>,
    /// The path to the file to read. Defaults to current folder
    #[arg(long)]
    path: Option<std::path::PathBuf>,
    /// The folder where pot files will be written. Each domain will have its own file.
    #[arg(long)]
    output_folder: std::path::PathBuf,
    /// Which folder the references are relative to. Defaults to the output folder.
    #[arg(long)]
    references_relative_to: Option<std::path::PathBuf>,
    /// The default domain to use for strings that don't have a domain specified.
    #[arg(long, default_value = "default")]
    default_domain: String,
}

fn main() {
    let args = Cli::parse();
    run(args)
}

fn run(args: Cli) {
    let default_domain = args.default_domain;
    let exclude = args.exclude;
    let path = args.path.unwrap_or(std::path::PathBuf::from("."));
    let output_folder = args.output_folder;
    let references_relative_to = args.references_relative_to.unwrap_or(output_folder.clone());

    // println!(
    //     "exclude: {:?}\npath: {:?}\noutput: {:?}\nreferences_relative_to: {:?}\ndefault_domain: {:?}",
    //     exclude, path, output_folder, references_relative_to, default_domain
    // );

    let pot = Arc::new(Mutex::new(pot::POT::new(default_domain)));

    match walker::find_ts_files(path, exclude) {
        Ok(entries) => {
            for entry in entries {
                println!("Processing {}", entry.path().to_str().unwrap());
                parser::parse_file(&entry.into_path(), Arc::clone(&pot));
            }
        }
        Err(e) => {
            eprintln!("Error reading path: {}", e);
            exit(1)
        }
    }

    println!("Writing pot files to {}", output_folder.to_str().unwrap());
    let _ = fs::remove_dir_all(&output_folder);
    match fs::create_dir_all(&output_folder) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error creating output folder: {}", e);
            exit(1)
        }
    }

    pot.lock()
        .unwrap()
        .domains
        .iter()
        .for_each(|(domain, pot_file)| {
            let file_path = output_folder.join(format!("{}.pot", domain));
            println!("Writing {}", file_path.to_str().unwrap());
            let mut file = match std::fs::File::create(file_path) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to create file: {}", e);
                    exit(1)
                }
            };
            match file.write_all(pot_file.to_string().as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to write file: {}", e);
                    exit(1)
                }
            };
        });

    println!("Done");
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};
    use pretty_assertions::assert_eq;
    use walkdir::WalkDir;

    use crate::*;

    #[test]
    fn verify_cmd() {
        Cli::command().debug_assert();
    }

    #[test]
    fn verify_snapshot() {
        let args = Cli::parse_from([
            "",
            "--path",
            "./tests/src/",
            "--output-folder",
            "./tests/output/",
            "--default-domain",
            "test",
        ]);
        run(args);
        for entry in WalkDir::new(&"./tests/output/")
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .metadata()
                    .ok()
                    .map(|metadata| metadata.is_file())
                    .unwrap_or(false)
            })
        {
            println!("{:?}", entry.path());
            let actual = fs::read_to_string(entry.path()).unwrap();
            let expected = fs::read_to_string(format!(
                "./tests/expected-output/{}",
                entry.file_name().to_str().unwrap()
            ))
            .unwrap();
            assert_eq!(actual, expected);
        }
    }
}
