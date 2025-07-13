use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct BreakpointCommand;

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
        println!("execute once");
        debugger.set_breakpoint_by_input(arg)
    }
}
