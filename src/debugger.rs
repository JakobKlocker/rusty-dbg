use crate::breakpoint::*;
use crate::functions::*;
use crate::process::*;
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use nix::sys::wait::{waitpid, WaitStatus};
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct Debugger {
    pub process: Process,
    pub breakpoint: Breakpoint,
    pub functions: Vec<FunctionInfo>,
}

impl Debugger {
    pub fn new(debugee_pid_path: String, debuger_name: String) -> Self {
        let pid = get_pid_from_input(debugee_pid_path.clone()); // Using clone to bypass, learn rust better

        Debugger {
            process: Process::attach(pid),
            breakpoint: Breakpoint::new(),
            functions: FunctionInfo::new(debugee_pid_path, debuger_name), 
        }
    }

    fn get_command(&self) -> String {
        println!("Enter Command: ");

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();
        println!("{}", command);
        return command.to_string();
    }

    fn handle_command(&mut self, command: &str) {
        let mut parts = command.split_whitespace();
        let command_word = parts.next();

        match command_word {
            Some("breakpoint") => {
                if let Some(arg) = parts.next() {
                     let arg  = if arg.starts_with("0x")
                    {
                        &arg[2..]
                    } else{
                        arg
                    };
                    match u64::from_str_radix(arg, 16) {
                        Ok(breakpoint_addr) => {
                            println!("{:#x}", breakpoint_addr);
                            self.breakpoint
                                .set_breakpoint(breakpoint_addr, self.process.pid);
                        }
                        Err(_) => {
                            if let Some(function) =
                                self.functions.iter().find(|function| function.name == arg)
                            {
                                println!(
                                    "Found function, setting bp on {}, addr: {:#x}",
                                    arg, function.start_addr
                                );
                                self.breakpoint.set_breakpoint(
                                    function.start_addr + self.process.base_addr,
                                    self.process.pid,
                                );
                            } else {
                                println!(
                                    "Breakpoint failed, has to be addr or function name: {}",
                                    arg
                                );
                            }
                        }
                    }
                } else {
                    println!("No seconda argument provided for breakpoint");
                }
            }
            Some("rm-bp") => {
                if let Some(arg) = parts.next() {
                    let arg  = if arg.starts_with("0x")
                    {
                        &arg[2..]
                    } else{
                        arg
                    };
                  match u64::from_str_radix(arg, 16) {
                        Ok(breakpoint_addr) => {
                            println!("remove {}", breakpoint_addr);
                            self.breakpoint
                                .remove_breakpoint(breakpoint_addr, self.process.pid);
                        }
                        Err(e) => {
                            println!("rm breakpoint failed: {}", e);
                        }
                    }
                } else {
                    println!("No seconda argument provided for rm breakpoint");
                }
            }
            Some("show-bp") => self.breakpoint.show_breakpoints(),
            Some("continue") => self.cont(),
            Some("reg") => self.print_registers(),
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

    pub fn run(&mut self) {
        loop {
            let status = waitpid(self.process.pid, None);
            match status {
                Ok(WaitStatus::Exited(_, exit_status)) => {
                    println!("Process exited with status: {}", exit_status);
                    break;
                }
                Ok(WaitStatus::Stopped(_, signal)) => {
                    println!("Process stopped by signal: {:?}", signal);
                    loop {
                        let command = self.get_command();
                        self.handle_command(command.as_str());
                    }
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

    fn print_registers(&self) {
        match getregs(self.process.pid) {
            Ok(regs) => println!("Registers: {:?}", regs),
            Err(err) => println!("Failed to get registers: {}", err),
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
