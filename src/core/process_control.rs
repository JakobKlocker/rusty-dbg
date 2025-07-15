use crate::core::Debugger;
use anyhow::Result;
use log::info;
use nix::sys::wait::{waitpid, WaitStatus};

pub trait ProcessControl {
    fn resume_and_wait(&mut self);
    fn handle_sigtrap(&mut self);
}

impl ProcessControl for Debugger {
    fn resume_and_wait(&mut self) {
        let status = waitpid(self.process.pid, None);
        match status {
            Ok(WaitStatus::Exited(_, exit_status)) => {
                println!("Process exited with status: {}", exit_status);
            }
            Ok(WaitStatus::Stopped(_, signal)) => {
                let regs = getregs(self.process.pid).unwrap();
                if let Some(function_name) =
                    self.get_function_name(regs.rip - self.process.base_addr)
                {
                    println!(
                        "Process stopped by signal: {:?} at addr: 0x{:x} ({})",
                        signal,
                        regs.rip - 1,
                        function_name
                    );
                } else {
                    println!(
                        "Process stopped by signal: {:?} at addr: 0x{:x}",
                        signal,
                        regs.rip - 1
                    )
                }

                if signal != nix::sys::signal::Signal::SIGTRAP {
                    //temp for now, thats why sigtrap check below stays
                    nix::sys::ptrace::cont(self.process.pid, None)
                        .expect("Failed to continue process");
                    return;
                }
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

    fn handle_sigtrap(&mut self) {
        // remove BP and replace with org. once hit
        let mut regs = getregs(self.process.pid).unwrap();
        let cur_addr = regs.rip - 1;
        info!("Sigtrap HANDLE Cur Addr: 0x{:x}", cur_addr);
        if self.breakpoint.is_breakpoint(cur_addr) {
            self.breakpoint
                .remove_breakpoint(cur_addr, self.process.pid)
                .unwrap();
            regs.rip -= 1;
            let _ = setregs(self.process.pid, regs);
        }
    }
}
