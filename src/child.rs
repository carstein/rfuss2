//extern crate linux_personality;
use nix::sys::ptrace;
use nix::unistd::Pid;

use linux_personality::personality;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};


pub fn run_child(prog_name: &String, filename: &str) -> Pid {
  // Execute binary spawning new process
  let child = unsafe { Command::new(prog_name)
      .arg(filename)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .pre_exec(|| {
        ptrace::traceme().expect("Process doesn't want to be traced ...");
        personality(linux_personality::ADDR_NO_RANDOMIZE).unwrap();
        Ok(())
      })
      .spawn()
      .expect("Failed to run process")
    };

    Pid::from_raw(child.id() as i32)
}