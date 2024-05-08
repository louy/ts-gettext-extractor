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

    let default_domain = args.default_domain;
    let exclude = args.exclude;
    let path = args.path.unwrap_or(std::path::PathBuf::from("."));
    let output_folder = args.output_folder;
    let references_relative_to = args.references_relative_to.unwrap_or(output_folder.clone());

    println!(
        "exclude: {:?}\npath: {:?}\noutput: {:?}\nreferences_relative_to: {:?}\ndefault_domain: {:?}",
        exclude, path, output_folder, references_relative_to, default_domain
    );

    let pot = Arc::new(Mutex::new(pot::POT::new(None)));

    match walker::find_ts_files(path, exclude) {
        Ok(entries) => {
            for entry in entries {
                println!("{:?}", entry.path());
                parser::parse_file(&entry.into_path(), Arc::clone(&pot));
            }
        }
        Err(e) => {
            eprintln!("Error reading path: {}", e);
            exit(1)
        }
    }

    println!("Writing pot files to {:?}", output_folder);
    fs::remove_dir_all(&output_folder).unwrap_or_default();
    fs::create_dir(&output_folder).unwrap();

    pot.lock()
        .unwrap()
        .domains
        .iter()
        .for_each(|(domain, pot_file)| {
            let file_path = output_folder.join(format!("{}.pot", domain));
            println!("Writing {:?}", file_path);
            let mut file = std::fs::File::create(file_path).unwrap();
            file.write_all(pot_file.to_string().as_bytes()).unwrap();
        });
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::Cli;

    #[test]
    fn verify_cmd() {
        Cli::command().debug_assert();
    }
}
