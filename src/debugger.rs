use crate::process;
use nix::sys::wait::{waitpid, WaitStatus};
pub struct Debugger {
    process: process::Process,
    // breakpoint later here
}

impl Debugger {
    pub fn new(pid :i32) -> Self{
        Debugger{
            process: process::Process::attach(pid),
        }
    }

    pub fn run(&self){
        loop {
            let status = waitpid(self.process.pid, None);
    
            


            match status {
                Ok(WaitStatus::Exited(_, exit_status)) => {
                    println!("Process exited with status: {}", exit_status);
                    break; 
                }
                Ok(WaitStatus::Stopped(_, signal)) => {
                    println!("Process stopped by signal: {:?}", signal);
                
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    println!("Process terminated by signal: {:?}", signal);
                    break; 
                }
                Ok(_) => {
                    println!("Process changed state.");
                }
                Err(e) => {
                    println!("Error waiting for process: {}", e);
                    break;
                }
            }
        }
        println!("Debugger run complete.");
    }
}