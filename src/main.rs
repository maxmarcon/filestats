mod dirutils;

use crate::dirutils::SizeEntry;
use clap::Parser;
use std::error::Error;
use std::io::Error as IOError;
use std::process::exit;

#[derive(Parser, Debug)]
struct Args {
    paths: Vec<String>,
    #[arg(long, short)]
    depth: Option<u32>,
}

fn main() {
    let args = Args::parse();

    if let Err(error) = run(args) {
        println!("{error}");
        exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    if args.paths.is_empty() {
        return Err("You should specify at least one path!".into());
    }

    let size_entries = args
        .paths
        .iter()
        .map(|path| dirutils::list(std::path::Path::new(path), args.depth))
        .collect::<Result<Vec<_>, IOError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    dir_summary(&size_entries);

    Ok(())
}

fn dir_summary(entries: &[SizeEntry]) {
    println!("TOTAL:");
    println!(
        "{} files, {} bytes",
        entries.len(),
        entries.iter().map(|e| e.size).sum::<u64>()
    );
}
