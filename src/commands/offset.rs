use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct OffsetCommand;

impl DebugCommand for OffsetCommand {
    fn name(&self) -> &'static str {
        "offset"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.print_offset();
        Ok(())
    }
}
