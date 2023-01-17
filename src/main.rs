mod dir;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    paths: Vec<String>,
}

fn main() {
    let args = Args::parse();

    if args.paths.is_empty() {
        panic!("You should specify at least one path!");
    }

    for p in args.paths.iter() {
        match dir::list(std::path::Path::new(p)) {
            Ok(size_entries) => size_entries.iter().for_each(|size_entry| {
                println!("{}\t\t\t{}", size_entry.name, size_entry.size);
            }),
            Err(io_error) => panic!("Error occurred: {}", io_error),
        }
    }
}
