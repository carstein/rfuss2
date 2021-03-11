// Mutator and samples
use std::{cmp, fs};
use std::vec::Vec;

use std::collections::BTreeSet;

use rand::prelude::*;

#[derive(Debug, Clone)]
enum MutationMethod {
    Raw,
    BitFlip,
    ByteFlip,
    InsertBlock,
    RemoveBlock,
}

pub struct Mutator {
    corpus:    Vec<Sample>, // corpus 
    samples:   Vec<Sample>, // latest mutation round

    // Trace
    trace_list: BTreeSet<Vec<u64>>,

    // associated rng
    rng: rand::prelude::ThreadRng,
}

// Simplified mutator
impl Mutator {
    // Create new mutator and initialize all the fields
    pub fn new() -> Self {
        Mutator {
            corpus:         vec!(),
            samples:        vec!(),

            trace_list:     BTreeSet::new(),

            rng: thread_rng()
        }
    }

    // Consume initial corpus; move them to samples first.
    pub fn consume(&mut self, corpus: &Vec<Vec<u8>>) {
        for entry in corpus {
            self.samples.push(Sample::new(entry));
        }
    }

    // Consume sample with added trace 
    pub fn update(&mut self, samples: &Vec<Sample>) {
        for sample in samples {
            match &sample.trace {
                Some(trace) => {
                    if !self.trace_list.contains(trace) {
                        println!("[-] New coverage for input {:?} [{:?}]", sample.data, sample.method);
                        self.corpus.push(sample.clone());
                        self.trace_list.insert(trace.clone());
                    }
                },
                None => {
                    println!("[!] missing trace info ...");
                }
            }
        }
        self.mutate()
    }

    // Mutate mutation pool to create a set of new samples
    fn mutate(&mut self) {
        for sample in &mut self.corpus {
            for _ in 0..100 { //completely arbitraty number
                &self.samples.push(sample.mutate(&mut self.rng));
            }
        }
    }
}

impl Iterator for Mutator {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        self.samples.pop()
    }
}

// Individual sample
#[derive(Clone)]
pub struct Sample {
    version: u32,
    data: Vec<u8>,
    method: MutationMethod,
    trace: Option<Vec<u64>>,
}


impl Sample {
    fn new(content: &Vec<u8>) -> Self {
        Sample {
            version:1, 
            data: content.clone(), 
            method: MutationMethod::Raw, 
            trace: None 
        }
    }

    pub fn materialize_sample(&self, filename: &str) {
      fs::write(filename, &self.data).expect("error saving file");
    }

    pub fn add_trace(&mut self, trace: Vec<u64>) {
        self.trace = Some(trace)
    }

    fn mutate(&mut self, rng: &mut ThreadRng) -> Sample {

        let strategy: u32 = rng.gen_range(0..=3);
        let index: usize = rng.gen_range(0..self.data.len());

        match strategy {
            0 => self.bit_flip(rng, index),
            1 => self.byte_flip(rng, index),
            2 => self.insert_block(rng, index),
            3 => self.remove_block(rng, index),
            _ => self.raw(),
        }
    }

    fn raw(&self) -> Sample {
        Sample {
            version: &self.version + 1,
            data: self.data.to_vec(),
            method: MutationMethod::Raw,
            trace: None,
        }
    }

    fn bit_flip(&self, rng: &mut ThreadRng, idx: usize) -> Sample {
        let mut bytecode = self.data.to_vec();
        let b = [1, 2, 4, 8,16, 32, 64, 128].choose(rng).unwrap();
        bytecode[idx] ^= b;

        Sample {
            version: &self.version + 1,
            data: bytecode,
            method: MutationMethod::BitFlip,
            trace: None,
        }
    }

    fn byte_flip(&self, rng: &mut rand::prelude::ThreadRng, idx: usize) -> Sample {
        let mut bytecode = self.data.to_vec();

        let b: u8 = rng.gen::<u8>();
        bytecode[idx] = b;

        Sample {
            version: &self.version + 1,
            data: bytecode,
            method: MutationMethod::ByteFlip,
            trace: None,
        }
    }

    fn insert_block(&self, rng: &mut rand::prelude::ThreadRng, idx: usize) -> Sample {
        let mut bytecode = self.data.to_vec();

        // Insert random block of data in range of 1-8 bytes
        // for i in 0..rng.gen_range(1..8) {
        //     bytecode.insert(idx+i, rng.gen::<u8>());
        // }
        bytecode.insert(idx, rng.gen::<u8>());

        Sample {
            version: &self.version + 1,
            data: bytecode,
            method: MutationMethod::InsertBlock,
            trace: None,
        }
    }

    fn remove_block(&self, rng: &mut rand::prelude::ThreadRng, idx: usize) -> Sample {
        let mut bytecode = self.data.to_vec();

        // Remove random block of data in range of 1-8 bytes
        let upper_bound = cmp::min(rng.gen_range(1..8), bytecode.len() - idx);

        for _ in 0..upper_bound {
            bytecode.remove(idx);
        }

        Sample {
            version: &self.version + 1,
            data: bytecode,
            method: MutationMethod::RemoveBlock,
            trace: None,
        }
    }
}