use crate::core::{Debugger, DebuggerState};
use capstone::prelude::*;
use nix::sys::ptrace::{self, getregs};
use std::io;
use log::{debug};

use crate::memory::read_process_memory;
pub struct CommandHandler<'a> {
    pub debugger: &'a mut Debugger,
}

impl<'a> CommandHandler<'a> {
    pub fn get_command(&self) -> String {
        println!("Enter Command:");

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();
        return command.to_string();
    }

    pub fn handle_command(&mut self, command: &str) {
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
                            self.debugger
                                .breakpoint
                                .set_breakpoint(breakpoint_addr, self.debugger.process.pid);
                        }
                        Err(_) => {
                            if let Some(function) = self
                                .debugger
                                .functions
                                .iter()
                                .find(|function| function.name == arg)
                            {
                                debug!(
                                    "Found function, setting bp on {}, addr: {:#x}",
                                    arg, function.offset
                                );
                                self.debugger.breakpoint.set_breakpoint(
                                    function.offset + self.debugger.process.base_addr,
                                    self.debugger.process.pid,
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
                            self.debugger
                                .breakpoint
                                .remove_breakpoint(breakpoint_addr, self.debugger.process.pid);
                        }
                        Err(e) => {
                            println!("rm breakpoint failed: {}", e);
                        }
                    }
                } else {
                    println!("No seconda argument provided for rm breakpoint");
                }
            }
            Some("step") => {
                ptrace::step(self.debugger.process.pid, None).expect("Single-step failed")
            }
            Some("over") => self.step_over(),
            Some("instr") => self.dissasembl_instructions(),

            Some("show-bp") => self.debugger.breakpoint.show_breakpoints(),
            Some("cont") => self.cont(),
            Some("regs") => self.print_registers(),
            Some("offset") => self.print_offset(),
            Some("bt") => self.backtrace(),
            Some("exit") => self.exit(),
            _ => println!("command not found {}", command),
        }
    }
    
    fn backtrace(&self){
        let regs = getregs(self.debugger.process.pid).unwrap();

    }
    
    fn print_offset(&self){
        let regs = getregs(self.debugger.process.pid).unwrap();
        println!("{}", regs.rip - self.debugger.process.base_addr);
    }

    fn step_over(&mut self) {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");

        let regs = getregs(self.debugger.process.pid).unwrap();


        let rip = regs.rip;

        let num_bytes = 10;
        let mut code = vec![0u8; num_bytes];

        if !read_process_memory(self.debugger.process.pid, rip as usize, &mut code) {
            println!("Failed to read memory at 0x{:x}", rip);
            return;
        }
        let insns = cs.disasm_all(&code, rip).expect("Disassembly failed");
        let next_inst = insns.iter().next().unwrap();
        if next_inst.mnemonic() == Some("call") {
            let next_addr = rip + next_inst.len() as u64;
            self.debugger
                .breakpoint
                .set_breakpoint(next_addr, self.debugger.process.pid);
            self.cont();
        } else {
            ptrace::step(self.debugger.process.pid, None).expect("Single-step failed");
        }
    }

    fn dissasembl_instructions(&self) {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");

        let regs = getregs(self.debugger.process.pid).unwrap();

        let rip = regs.rip;

        let num_bytes = 64;
        let mut code = vec![0u8; num_bytes];

        if !read_process_memory(self.debugger.process.pid, rip as usize, &mut code) {
            println!("Failed to read memory at 0x{:x}", rip);
            return;
        }
        
        debug!("{:?}", code);

        let insns = cs.disasm_all(&code, rip).expect("Disassembly failed");

        for i in insns.iter() {
            println!(
                "0x{:x}: {}\t{}",
                i.address(),
                i.mnemonic().unwrap_or(""),
                i.op_str().unwrap_or("")
            );
            self.debugger.dwarf.get_line_and_file(i.address() - self.debugger.process.base_addr);
        }
    }

    fn cont(&mut self) {
        println!("Continuing execution...");
        ptrace::cont(self.debugger.process.pid, None).expect("Cont fucntion failed");
        self.debugger.state = DebuggerState::AwaitingTrap;
    }

    fn exit(&self) {
        println!("Exiting the debugger...");
        std::process::exit(0);
    }

    fn print_registers(&self) {
        match getregs(self.debugger.process.pid) {
            Ok(regs) => println!("Registers: {:?}", regs),
            Err(err) => println!("Failed to get registers: {}", err),
        }
    }
    
    #[allow(dead_code)]
    fn print_file_and_line(&self){
        let regs = getregs(self.debugger.process.pid).unwrap();

        let rip = regs.rip;

        self.debugger.dwarf.get_line_and_file(rip - self.debugger.process.base_addr);
       
    }
}
