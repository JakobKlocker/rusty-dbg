use crate::commands::DebugCommand;
use crate::core::stepping::Stepping;
use crate::core::Debugger;
use anyhow::Result;

#[derive(Clone)]
pub struct SingleStepCommand;

#[derive(Clone)]
pub struct ContinueCommand;

#[derive(Clone)]
pub struct StepOverCommand;

impl DebugCommand for SingleStepCommand {
    fn name(&self) -> &'static str {
        "step"
    }

    fn aliases(&self) -> &[&'static str] {
        &["s"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.single_step()
    }
}

impl DebugCommand for ContinueCommand {
    fn name(&self) -> &'static str {
        "cont"
    }

    fn aliases(&self) -> &[&'static str] {
        &["c"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.cont()
    }
}

impl DebugCommand for StepOverCommand {
    fn name(&self) -> &'static str {
        "next"
    }

    fn aliases(&self) -> &[&'static str] {
        &["n"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.step_over()
    }
}
