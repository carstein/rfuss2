//extern crate linux_personality;
use nix::sys::ptrace;

use linux_personality::personality;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio, exit};

// Code that runs only for child
pub fn run_child(prog_name: &String, filename: &str) {
  // Allows process to be traced
  ptrace::traceme().expect("Process doesn't want to be traced ...");

  // Disable ASLR for this process
  personality(linux_personality::ADDR_NO_RANDOMIZE).unwrap();

  // Execute binary without spawning new process
  Command::new(prog_name)
    .arg(filename)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .exec();

  exit(0);
}