use clap::Parser;
use filestats::SizeEntry;
use std::error::Error;
use std::io::Error as IOError;
use std::process::exit;

#[derive(Parser, Debug)]
struct Args {
    paths: Vec<String>,
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

    let size_entries: Result<Vec<Vec<SizeEntry>>, IOError> = args
        .paths
        .iter()
        .map(|path| filestats::list(std::path::Path::new(path)))
        .collect();

    let size_entries: Vec<SizeEntry> = size_entries?.into_iter().flatten().collect();

    dir_summary(&size_entries);

    Ok(())
}

fn dir_summary(entries: &[SizeEntry]) {
    entries.iter().for_each(|size_entry| {
        println!("{}\t\t\t{}", size_entry.name, size_entry.size);
    });
    println!("TOTAL:");
    println!(
        "{} files, {} bytes",
        entries.len(),
        entries.iter().map(|e| e.size).sum::<u64>()
    );
}
