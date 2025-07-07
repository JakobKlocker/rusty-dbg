use crate::breakpoint::*;
use crate::functions::*;
use crate::process::*;
use capstone::prelude::*;
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use nix::sys::ptrace::setregs;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::debugger::command::CommandHandler;

#[derive(Debug)]
pub enum DebuggerState {
    Interactive,
    AwaitingTrap,
}

#[derive(Debug)]
pub struct Debugger {
    pub process: Process,
    pub breakpoint: Breakpoint,
    pub functions: Vec<FunctionInfo>,
    pub state: DebuggerState,
}

impl Debugger {
    pub fn new(debugee_pid_path: String, debuger_name: String) -> Self {
        let pid = get_pid_from_input(debugee_pid_path.clone());

        Debugger {
            process: Process::attach(pid),
            breakpoint: Breakpoint::new(),
            functions: FunctionInfo::new(debugee_pid_path, debuger_name),
            state: DebuggerState::Interactive,
        }
    }

    fn read_process_memory(&self, pid: Pid, addr: usize, buf: &mut [u8]) -> bool {
        let local = iovec {
            iov_base: buf.as_mut_ptr() as *mut _,
            iov_len: buf.len(),
        };

        let remote = iovec {
            iov_base: addr as *mut _,
            iov_len: buf.len(),
        };

        let result = unsafe { process_vm_readv(pid.as_raw() as pid_t, &local, 1, &remote, 1, 0) };

        result == buf.len() as isize
    }

    fn handle_sigtrap(&mut self) {
        // remove BP and replace with org. once hit
        let mut regs = getregs(self.process.pid).unwrap();
        let cur_addr = regs.rip - 1;
        println!("Sigtrap HANDLE Cur Addr: 0x{:x}", cur_addr);
        if self.breakpoint.is_breakpoint(cur_addr) {
            self.breakpoint
                .remove_breakpoint(cur_addr, self.process.pid);
            regs.rip -= 1;
            setregs(self.process.pid, regs);
            ptrace::step(self.process.pid, None).expect("Single-step after breakpoint failed");
        }
        self.state = DebuggerState::Interactive;
    }

    pub fn run(&mut self) {
        loop {
            let mut handler = CommandHandler { debugger: self };
            match (self.state) {
                DebuggerState::AwaitingTrap => self.resume_and_wait(),
                DebuggerState::Interactive => {
                    let input = handler.get_command();
                    handler.handle_command(input.as_str());
                }
            }
        }
    }

    fn resume_and_wait(&mut self) {
        let status = waitpid(self.process.pid, None);
        match status {
            Ok(WaitStatus::Exited(_, exit_status)) => {
                println!("Process exited with status: {}", exit_status);
            }
            Ok(WaitStatus::Stopped(_, signal)) => {
                let regs = getregs(self.process.pid).unwrap();
                println!(
                    "Process stopped by signal: {:?} at addr: 0x{:x}",
                    signal,
                    regs.rip - 1
                );
                if signal == nix::sys::signal::Signal::SIGTRAP {
                    self.handle_sigtrap();
                }
            }
            Ok(WaitStatus::Signaled(_, signal, _)) => {
                println!("Process terminated by signal: {:?}", signal);
            }
            Ok(_) => {
                println!("Process changed state.");
            }
            Err(e) => {
                println!("Error waiting for process: {}", e);
            }
        }
    }

    pub fn print_functions(&self) {
        println!("{:?}", self.functions);
    }
}

fn get_pid_from_input(input: String) -> i32 {
    let mut pid: i32 = 0;
    if Path::new(&format!("/proc/{}", input)).is_dir() {
        println!("{} is a pid", input);
        pid = input.parse().expect("Failed to parse PID");
    } else if Path::new(&input).is_file() {
        println!("{} is a file", input);
        println!("Executing {}", input);
        let child = Command::new(input).spawn().unwrap();
        pid = child.id() as i32;
    } else {
        panic!("provided pid|path not valid");
    }
    return pid;
}
