use crate::process;
use crate::breakpoint;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::sys::ptrace;
use std::io;
pub struct Debugger {
    process: process::Process,
    breakpoint: breakpoint::Breakpoint,
}

impl Debugger {
    pub fn new(pid :i32) -> Self{
        Debugger{
            process: process::Process::attach(pid),
            breakpoint: breakpoint::Breakpoint::new(),
        }
    }

    fn get_command(&self) -> String{
        println!("Enter Command: ");

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();
        println!("{}", command);
        return command.to_string();
    }

    fn handle_command(&mut self, command: &str){
        let mut parts = command.split_whitespace();
        let command_word = parts.next();
        match command_word {
            Some("breakpoint") => {
                if let Some(arg) = parts.next(){

                    match u64::from_str_radix(arg, 16){
                        Ok(breakpoint_addr)=> {
                            println!("{}", breakpoint_addr);
                            self.breakpoint.set_breakpoint(breakpoint_addr, self.process.pid);
                        }
                        Err(e) => {
                            println!("breakpoint failed: {}", e);
                        }
                    }
                } else {
                    println!("No seconda argument provided for breakpoint");
                }
            },
            Some("continue") => self.cont(),
            Some("exit") => self.exit(),
            _ => println!("command not found {}", command),
        }
    }
    
    fn cont(&self) {
        println!("Continuing execution...");
        ptrace::cont(self.process.pid, None).expect("Cont fucntion failed");
    }
    
    fn exit(&self) {
        println!("Exiting the debugger...");
        std::process::exit(0);
    }
    

    pub fn run(&mut self){
        loop {
            let status = waitpid(self.process.pid, None);
            match status {
                Ok(WaitStatus::Exited(_, exit_status)) => {
                    println!("Process exited with status: {}", exit_status);
                    break; 
                }
                Ok(WaitStatus::Stopped(_, signal)) => {
                    println!("Process stopped by signal: {:?}", signal);
                    let command = self.get_command();
                    self.handle_command(command.as_str());
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