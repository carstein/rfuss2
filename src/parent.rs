// Code that runs only for parent
use nix::sys::ptrace;
use nix::sys::signal::Signal;
use nix::unistd::{Pid};
use nix::sys::wait::{WaitStatus, wait};

use std::ffi::c_void;
use std::collections::HashMap;


pub enum ParentStatus {
  Clean,
  Crash(u64)
}

pub fn set_breakpoint(pid: Pid, addr: u64) -> i64 {
  // Read 8 bytes from the process memory
  let value = ptrace::read(pid, addr as *mut c_void).unwrap();

  // Insert breakpoint by write new values
  let bp = (value & (i64::MAX ^ 0xFF)) | 0xCC;

  unsafe {
      ptrace::write(pid, addr as *mut c_void, bp as *mut c_void).unwrap();
  }

  // Return original bytecode
  value
}

pub fn restore_breakpoint(pid: Pid, addr: u64, orig_value: i64) {
  unsafe {
      // Restore original bytecode
      ptrace::write(pid, addr as *mut c_void, orig_value as *mut c_void).unwrap();
  }
}

fn handle_sigstop(pid: Pid, saved_values: &HashMap<u64, i64>) {
  let mut regs = ptrace::getregs(pid).unwrap();
  println!("Hit breakpoint at 0x{:x}", regs.rip - 1);

  match saved_values.get(&(regs.rip - 1)) {
      Some(orig) => {
          restore_breakpoint(pid, regs.rip - 1, *orig);

          // rewind rip
          regs.rip -= 1;
          ptrace::setregs(pid, regs).expect("Error rewinding RIP");

      }
      _ => print!("Nothing saved here"),
  }

  ptrace::cont(pid, None).expect("Restoring breakpoint failed");

}

pub fn run_parent(pid: Pid, mapping: &HashMap<u64, i64>) -> ParentStatus {
    loop {
        match wait() {
            Ok(WaitStatus::Stopped(pid_t, signal)) => {
                match signal {
                    Signal::SIGTRAP => {
                      handle_sigstop(pid, mapping);
                      ptrace::cont(pid, None).expect("Failed to continue process");
                    }
                    
                    Signal::SIGSEGV => {
                      let regs = ptrace::getregs(pid_t).unwrap();
                      return ParentStatus::Crash(regs.rip)
                    }
                    _ => {
                      println!("Some other signal - {}", signal);
                      break
                    }
                }
            },

            Ok(WaitStatus::Exited(_, _)) => {
                return ParentStatus::Clean;
            },

            Ok(status) =>  {
                println!("Received status: {:?}", status);
                ptrace::cont(pid, None).expect("Failed to deliver signal");
            },

            Err(err) => {
                println!("Some kind of error - {:?}",err);
            
            },
        }
    }
  return ParentStatus::Clean;
}