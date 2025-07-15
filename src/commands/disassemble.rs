use crate::commands::DebugCommand;
use crate::core::disassembler::Disassembler;
use crate::core::Debugger;
use anyhow::Result;

#[derive(Clone)]
pub struct DisassembleCommand;

impl DebugCommand for DisassembleCommand {
    fn name(&self) -> &'static str {
        "dissasemble"
    }

    fn aliases(&self) -> &[&'static str] {
        &["disas"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.disassemble()
    }
}
