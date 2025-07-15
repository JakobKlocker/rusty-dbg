use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
use crate::core::registers::Registers;

#[derive(Clone)]
pub struct SetRegisterCommand;

#[derive(Clone)]
pub struct GetRegisterCommand;

#[derive(Clone)]
pub struct GetAllRegistersCommand;

impl DebugCommand for SetRegisterCommand {
    fn name(&self) -> &'static str {
        "set-reg"
    }

    fn aliases(&self) -> &[&'static str] {
        &["set"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let reg = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: set <reg> <value>"))?;
        let value_str = args
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("Usage: set <reg> <value>"))?;
        debugger.set_register(reg, value_str)?;
        println!("set {} to {}", reg, value_str);
        Ok(())
    }
}

impl DebugCommand for GetRegisterCommand {
    fn name(&self) -> &'static str {
        "get-reg"
    }

    fn aliases(&self) -> &[&'static str] {
        &["get"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let reg = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: get <reg>"))?;
        let value = debugger.get_register_value(reg)?;
        println!("{}: 0x{:x}", reg, value);
        Ok(())
    }
}

impl DebugCommand for GetAllRegistersCommand {
    fn name(&self) -> &'static str {
        "registers"
    }

    fn aliases(&self) -> &[&'static str] {
        &["regs"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let regs = debugger.get_registers()?;
        println!("RIP:  0x{:016x}", regs.rip);
        println!("RSP:  0x{:016x}", regs.rsp);
        println!("RBP:  0x{:016x}", regs.rbp);
        println!("RAX:  0x{:016x}", regs.rax);
        println!("RBX:  0x{:016x}", regs.rbx);
        println!("RCX:  0x{:016x}", regs.rcx);
        println!("RDX:  0x{:016x}", regs.rdx);
        println!("RSI:  0x{:016x}", regs.rsi);
        println!("RDI:  0x{:016x}", regs.rdi);
        println!("R8:   0x{:016x}", regs.r8);
        println!("R9:   0x{:016x}", regs.r9);
        println!("R10:  0x{:016x}", regs.r10);
        println!("R11:  0x{:016x}", regs.r11);
        println!("R12:  0x{:016x}", regs.r12);
        println!("R13:  0x{:016x}", regs.r13);
        println!("R14:  0x{:016x}", regs.r14);
        println!("R15:  0x{:016x}", regs.r15);
        println!("EFLAGS: 0x{:08x}", regs.eflags);
        Ok(())
    }
}
