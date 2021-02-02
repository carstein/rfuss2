// Mutator and samples
use std::{cmp, fs};
use std::vec::Vec;

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
    core_samples:    Vec<Sample>, // Reservoir of original samples
    corpus:          Vec<Sample>, // Samples with coverage data
    mutation_pool:   Vec<Sample>, // Set of samples for future mutation
    samples:         Vec<Sample>, // Mutated samples; ready to be fed to engine

    // associated rng
    rng: rand::prelude::ThreadRng,
}

impl Mutator {
    // Create new mutator and initialize all the fields
    pub fn new() -> Self {
        Mutator {
            core_samples:   vec!(),
            corpus:         vec!(),
            mutation_pool:  vec!(),
            samples:        vec!(),

            rng: thread_rng()
        }
    }

    // Consume data from file as a fresh sample
    pub fn feed(&mut self, data: Vec<u8>) {
        self.core_samples.push(Sample::new(data));
    }

    // Mutate mutation pool to create a set of new samples
    fn mutate(&mut self) {
        if self.mutation_pool.len() == 0 {
            self.fit_function();
        }

        for sample in &mut self.mutation_pool {
            for _ in 0..100 { //completely arbitraty number
                &self.samples.push(sample.mutate(&mut self.rng));
            }
        }
    }

    fn fit_function(&mut self) {
        // Add samples from unmodified pool
        for sample in &mut self.core_samples {
            self.mutation_pool.push(sample.clone())
        }

        // Select elements from corpus for future mutation

        // Backfill to 100

        // Update trace info
        
        // clean the corpus and mutation_pool
        self.corpus.clear();
    }
}

impl Iterator for Mutator {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {

        if self.samples.len() == 0 {
            self.mutate();
        }

        self.samples.pop()
    }
}

// Individual sample
#[derive(Clone)]
pub struct Sample {
    version: u32,
    data: Vec<u8>,
    method: MutationMethod,

}

impl Sample {
    fn new(content: Vec<u8>) -> Self {
        Sample {version:1, data: content, method: MutationMethod::Raw }
    }

    pub fn materialize_sample(&self, filename: &str) {
      fs::write(filename, &self.data).expect("error saving file");
    }

    fn mutate(&mut self, rng: &mut ThreadRng) -> Sample {

        let strategy: u32 = rng.gen_range(0, 3);
        let index: usize = rng.gen_range(0, self.data.len());

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
        }
    }

    fn insert_block(&self, rng: &mut rand::prelude::ThreadRng, idx: usize) -> Sample {
        let mut bytecode = self.data.to_vec();

        // Insert random block of data in range of 1-8 bytes
        for i in 0..rng.gen_range(1, 8) {
            bytecode.insert(idx+i, rng.gen::<u8>());
        }

        Sample {
            version: &self.version + 1,
            data: bytecode,
            method: MutationMethod::InsertBlock,
        }
    }

    fn remove_block(&self, rng: &mut rand::prelude::ThreadRng, idx: usize) -> Sample {
        let mut bytecode = self.data.to_vec();

        // Remove random block of data in range of 1-8 bytes
        let upper_bound = cmp::min(rng.gen_range(1, 8), bytecode.len() - idx);

        for _ in 0..upper_bound {
            bytecode.remove(idx);
        }

        Sample {
            version: &self.version + 1,
            data: bytecode,
            method: MutationMethod::RemoveBlock,
        }
    }
}