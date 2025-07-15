use crate::core::memory::read_process_memory;
use crate::core::*;
use anyhow::Result;
use capstone::prelude::*;
use nix::sys::ptrace;
use nix::sys::ptrace::*;

pub trait Stepping {
    fn cont(&mut self) -> Result<()>;
    fn single_step(&mut self) -> Result<()>;
    fn step_over(&mut self) -> Result<()>;
}

impl Stepping for Debugger {
    fn cont(&mut self) -> Result<()> {
        nix::sys::ptrace::cont(self.process.pid, None)?;
        self.state = DebuggerState::AwaitingTrap;
        Ok(())
    }

    fn single_step(&mut self) -> Result<()> {
        nix::sys::ptrace::step(self.process.pid, None)?;
        Ok(())
    }

    fn step_over(&mut self) -> Result<()> {
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

        read_process_memory(self.process.pid, rip as usize, &mut code)?;
        let insns = cs.disasm_all(&code, rip).expect("Disassembly failed");
        let next_inst = insns.iter().next().unwrap();
        if next_inst.mnemonic() == Some("call") {
            let next_addr = rip + next_inst.len() as u64;
            self.breakpoint
                .set_breakpoint(next_addr, self.process.pid)?;
            self.cont()?;
        } else {
            ptrace::step(self.process.pid, None)?;
        }
        Ok(())
    }
}
