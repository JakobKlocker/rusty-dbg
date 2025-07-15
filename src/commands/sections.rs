use crate::commands::DebugCommand;
use crate::core::Debugger;
 use crate::core::symbols::Symbols;
use anyhow::Result;
#[derive(Clone)]
pub struct SectionsCommand;

impl DebugCommand for SectionsCommand {
    fn name(&self) -> &'static str {
        "sections"
    }

    fn aliases(&self) -> &[&'static str] {
        &["sec"]
    }

    fn execute(&self, _args: &[&str], debugger: &mut Debugger) -> Result<()> {
        debugger.print_sections()
    }
}
