use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct BreakpointCommand;
#[derive(Clone)]
pub struct RemoveBreakpointCommand;

impl DebugCommand for BreakpointCommand {
    fn name(&self) -> &'static str {
        "bp"
    }

    fn aliases(&self) -> &[&'static str] {
        &["b"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let arg = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: bp <address>"))?;
        let bp_addr = debugger.set_breakpoint_by_input(arg)?;
        println!("breakpoint set at 0x{:x}", bp_addr);
        Ok(())
    }
}

impl DebugCommand for RemoveBreakpointCommand {
    fn name(&self) -> &'static str {
        "rm-bp"
    }

    fn aliases(&self) -> &[&'static str] {
        &["rmb"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let addr_str = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: rmb <address>"))?;
        debugger.rm_breakpoint_by_input(addr_str)?;
        println!("breakpoint removed at {}", addr_str);
        Ok(())
    }
}

#[derive(Clone)]
pub struct ShowBreakpointsCommand;

impl DebugCommand for ShowBreakpointsCommand {
    fn name(&self) -> &'static str {
        "show-bp"
    }

    fn aliases(&self) -> &[&'static str] {
        &["show"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        for entry in debugger.list_breakpoints() {
            println!("0x{:x} (original byte: {:02x})", entry.0, entry.1);
        }
        Ok(())
    }
}
