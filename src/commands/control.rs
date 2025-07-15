use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct SingleStepCommand;

#[derive(Clone)]
pub struct ContinueCommand;

impl DebugCommand for SingleStepCommand {
    fn name(&self) -> &'static str {
        "step"
    }

    fn aliases(&self) -> &[&'static str] {
        &["s"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.single_step()
    }
}

impl DebugCommand for ContinueCommand {
    fn name(&self) -> &'static str {
        "continue"
    }

    fn aliases(&self) -> &[&'static str] {
        &["c"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.cont()
    }
}
