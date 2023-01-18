use clap::Parser;
use filestats::SizeEntry;

#[derive(Parser, Debug)]
struct Args {
    paths: Vec<String>,
}

fn main() {
    let args = Args::parse();

    if args.paths.is_empty() {
        panic!("You should specify at least one path!");
    }

    for path in args.paths.iter() {
        match filestats::list(std::path::Path::new(path)) {
            Ok(size_entries) => dir_summary(&size_entries),
            Err(io_error) => panic!("Error occurred: {}", io_error),
        }
    }
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
