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
            .ok_or_else(|| anyhow::anyhow!("Missing address"))?;
        debugger.set_breakpoint_by_input(arg)
    }
}

impl DebugCommand for RemoveBreakpointCommand{
    fn name(&self) -> &'static str{
        "rm-bp"
    }
    
    fn aliases(&self) -> &[&'static str] {
        &["rmb"]
    }
    
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let arg = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing address"))?;
        println!("insinde rmb");
        debugger.rm_breakpoint_by_input(arg)
    }
}