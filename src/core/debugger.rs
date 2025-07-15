use crate::core::breakpoint::*;
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
use object::{Object, ObjectSection};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::core::memory::read_process_memory;
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
