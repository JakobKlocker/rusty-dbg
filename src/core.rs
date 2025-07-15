use crate::breakpoint::*;
use crate::dwarf::*;
use crate::functions::*;
use crate::process::*;
use anyhow::{bail, Result};
use capstone::prelude::*;
use log::{debug, info};
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use nix::sys::ptrace::setregs;
use nix::sys::wait::{waitpid, WaitStatus};
use object::{Object, ObjectSection};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::command::CommandHandler;
use crate::memory::read_process_memory;
use crate::stack_unwind::get_unwind_info;

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

    pub fn print_functions(&self) {
        debug!("{:?}", self.functions);
    }

    pub fn print_offset(&self) {
        let regs = getregs(self.process.pid).unwrap();
        let func_offset = regs.rip - self.process.base_addr;
        println!("{}", func_offset);
    }

    pub fn list_breakpoints(&self) -> &[(u64, u8)] {
        &self.breakpoint.breakpoint
    }

    pub fn exit(&self) -> Result<()> {
        println!("Exiting the debugger...");
        std::process::exit(0);
    }

    pub fn backtrace(&self) -> anyhow::Result<()> {
        let regs = getregs(self.process.pid)?;
        let mut rip = regs.rip;
        let mut rsp = regs.rsp;
        let mut rbp = regs.rbp;
        let mut first = true;

        loop {
            let mut func_offset = rip - self.process.base_addr;

            let info = get_unwind_info(&self.path, func_offset)?;
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

            let ret_addr = ptrace::read(self.process.pid, ret_addr_addr as ptrace::AddressType)
                .unwrap() as u64;

            debug!(
                "Return address (caller RIP): 0x{:016x}",
                ret_addr - self.process.base_addr
            );

            func_offset = ret_addr - self.process.base_addr;
            let name: String = self
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
                rbp = ptrace::read(self.process.pid, saved_rbp_addr as ptrace::AddressType).unwrap()
                    as u64;
            }
            first = false;
        }
        Ok(())
    }

    pub fn set_register(&self, reg: &str, value_str: &str) -> Result<()> {
        let value = self.parse_address(value_str)?;

        let mut regs = ptrace::getregs(self.process.pid)?;
        match reg {
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
            _ => bail!("Unknown register: {}", reg),
        }
        ptrace::setregs(self.process.pid, regs)?;
        Ok(())
    }

    pub fn get_register_value(&self, name: &str) -> Result<u64> {
        let regs = getregs(self.process.pid)?;
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

    pub fn get_address_value(&self, addr_str: &str) -> Result<i64> {
        let addr = self.parse_address(addr_str)?;
        Ok(ptrace::read(self.process.pid, addr as ptrace::AddressType)?)
    }

    pub fn get_function_name(&self, target_addr: u64) -> Option<String> {
        self.functions
            .iter()
            .find(|f| f.offset <= target_addr && f.offset + f.size > target_addr)
            .map(|f| f.name.clone())
    }

    pub fn set_breakpoint_by_input(&mut self, input: &str) -> Result<u64> {
        let addr = if let Ok(addr) = self.parse_address(input) {
            addr
        } else if let Some(function) = self.functions.iter().find(|f| f.name == input) {
            debug!(
                "Found function, setting bp on {}, addr: {:#x}",
                input, function.offset
            );
            function.offset + self.process.base_addr
        } else {
            bail!("Invalid breakpoint input: {}", input);
        };
        self.breakpoint.set_breakpoint(addr, self.process.pid);
        Ok(addr)
    }

    pub fn rm_breakpoint_by_input(&mut self, input: &str) -> Result<()> {
        let addr = if let Ok(addr) = self.parse_address(input) {
            addr
        } else {
            bail!("Invalid rm breakpoint input: {}", input);
        };

        self.breakpoint.remove_breakpoint(addr, self.process.pid)
    }

    pub fn dump_hex(&mut self, addr_str: &str, size: usize) -> Result<()> {
        let addr = self.parse_address(addr_str)?;
        let mut buf = vec![0u8; size];
        read_process_memory(self.process.pid, addr as usize, &mut buf)?;

        for (i, chunk) in buf.chunks(16).enumerate() {
            print!("0x{:08X}: ", addr as usize + i * 16);

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
        Ok(())
    }

    pub fn patch(&self, addr_str: &str, value_str: &str) -> Result<()> {
        let addr = self.parse_address(addr_str)?;
        let value = self.parse_address(value_str)?;

        ptrace::write(self.process.pid, addr as ptrace::AddressType, value as i64)?;
        Ok(())
    }

    pub fn single_step(&mut self) -> Result<()> {
        nix::sys::ptrace::step(self.process.pid, None)?;
        Ok(())
    }

    pub fn cont(&mut self) -> Result<()> {
        nix::sys::ptrace::cont(self.process.pid, None)?;
        self.state = DebuggerState::AwaitingTrap;
        Ok(())
    }

    pub fn step_over(&mut self) -> Result<()> {
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

        match read_process_memory(self.process.pid, rip as usize, &mut code) {
            Ok(_) => {}
            Err(e) => println!("read process memory failed with error {}", e),
        }
        let insns = cs.disasm_all(&code, rip).expect("Disassembly failed");
        let next_inst = insns.iter().next().unwrap();
        if next_inst.mnemonic() == Some("call") {
            let next_addr = rip + next_inst.len() as u64;
            self.breakpoint.set_breakpoint(next_addr, self.process.pid);
            self.cont();
        } else {
            ptrace::step(self.process.pid, None)?;
        }
        Ok(())
    }

    pub fn disassemble(&self) -> Result<()> {
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
        read_process_memory(self.process.pid, rip as usize, &mut code)?;
        debug!("{:?}", code);

        let insns = cs.disasm_all(&code, rip)?;

        for i in insns.iter() {
            println!(
                "0x{:x}: {}\t{}",
                i.address(),
                i.mnemonic().unwrap_or(""),
                i.op_str().unwrap_or("")
            );
            self.dwarf
                .get_line_and_file(i.address() - self.process.base_addr);
        }
        Ok(())
    }

    pub fn print_sections(&self) -> Result<()> {
        let data = fs::read(self.path.clone()).unwrap();
        let obj_file = object::File::parse(&*data)?;
        for section in obj_file.sections() {
            println!(
                "Section: {:<20} Addr: 0x{:08x}, Size: 0x{:x}",
                section.name().unwrap_or("<unnamed>"),
                section.address(),
                section.size(),
            );
        }
        Ok(())
    }

    fn parse_address(&self, input: &str) -> Result<u64> {
        let trimmed = input.trim();

        if let Some(stripped) = trimmed.strip_prefix("0x") {
            u64::from_str_radix(stripped, 16)
                .map_err(|e| anyhow::anyhow!("invalid hex address: {}", e))
        } else {
            u64::from_str_radix(trimmed, 10)
                .map_err(|e| anyhow::anyhow!("invalid dec address: {}", e))
        }
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
