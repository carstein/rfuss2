use std::env;
use std::fs;
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use std::collections::HashMap;

mod child;
mod parent;
mod mutator;

const FILE: &str = "sample.jpg";

#[derive(Default)]
struct Statistics {
    // Number of fuzz cases
    fuzz_cases: u64,

    // Number of crashes
    crashes: u64,

}

struct Corpus {
    files: Vec<Vec<u8>>,
}

impl Corpus {
    fn new() -> Self {
        Corpus {
            files: vec!(),
        }
    }
}

fn read_corpus(entry: &Path) -> Option<Corpus> {
    let mut corpus = Corpus::new();

    if entry.is_dir() {
        for f in fs::read_dir(entry).unwrap() {
            let data = fs::read(f.unwrap().path()).unwrap();
            corpus.files.push(data.to_vec())
        }

    } else {
        corpus.files.push(fs::read(entry).unwrap())
    }

    Some(corpus)
}

fn parse_breakpoint_map(filename: &Path) -> Option<Vec<u64>> {
    let mut bp_map: Vec<u64> = vec!();

    if filename.is_file() {
        let fh = fs::File::open(filename).unwrap();
        let reader = BufReader::new(fh);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        bp_map.push(u64::from_str_radix(line.trim_start_matches("0x"), 16).expect("Failed parsing"));
                    }
                },
                Err(_) => {
                    println!("Failed reading line");
                }
            }
        }

        Some(bp_map)
    } else {
        None
    }
}

fn main() {
    // Extract two arguments
    let prog_name = env::args().nth(1).expect("Program name not provided");
    let corpus =  env::args().nth(2).expect("Filename not provided");
    let breakpoint_map = env::args().nth(3).expect("Breakpoint map not provided");

    // Stats
    let mut stats = Statistics::default();

    // Mutation engine
    let mut mutator = mutator::Mutator::new();

    // Consume corpus of files
    println!("[+] Loading original file");
    let corpus = read_corpus(Path::new(&corpus)).expect("Failed to obtain corpus");

    println!("[+] Parsing breakpoin map");
    let bp_map = parse_breakpoint_map(Path::new(&breakpoint_map))
                    .expect("Failed to parse breakpoint map");

    // Hash map for storing original bytes where we inserted breakpoints
    let mut bp_mapping: HashMap<u64, i64> = HashMap::new();

    // Feed it to mutator
    for sample in corpus.files {
        mutator.feed(sample);
    }

    // Timer for stats
    let start = Instant::now();

    // Enter loop
    loop {
        for sample in &mut mutator {
            // Save sample
            sample.materialize_sample(FILE);

            let child_pid = child::run_child(&prog_name, &FILE);

            // Pre-set all breakpoints for coverage tracking
            for bp in &bp_map {
                bp_mapping.insert(*bp, parent::set_breakpoint(child_pid, *bp));
            }

            stats.fuzz_cases += 1;
            match parent::run_parent(child_pid, &bp_mapping) {
                parent::ParentStatus::Clean => {
                    // just handle stats
                }

                parent::ParentStatus::Crash(rip) => {
                    stats.crashes += 1;
                    let crash_filename = format!("crash_{}.jpg", rip);
                    fs::copy(FILE, crash_filename)
                        .expect("Failed to save crash file");
                }
            }

            if stats.fuzz_cases % 1000 == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                print!("[{:10.6}] cases {:10} | fcps  {:10.2} | crashes {:10}\n",
                        elapsed, stats.fuzz_cases, 
                        stats.fuzz_cases as f64/ elapsed, stats.crashes);
            }
        }
    }
}