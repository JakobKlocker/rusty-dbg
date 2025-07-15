use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct SetRegisterCommand;

impl DebugCommand for SetRegisterCommand {
    fn name(&self) -> &'static str {
        "set"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let reg = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: set <reg> <value>"))?;
        let value_str = args
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("Usage: set <reg> <value>"))?;
        debugger.set_register(reg, value_str)?;
        println!("set {} to {}", reg, value_str);
        Ok(())
    }
}
