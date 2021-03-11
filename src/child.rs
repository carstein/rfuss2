//extern crate linux_personality;
use nix::sys::ptrace;
use nix::unistd::Pid;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::sys::signal::Signal;
use std::collections::HashMap;

use std::fs;
use std::path::Path;
use std::io::{BufRead, BufReader};

use linux_personality::personality;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::config;
use crate::parent;


fn get_executable_base(filename: String) -> Option<u64> {
  let filename = Path::new(&filename);

  if filename.is_file() {
      let fh = fs::File::open(filename).unwrap();
      let reader = BufReader::new(fh);

      for line in reader.lines() {
          let line = line.unwrap();
          let fields: Vec<&str> = line.split_whitespace().collect();

          if fields[1].contains("x") {
              let addr: Vec<&str>  = fields[0].split("-").collect();
              let base: u64 = u64::from_str_radix(addr[0], 16)
                  .expect("Failed parsing base address");

              return Some(base);
          }
      }
      None
  } else {
      None
  }
}


pub fn run_child(config: &config::RuntimeConfig, bpmap: &mut HashMap<u64, i64>, filename: &str) -> Pid {
  // Execute binary spawning new process
  let child = unsafe { Command::new(&config.prog_name)
      .arg(filename)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .pre_exec(|| {
        ptrace::traceme().expect("Process doesn't want to be traced ...");
        personality(linux_personality::ADDR_NO_RANDOMIZE).unwrap();
        Ok(())
      })
      .spawn()
      .expect("[!] Failed to run process")
    };

    let res = Pid::from_raw(child.id() as i32);

    match waitpid(res, None) {
      Ok(WaitStatus::Stopped(_, Signal::SIGTRAP)) => {
        // Get file base
        let base = get_executable_base(format!("/proc/{}/maps", res))
          .expect("[!] Failed to get executable base!");

        // Pre-set all breakpoints for coverage tracking
        for bp in &config.breakpoint_map {
          bpmap.insert(bp+base, parent::set_breakpoint(res, bp+base));
        }

        ptrace::cont(res, None).expect("Should have continued");
      }
      _ => println!("COULD NOT START"),
  }

  res

}