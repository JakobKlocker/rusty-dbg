use crate::commands::DebugCommand;
use crate::core::memory::Memory;
use crate::core::Debugger;
use anyhow::Result;

#[derive(Clone)]
pub struct DumpHexCommand;

impl DebugCommand for DumpHexCommand {
    fn name(&self) -> &'static str {
        "dump"
    }

    fn aliases(&self) -> &[&'static str] {
        &["d"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let addr = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: dump <address> [size]"))?;
        let size = args
            .get(1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(128);
        debugger.dump_hex(addr, size);
        Ok(())
    }
}
