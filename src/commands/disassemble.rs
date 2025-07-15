use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
use crate::core::disassembler::Disassembler;

#[derive(Clone)]
pub struct DisassembleCommand;

impl DebugCommand for DisassembleCommand {
    fn name(&self) -> &'static str {
        "dissasemble"
    }

    fn aliases(&self) -> &[&'static str] {
        &["dis"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
       debugger.disassemble()
    }
}