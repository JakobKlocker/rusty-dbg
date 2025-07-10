use crate::core::{Debugger, DebuggerState};
use anyhow::{bail, Result};
use capstone::prelude::*;
use log::debug;
use nix::sys::ptrace::{self, getregs};
use rustyline::{DefaultEditor, error::ReadlineError};

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
        let mut parts = command.split_whitespace();
        let command_word = parts.next();

        match command_word {
            Some("bp") | Some("b") => {
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
            Some("rm-bp") | Some("rm") => {
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
            Some("dump") => {
                match parts.next() {
                    Some(size_str) => {
                        let size = usize::from_str_radix(size_str, 10); //only dec for now
                        match size {
                            Ok(size) => self.dump_hex(size),
                            _ => self.dump_hex(64),
                        }
                    }
                    None => self.dump_hex(64),
                }
            }
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
            Some("write") => {
                let args: Vec<&str> = parts.collect();
                if args.len() != 2 {
                    println!("Usage: write <addr> <value>");
                    return;
                }

                let addr = if args[0].starts_with("0x") {
                    u64::from_str_radix(&args[0][2..], 16)
                } else {
                    u64::from_str_radix(args[0], 10)
                };

                let value = if args[1].starts_with("0x") {
                    i64::from_str_radix(&args[1][2..], 16)
                } else {
                    i64::from_str_radix(args[1], 10)
                };

                match (addr, value) {
                    (Ok(addr), Ok(value)) => match self.write(addr, value) {
                        Ok(()) => println!("writing value: {} to address: {:x}", value, addr),
                        Err(e) => println!("ptrace write failed with error {}", e),
                    },
                    _ => {
                        println!("Usage: write <addr> <value>");
                    }
                }
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
            Some("step") | Some("s") => {
                ptrace::step(self.debugger.process.pid, None).expect("Single-step failed")
            }
            Some("over") | Some("o") => self.step_over(),
            Some("instr") => self.dissasembl_instructions(),

            Some("show-bp") | Some("show") => self.debugger.breakpoint.show_breakpoints(),
            Some("continue") | Some("c") => self.cont(),
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

    fn dump_hex(&self, size: usize) {
        let mut buf = vec![0u8; size as usize];
        let regs = getregs(self.debugger.process.pid).unwrap();
        match read_process_memory(self.debugger.process.pid, regs.rip as usize, &mut buf) {
            Ok(_) => {}
            Err(e) => println!("read process memory failed with error {}", e),
        }

        for (i, chunk) in buf.chunks(16).enumerate() {
            print!("0x{:08X}: ", regs.rip as usize + i * 16);

            for byte in chunk {
                print!("{:02X} ", byte);
            }
            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }
            print!("|");

            for byte in chunk {
                let c = *byte as char;
                if c.is_ascii_graphic() || c == ' ' {
                    print!("{}", c);
                } else {
                    print!(".");
                }
            }
            println!("|");
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

        match read_process_memory(self.debugger.process.pid, rip as usize, &mut code) {
            Ok(_) => {}
            Err(e) => println!("read process memory failed with error {}", e),
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
        match read_process_memory(self.debugger.process.pid, rip as usize, &mut code) {
            Ok(_) => {}
            Err(e) => println!("read process memory failed with error {}", e),
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
            self.debugger
                .dwarf
                .get_line_and_file(i.address() - self.debugger.process.base_addr);
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
