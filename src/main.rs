use clap::{App, Arg};

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

mod child;
mod parent;
mod mutator;
mod config;


const FILE: &str = "input_file";

#[derive(Default)]
struct Statistics {
    // Number of fuzz cases
    fuzz_cases: u64,

    // Number of crashes
    crashes: u64,
}

fn main() {
    // Extract arguments via clap
    let options = App::new("Rufus2 fuzzer")
        .version("1.0")
        .author("@carste1n")
        .about("Simple fuzzer in Rust")
        .arg(Arg::with_name("prog_name")
            .short("p")
            .long("prog")
            .takes_value(true)
            .help("Name of the program to execute")
            .required(true))
        .arg(Arg::with_name("corpus")
            .short("c")
            .long("corpus")
            .takes_value(true)
            .help("File corpus to be consumed by the fuzzer")
            .required(true))
        .arg(Arg::with_name("bpmap")
            .short("b")
            .long("bpmap")
            .takes_value(true)
            .help("List of breakpoints to set")
            .required(true))
        .get_matches();

    println!("[#] Generating runtime config.");
    let runtime_config = config::generate_runtime_config(options);

    // Mutation engine
    println!("[#] Initializing mutation engine");
    let mut mutator = mutator::Mutator::new();

    // Hash map for storing original bytes where we inserted breakpoints
    let mut bp_mapping: HashMap<u64, i64> = HashMap::new();

    // Feed it to mutator
    println!("[+] Feeding corpus to mutator");
    mutator.consume(&runtime_config.corpus);

    // Runtime container for executed samples
    let mut sample_pool: Vec<mutator::Sample> = vec!();

    // Timer for stats
    let mut stats = Statistics::default();
    let start = Instant::now();
    let mut go = true;

    // MAIN FUZZ LOOP
   while go {
        for mut sample in &mut mutator {
            // Save sample
            sample.materialize_sample(FILE);
            let child_pid = child::run_child(&runtime_config, &mut bp_mapping, FILE);

            stats.fuzz_cases += 1;
            match parent::run_parent(child_pid, &bp_mapping) {
                parent::ParentStatus::Clean(trace) => {
                    sample.add_trace(trace);
                    sample_pool.push(sample);
                }

                parent::ParentStatus::Crash(rip) => {
                    stats.crashes += 1;
                    println!("[!] Crash for input {:?}", sample);
                    let crash_filename = format!("crash_{}", rip);
                    fs::copy(FILE, crash_filename)
                        .expect("Failed to save crash file");
                    go = false;
                }
            }
        }

        // Send back all the sample with traces to the mutator
        mutator.update(&sample_pool);
    }

    let elapsed = start.elapsed().as_secs_f64();
    print!("[{:10.2}] cases {:10} | fcps  {:10.2} | crashes {:10}\n",
            elapsed, stats.fuzz_cases, 
            stats.fuzz_cases as f64/ elapsed, stats.crashes);
}