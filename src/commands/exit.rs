use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct ExitCommand;

impl DebugCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.exit()?;
        Ok(())
    }
}
