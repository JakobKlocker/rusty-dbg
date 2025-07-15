use crate::breakpoint::*;
use crate::dwarf::*;
use crate::functions::*;
use crate::process::*;
use anyhow::{bail, Result};
use capstone::prelude::*;
use libc::user_regs_struct;
use log::{debug, info};
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use nix::sys::ptrace::setregs;
use nix::sys::wait::{waitpid, WaitStatus};
use object::{Object, ObjectSection};
use std::fs;
use std::path::Path;
use std::process::Command;

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

    pub fn get_registers(&self) -> Result<user_regs_struct> {
        Ok(getregs(self.process.pid)?)
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
        self.breakpoint.set_breakpoint(addr, self.process.pid)?;
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


    pub fn cont(&mut self) -> Result<()> {
        nix::sys::ptrace::cont(self.process.pid, None)?;
        self.state = DebuggerState::AwaitingTrap;
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

    #[allow(dead_code)]
    fn print_file_and_line(&self) {
        let regs = getregs(self.process.pid).unwrap();

        let rip = regs.rip;

        self.dwarf.get_line_and_file(rip - self.process.base_addr);
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
