use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct BacktraceCommand;

impl DebugCommand for BacktraceCommand {
    fn name(&self) -> &'static str {
        "backtrace"
    }

    fn aliases(&self) -> &[&'static str] {
        &["bt"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.backtrace()
    }
}
