use crate::commands::DebugCommand;
use crate::core::Debugger;
use anyhow::Result;
#[derive(Clone)]
pub struct SetRegisterCommand;

#[derive(Clone)]
pub struct GetRegisterCommand;

impl DebugCommand for SetRegisterCommand {
    fn name(&self) -> &'static str {
        "set-reg"
    }

    fn aliases(&self) -> &[&'static str] {
        &["set"]
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

impl DebugCommand for GetRegisterCommand {
    fn name(&self) -> &'static str {
        "get-reg"
    }

    fn aliases(&self) -> &[&'static str] {
        &["get"]
    }

    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()> {
        let reg = args
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Usage: get <reg>"))?;
        let value = debugger.get_register_value(reg)?;
        println!("{}: 0x{:x}", reg, value);
        Ok(())
    }
}