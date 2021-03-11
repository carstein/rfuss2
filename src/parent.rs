// Code that runs only for parent
use nix::sys::ptrace;
use nix::sys::signal::Signal;

use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{Pid};

use std::collections::HashMap;
use std::ffi::c_void;

pub enum ParentStatus {
  Clean(Vec<u64>),
  Crash(u64)
}

pub fn set_breakpoint(pid: Pid, addr: u64) -> i64 {
  // Read 8 bytes from the process memory
  let value = ptrace::read(pid, (addr) as *mut c_void).unwrap() as u64;

  // Insert breakpoint by write new values
  let bp = (value & (u64::MAX ^ 0xFF)) | 0xCC;

  unsafe {
      ptrace::write(pid, addr as *mut c_void, bp as *mut c_void).unwrap();
  }

  // Return original bytecode
  value as i64
}

pub fn restore_breakpoint(pid: Pid, addr: u64, orig_value: i64) {
  unsafe {
      // Restore original bytecode
      ptrace::write(pid, addr as *mut c_void, orig_value as *mut c_void).unwrap();
  }
}

fn handle_sigstop(pid: Pid, saved_values: &HashMap<u64, i64>, trace: &mut Vec<u64>) {
  let mut regs = ptrace::getregs(pid).unwrap();

  match saved_values.get(&(regs.rip - 1)) {
      Some(orig) => {
          restore_breakpoint(pid, regs.rip - 1, *orig);

          // rewind rip by one
          regs.rip -= 1;

          //storing trace info
          trace.push(regs.rip);
          ptrace::setregs(pid, regs).expect("Error rewinding RIP");

      }
      None => {
      }
  }

  ptrace::cont(pid, None).expect("Restoring breakpoint failed");
}

pub fn run_parent(pid: Pid, mapping: &HashMap<u64, i64>) -> ParentStatus {
  let mut trace: Vec<u64> = vec!();

  loop {
      match waitpid(pid, None) {
          Ok(WaitStatus::Stopped(pid_t, signal)) => {
              match signal {
                  Signal::SIGTRAP => {
                    handle_sigstop(pid, mapping, &mut trace);
                  }
                  
                  Signal::SIGSEGV => {
                    let regs = ptrace::getregs(pid_t).unwrap();
                    return ParentStatus::Crash(regs.rip)
                  }
                  _ => {
                    println!("Some other signal - {}", signal);
                    let regs = ptrace::getregs(pid).unwrap();
                    println!("Error at 0x{:x}", regs.rip - 1);
                    break
                  }
              }
          },

          Ok(WaitStatus::Exited(_, _)) => {
              return ParentStatus::Clean(trace);
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

  return ParentStatus::Clean(trace);
}