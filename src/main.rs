use clap::Parser;
use std::error::Error;
use std::io::Error as IOError;
use std::process::exit;
use std::time::Instant;

use filestats::dirutils;
use filestats::stats::Histogram;

#[derive(Parser, Debug)]
struct Args {
    paths: Vec<String>,
    #[arg(long, short)]
    depth: Option<u32>,
}

fn main() {
    let args = Args::parse();

    if let Err(error) = run(args) {
        eprintln!("{error}");
        exit(1);
    }
}

const SIZES: [u64; 3] = [1, 10, 100];
const EXP: [u32; 4] = [10, 20, 30, 40];

fn progress(count: usize) -> () {
    print!("\rlooking at {} files", count)
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    if args.paths.is_empty() {
        return Err("You should specify at least one path!".into());
    }

    let ceilings = EXP
        .iter()
        .flat_map(|&e| SIZES.map(|s| s * 2_u64.pow(e)))
        .collect::<Vec<_>>();

    let start = Instant::now();
    let hist = args
        .paths
        .iter()
        .map(|path| dirutils::list(std::path::Path::new(path), args.depth, progress))
        .collect::<Result<Vec<_>, IOError>>()?
        .into_iter()
        .flatten()
        .fold(Histogram::new(&ceilings), |mut hist, size_entry| {
            hist.add(size_entry.size);
            hist
        });

    println!("\r");
    println!("{}", hist);

    println!(
        "Looked at {} files in {} seconds",
        hist.count(),
        start.elapsed().as_secs()
    );

    Ok(())
}
