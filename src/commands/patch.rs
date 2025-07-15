use crate::commands::DebugCommand;
use crate::core::memory::Memory;
use crate::core::Debugger;
use anyhow::Result;

#[derive(Clone)]
pub struct PatchCommand;

impl DebugCommand for PatchCommand {
    fn name(&self) -> &'static str {
        "patch"
    }

    fn aliases(&self) -> &[&'static str] {
        &["set"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let addr_str = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: patch <address> <value>"))?;
        let value_str = args
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("Usage: patch <address> <value>"))?;
        debugger.patch(addr_str, value_str)?;
        println!("Patched address {} with value {}", addr_str, value_str);
        Ok(())
    }
}
