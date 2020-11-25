use std::env;
use std::fs;
use std::time::Instant;
use std::vec::Vec;

use rand::{thread_rng};
use rand::seq::SliceRandom as _;

use nix::unistd::{fork, ForkResult};

mod child;
mod parent;

const FILE: &str = "sample.jpg";

#[derive(Default)]
struct Statistics {
    // Number of fuzz cases
    fuzz_cases: u64,

    // Number of crashes
    crashes: u64,

}

struct Sample {
    version: u32,
    data: Vec<u8>
}

impl Sample {
    fn new(content: Vec<u8>) -> Self {
        Sample {version:1, data: content }
    }

    fn mutate(&self, rng: &mut rand::prelude::ThreadRng) -> Sample {
        let mut bytecode = self.data.to_vec();
        let choosen_byte = bytecode.as_mut_slice().choose_mut(rng).unwrap();

        let b = [1, 2, 4, 8,16, 32, 64, 128].choose(rng).unwrap();
        *choosen_byte ^= b;

        Sample {
            version: &self.version + 1,
            data: bytecode,
        }
    }
}


fn main() {
    // Extract two arguments
    let prog_name = env::args().nth(1).expect("Program name not provided");
    let argument =  env::args().nth(2).expect("Filename not provided");

    // Stats
    let mut stats = Statistics::default();

    let mut rng = thread_rng();
    // Read a file and create fuzzing sample
    println!("[+] Loading original file");
    let orig_file = fs::read(argument).expect("Failed opening file!");
    let oring_sample = Sample::new(orig_file);

    // Timer for stats
    let start = Instant::now();

    // Enter loop
    loop {

        // Mutate sample
        let mutated_sample = oring_sample.mutate(&mut rng);

        // Save sample
        fs::write(FILE, &mutated_sample.data)
            .expect("error saving file");

        match unsafe{fork()} {
            
            Ok(ForkResult::Child) => {
                child::run_child(&prog_name, &FILE);
            }
            
            Ok(ForkResult::Parent {child}) => {
                stats.fuzz_cases += 1;
                match parent::run_parent(child) {
                    parent::ParentStatus::Clean => {
                        // just handle stats
                    }

                    parent::ParentStatus::Crash(rip) => {
                        stats.crashes += 1;
                        let crash_filename = format!("crash_{}.jpg", rip);
                        fs::copy(FILE, crash_filename)
                            .expect("Failed to save crash file");
                        //break;
                    }
                }
            }
            
            Err(err) => {
                panic!("[main] fork() failed: {}", err);
            }
        };

        if stats.fuzz_cases % 100 == 0 {
        let elapsed = start.elapsed().as_secs_f64();
        print!("[{:10.6}] cases {:10} | fcps  {:10.2} | crashes {:10}\n",
                elapsed, stats.fuzz_cases, 
                stats.fuzz_cases as f64/ elapsed, stats.crashes);
        }
    }
}