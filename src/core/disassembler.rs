use crate::core::Debugger;
use crate::core::memory;
use anyhow::Result;
use capstone::prelude::*;
use log::debug;
use nix::sys::ptrace::getregs;
use crate::core::memory::read_process_memory;

pub trait Disassembler {
    fn disassemble(&self) -> Result<()>;
}

impl Disassembler for Debugger {
    fn disassemble(&self) -> Result<()> {
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
}
