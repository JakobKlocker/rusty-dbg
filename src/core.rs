use crate::breakpoint::*;
use crate::dwarf::*;
use crate::functions::*;
use crate::process::*;
use log::{debug, info};
use nix::sys::ptrace::getregs;
use nix::sys::ptrace::setregs;
use nix::sys::wait::{waitpid, WaitStatus};
use std::path::Path;
use std::process::Command;

use crate::command::CommandHandler;

#[derive(Debug, Clone)]
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
    pub dwarf: DwarfContext,
    pub path: String,
}

impl Debugger {
    pub fn new(debugee_pid_path: String, debuger_name: String) -> Self {
        let pid = get_pid_from_input(debugee_pid_path.clone());

        Debugger {
            process: Process::attach(pid),
            breakpoint: Breakpoint::new(),
            functions: FunctionInfo::new(&debugee_pid_path, debuger_name),
            state: DebuggerState::Interactive,
            dwarf: DwarfContext::new(&debugee_pid_path).unwrap(),
            path: debugee_pid_path,
        }
    }

    pub fn run(&mut self) {
        loop {
            let state = self.state.clone();
            match state {
                DebuggerState::AwaitingTrap => self.resume_and_wait(),
                DebuggerState::Interactive => {
                    let mut handler = CommandHandler { debugger: self };
                    let input = handler.get_command();
                    handler.handle_command(input.as_str());
                }
            }
            info!("state: {:?}", self.state);
        }
    }

    fn handle_sigtrap(&mut self) {
        // remove BP and replace with org. once hit
        let mut regs = getregs(self.process.pid).unwrap();
        let cur_addr = regs.rip - 1;
        info!("Sigtrap HANDLE Cur Addr: 0x{:x}", cur_addr);
        if self.breakpoint.is_breakpoint(cur_addr) {
            self.breakpoint
                .remove_breakpoint(cur_addr, self.process.pid);
            regs.rip -= 1;
            let _ = setregs(self.process.pid, regs);
            //ptrace::step(self.process.pid, None).expect("Single-step after breakpoint failed");
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
                self.state = DebuggerState::Interactive;
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
        debug!("{:?}", self.functions);
    }

    pub fn get_function_name(&self, target_addr: u64) -> Option<String> {
        self.functions
            .iter()
            .find(|f| f.offset <= target_addr && f.offset + f.size > target_addr)
            .map(|f| f.name.clone())
    }
}

fn get_pid_from_input(input: String) -> i32 {
    if Path::new(&format!("/proc/{}", input)).is_dir() {
        info!("{} is a pid", input);
        input.parse().expect("Failed to parse PID")
    } else if Path::new(&input).is_file() {
        info!("{} is a file", input);
        info!("Executing {}", input);
        let child = Command::new(input).spawn().unwrap();
        child.id() as i32
    } else {
        panic!("provided pid|path not valid");
    }
}
