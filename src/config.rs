// Runtime config functions
use clap::ArgMatches;
use std::fs;
use std::path::Path;
use std::io::{BufRead, BufReader};

pub struct RuntimeConfig {
  pub prog_name: String,
  pub corpus: Vec<Vec<u8>>,
  pub breakpoint_map: Vec<u64>,
}

pub fn generate_runtime_config(options: ArgMatches) -> RuntimeConfig {
  let corpus_path = options.value_of("corpus").unwrap();
  let map_path = options.value_of("bpmap").unwrap();

  RuntimeConfig {
      prog_name: String::from(options.value_of("prog_name").unwrap()),
      corpus: read_corpus(Path::new(corpus_path)),
      breakpoint_map: parse_breakpoint_map(Path::new(map_path)).unwrap(),
  }
}

fn read_corpus(entry: &Path) -> Vec<Vec<u8>> {
  print!("[#] Reading corpus files ... ");
  let mut files: Vec<Vec<u8>> = vec!();

  if entry.is_dir() {
      for f in fs::read_dir(entry).unwrap() {
          let data = fs::read(f.unwrap().path()).unwrap();
          files.push(data.to_vec())
      }

  } else {
      files.push(fs::read(entry).unwrap())
  }

  println!("{} files in total", files.len());
  files
}

fn parse_breakpoint_map(filename: &Path) -> Option<Vec<u64>> {
  print!("[#] Reading breakpoint map ... ");
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

      println!(" {} breakpoints", bp_map.len());
      Some(bp_map)
  } else {
      println!("empty");
      None
  }
}