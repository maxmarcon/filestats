use clap::Parser;
use std::process::exit;
use std::time::Instant;

use filestats::dirutils;
use filestats::dirutils::SizeEntry;
use filestats::stats::Histogram;

use filestats::utils::format_bytes;

#[derive(Parser, Debug)]
#[command(
    name = "filestats",
    about = "Utility that outputs colorful statistics about the size of your files",
    author = "Max Marcon (https://github.com/maxmarcon/)",
    version = "0.1.0"
)]
struct Args {
    paths: Vec<String>,
    #[arg(
        long,
        short = 'd',
        help = "max depth to consider. 0 = do not recurse into subdirectories. Default: infinity"
    )]
    max_depth: Option<u32>,
    #[arg(long, short, help = "shows verbose information about errors")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if let Err(error_msg) = run(args) {
        eprintln!("{error_msg}");
        exit(1);
    }
}

const SIZES: [u64; 3] = [1, 10, 100];
const EXP: [u32; 4] = [10, 20, 30, 40];

fn run(args: Args) -> Result<(), &'static str> {
    if args.paths.is_empty() {
        return Err("You should specify at least one path!");
    }

    let ceilings = EXP
        .iter()
        .flat_map(|&e| SIZES.map(|s| s * 2_u64.pow(e)))
        .collect::<Vec<_>>();

    let start = Instant::now();
    let (mut min, mut max): (Option<SizeEntry>, Option<SizeEntry>) = (None, None);

    let (hist, errors) = args
        .paths
        .iter()
        .flat_map(|path| dirutils::list(std::path::Path::new(path), args.max_depth))
        .enumerate()
        .map(|(cnt, r)| {
            if cnt % 10 == 0 {
                print!("\rScanning {} files", cnt);
            }
            match r {
                Err(ref error) => {
                    if args.verbose {
                        eprintln!("\n{}", error);
                    }
                }
                Ok(ref size_entry) => {
                    if max.is_none() || size_entry.size > max.as_ref().unwrap().size {
                        max = Some(size_entry.to_owned());
                    }
                    if min.is_none() || size_entry.size < min.as_ref().unwrap().size {
                        min = Some(size_entry.to_owned())
                    }
                }
            }
            r
        })
        .fold(
            (Histogram::new(&ceilings), 0),
            |(mut hist, errors), r| match r {
                Ok(size_entry) => {
                    hist.add(size_entry.size);
                    (hist, errors)
                }
                Err(_) => (hist, errors + 1),
            },
        );

    print!("\r");
    println!("{}", hist);

    println!(
        "Scanned {} files in {} seconds",
        hist.count(),
        start.elapsed().as_secs()
    );

    if let Some(avg_size) = hist.avg() {
        println!("Average size: {} bytes", format_bytes(avg_size as u64))
    }

    if let Some(max) = max {
        println!(
            "Largest file at {} bytes: {:?}",
            format_bytes(max.size),
            max.path
        );
    }

    if let Some(min) = min {
        println!(
            "Smallest file at {} bytes: {:?}",
            format_bytes(min.size),
            min.path
        );
    }

    if errors > 0 {
        println!("{} files could not be read", errors);
    }

    Ok(())
}
