// Code that runs only for parent
use nix::sys::ptrace;
use nix::sys::signal::Signal;
use nix::unistd::{Pid};
use nix::sys::wait::{WaitStatus, wait};


pub enum ParentStatus {
  Clean,
  Crash(u64)
}

pub fn run_parent(pid: Pid) -> ParentStatus {
    loop {
        match wait() {
            Ok(WaitStatus::Stopped(pid_t, signal)) => {
                match signal {
                    Signal::SIGTRAP => {
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