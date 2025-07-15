use crate::core::{Debugger, DebuggerState};
use anyhow::{bail, Result};
use capstone::prelude::*;
use log::debug;
use nix::sys::ptrace::{self, getregs};
use object::Object;
use object::ObjectSection;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::fs;

use crate::commands::CommandRouter;
use crate::memory::read_process_memory;
use crate::stack_unwind::*;
pub struct CommandHandler<'a> {
    pub debugger: &'a mut Debugger,
}

impl<'a> CommandHandler<'a> {
    pub fn get_command(&self) -> String {
        let mut rl = DefaultEditor::new().unwrap();
        let _ = rl.load_history(".history");

        match rl.readline("Enter Command: ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line).unwrap();
                let _ = rl.save_history(".history");
                line
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                std::process::exit(0);
            }

            Err(err) => {
                eprintln!("Unexpected error: {:?}", err);
                String::new()
            }
        }
    }

    pub fn handle_command(&mut self, command: &str) {
        let router = CommandRouter::new();
        router.handle(&command, self.debugger);
        return;
        let mut parts = command.split_whitespace();
        let command_word = parts.next();

        match command_word {
            Some("set") | Some("change") => {
                let reg = parts.next();
                let val = parts.next();
                match (reg, val) {
                    (Some(reg), Some(val_str)) => {
                        let value = if val_str.starts_with("0x") {
                            u64::from_str_radix(&val_str[2..], 16)
                        } else {
                            u64::from_str_radix(val_str, 10)
                        };

                        match value {
                            Ok(val) => {
                                if let Err(e) = self.set_register(reg, val) {
                                    println!("Failed to set register {}: {}", reg, e);
                                } else {
                                    println!("Set register {} to 0x{:x}", reg, val);
                                }
                            }
                            Err(e) => println!("Invalid value: {}", e),
                        }
                    }
                    _ => println!("Usage: set <register> <value>"),
                }
            }
           Some("sections") => {
                self.print_sections();
            }
            Some("read") => {
                if let Some(addr) = parts.next() {
                    let addr = if addr.starts_with("0x") {
                        u64::from_str_radix(&addr[2..], 16)
                    } else {
                        u64::from_str_radix(addr, 10)
                    };
                    match addr {
                        Ok(addr) => match self.read(addr) {
                            Ok(value) => println!("value at address {:x} is {}", addr, value),
                            Err(e) => println!("read failed with error: {}", e),
                        },
                        Err(e) => {
                            println!("Invalid address format: {}", e);
                        }
                    }
                }
            }
            Some("show-bp") | Some("show") => self.debugger.breakpoint.show_breakpoints(),
            Some("registers") | Some("r") => {
                if let Some(reg) = parts.next() {
                    self.dump_register(reg);
                } else {
                    self.dump_registers();
                }
            }
            Some("offset") => self.print_offset(),
            Some("backtrace") | Some("bt") => {
                if let Err(e) = self.backtrace() {
                    println!("");
                    println!("End of backtrace: {}", e);
                }
            }
            Some("exit") => self.exit(),
            _ => println!("command not found {}", command),
        }
    }

    fn print_sections(&self) {
        let data = fs::read(self.debugger.path.clone()).unwrap();
        let obj_file = object::File::parse(&*data).unwrap();
        for section in obj_file.sections() {
            println!(
                "Section: {:<20} Addr: 0x{:08x}, Size: 0x{:x}",
                section.name().unwrap_or("<unnamed>"),
                section.address(),
                section.size(),
            );
        }
    }

    fn print_offset(&self) {
        let regs = getregs(self.debugger.process.pid).unwrap();
        let func_offset = regs.rip - self.debugger.process.base_addr;
        println!("{}", func_offset);
    }

    fn backtrace(&self) -> anyhow::Result<()> {
        let regs = getregs(self.debugger.process.pid)?;
        let mut rip = regs.rip;
        let mut rsp = regs.rsp;
        let mut rbp = regs.rbp;
        let mut first = true;

        loop {
            let mut func_offset = rip - self.debugger.process.base_addr;

            let info = get_unwind_info(&self.debugger.path, func_offset)?;
            debug!("{:?}", info);

            let cfa_base = match info.cfa_register {
                6 => rbp,
                7 => rsp,
                16 => rip,
                other => panic!("unsupported cfa reg{}", other),
            };

            let cfa = (cfa_base as i64 + info.cfa_offset) as u64;
            debug!("CFA: 0x{:016x}", cfa);

            let ret_addr_addr = (cfa as i64 + info.ra_offset) as u64;

            let ret_addr = ptrace::read(
                self.debugger.process.pid,
                ret_addr_addr as ptrace::AddressType,
            )
            .unwrap() as u64;

            debug!(
                "Return address (caller RIP): 0x{:016x}",
                ret_addr - self.debugger.process.base_addr
            );

            func_offset = ret_addr - self.debugger.process.base_addr;
            let name: String = self
                .debugger
                .get_function_name(func_offset)
                .unwrap_or_else(|| "_start".to_string());
            if first != true {
                print!("-> ");
            }
            print!("{}", name);
            rip = ret_addr;
            rsp = cfa;
            if info.cfa_register == 6 {
                let saved_rbp_addr = (cfa as i64 - 16) as u64;
                rbp = ptrace::read(
                    self.debugger.process.pid,
                    saved_rbp_addr as ptrace::AddressType,
                )
                .unwrap() as u64;
            }
            first = false;
        }
    }

    fn read(&self, addr: u64) -> Result<i64> {
        Ok(ptrace::read(
            self.debugger.process.pid,
            addr as ptrace::AddressType,
        )?)
    }
    fn write(&self, addr: u64, value: i64) -> Result<()> {
        ptrace::write(
            self.debugger.process.pid,
            addr as ptrace::AddressType,
            value,
        )?;
        Ok(())
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

    fn dump_register(&self, reg: &str) {
        match self.get_register_value(reg) {
            Ok(value) => println!("{}: 0x{:x}", reg, value),
            Err(err) => println!("Failed to get register {}", err),
        }
    }

    fn dump_registers(&self) {
        match getregs(self.debugger.process.pid) {
            Ok(regs) => println!("{:?}", regs),
            Err(err) => println!("Failed to get registers: {}", err),
        }
    }

    #[allow(dead_code)]
    fn print_file_and_line(&self) {
        let regs = getregs(self.debugger.process.pid).unwrap();

        let rip = regs.rip;

        self.debugger
            .dwarf
            .get_line_and_file(rip - self.debugger.process.base_addr);
    }

    fn get_register_value(&self, name: &str) -> Result<u64> {
        let regs = getregs(self.debugger.process.pid)?;
        let value = match name {
            "rip" => Some(regs.rip),
            "rax" => Some(regs.rax),
            "rbx" => Some(regs.rbx),
            "rcx" => Some(regs.rcx),
            "rdx" => Some(regs.rdx),
            "rsi" => Some(regs.rsi),
            "rdi" => Some(regs.rdi),
            "rsp" => Some(regs.rsp),
            "rbp" => Some(regs.rbp),
            "r8" => Some(regs.r8),
            "r9" => Some(regs.r9),
            "r10" => Some(regs.r10),
            "r11" => Some(regs.r11),
            "r12" => Some(regs.r12),
            "r13" => Some(regs.r13),
            "r14" => Some(regs.r14),
            "r15" => Some(regs.r15),
            "eflags" => Some(regs.eflags),
            _ => None,
        };

        value.ok_or_else(|| anyhow::anyhow!("Unkown Register: {}", name))
    }

    fn set_register(&self, name: &str, value: u64) -> Result<()> {
        let mut regs = ptrace::getregs(self.debugger.process.pid)?;
        match name {
            "rip" => regs.rip = value,
            "rax" => regs.rax = value,
            "rbx" => regs.rbx = value,
            "rcx" => regs.rcx = value,
            "rdx" => regs.rdx = value,
            "rsi" => regs.rsi = value,
            "rdi" => regs.rdi = value,
            "rsp" => regs.rsp = value,
            "rbp" => regs.rbp = value,
            "r8" => regs.r8 = value,
            "r9" => regs.r9 = value,
            "r10" => regs.r10 = value,
            "r11" => regs.r11 = value,
            "r12" => regs.r12 = value,
            "r13" => regs.r13 = value,
            "r14" => regs.r14 = value,
            "r15" => regs.r15 = value,
            "eflags" => regs.eflags = value,
            _ => bail!("Unknown register: {}", name),
        }

        ptrace::setregs(self.debugger.process.pid, regs)?;
        Ok(())
    }
}
