use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct DisassembleCommand;

impl DebugCommand for DisassembleCommand {
    fn name(&self) -> &'static str {
        "dissasemble"
    }

    fn aliases(&self) -> &[&'static str] {
        &["dis"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
       debugger.disassemble()
    }
}