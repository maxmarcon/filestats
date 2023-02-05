mod dirutils;
mod stats;

use clap::Parser;
use std::error::Error;
use std::io::Error as IOError;
use std::process::exit;

use crate::dirutils::SizeEntry;
use crate::stats::Histogram;

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

const HIST_CEILINGS: [u64; 10] = [
    2_u64.pow(10),
    10 * 2_u64.pow(10),
    100 * 2_u64.pow(10),
    2_u64.pow(20),
    10 * 2_u64.pow(20),
    100 * 2_u64.pow(20),
    2_u64.pow(30),
    10 * 2_u64.pow(30),
    100 * 2_u64.pow(30),
    2_u64.pow(40),
];

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    if args.paths.is_empty() {
        return Err("You should specify at least one path!".into());
    }

    let hist = args
        .paths
        .iter()
        .map(|path| dirutils::list(std::path::Path::new(path), args.depth))
        .collect::<Result<Vec<_>, IOError>>()?
        .into_iter()
        .flatten()
        .fold(Histogram::new(&HIST_CEILINGS), |mut hist, size_entry| {
            hist.add(size_entry.size);
            hist
        });

    println!("{:#?}", hist);

    Ok(())
}

#[allow(dead_code)]
fn dir_summary(entries: &[SizeEntry]) {
    println!("TOTAL:");
    println!(
        "{} files, {} bytes",
        entries.len(),
        entries.iter().map(|e| e.size).sum::<u64>()
    );
}
