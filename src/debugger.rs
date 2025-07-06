use crate::breakpoint::*;
use crate::functions::*;
use crate::process::*;
use capstone::prelude::*;
use libc::{iovec, pid_t, process_vm_readv};
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;
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
        let pid = get_pid_from_input(debugee_pid_path.clone());

        Debugger {
            process: Process::attach(pid),
            breakpoint: Breakpoint::new(),
            functions: FunctionInfo::new(debugee_pid_path, debuger_name),
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

    fn dissasembl_instructions(&self) {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");

        let regs = getregs(self.process.pid).unwrap();

        let rip = regs.rip;

        let num_bytes = 64;
        let mut code = vec![0u8; num_bytes];

        if !self.read_process_memory(self.process.pid, rip as usize, &mut code) {
            println!("Failed to read memory at 0x{:x}", rip);
            return;
        }
        let insns = cs.disasm_all(&code, rip).expect("Disassembly failed");

        for i in insns.iter() {
            println!(
                "0x{:x}: {}\t{}",
                i.address(),
                i.mnemonic().unwrap_or(""),
                i.op_str().unwrap_or("")
            );
        }
    }

    fn step_over(&mut self) {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");

        let regs = getregs(self.process.pid).unwrap();

        let rip = regs.rip;

        let num_bytes = 10;
        let mut code = vec![0u8; num_bytes];

        if !self.read_process_memory(self.process.pid, rip as usize, &mut code) {
            println!("Failed to read memory at 0x{:x}", rip);
            return;
        }
        let insns = cs.disasm_all(&code, rip).expect("Disassembly failed");
        let next_inst = insns.iter().next().unwrap();
        if next_inst.mnemonic() == Some("call"){
            let next_addr = rip + next_inst.len() as u64;
            println!("next addr: {}", next_addr);
            self.breakpoint.set_breakpoint(next_addr, self.process.pid);
            self.cont();
        } else {
            ptrace::step(self.process.pid, None).expect("Single-step failed");
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
            Some("bp") => {
                if let Some(arg) = parts.next() {
                    let arg = if arg.starts_with("0x") {
                        &arg[2..]
                    } else {
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
                                    arg, function.offset
                                );
                                self.breakpoint.set_breakpoint(
                                    function.offset + self.process.base_addr,
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
                    println!("No second argument provided for breakpoint");
                }
            }
            Some("rm-bp") => {
                if let Some(arg) = parts.next() {
                    let arg = if arg.starts_with("0x") {
                        &arg[2..]
                    } else {
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
            Some("step") => ptrace::step(self.process.pid, None).expect("Single-step failed"),
            Some("over") => self.step_over(),
            Some("instr") => self.dissasembl_instructions(),

            Some("show-bp") => self.breakpoint.show_breakpoints(),
            Some("cont") => self.cont(),
            Some("regs") => self.print_registers(),
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
