use crate::core::*;
use anyhow::{Result, bail};
use log::debug;

pub trait BreakpointHelper {
    fn set_breakpoint_by_input(&mut self, input: &str) -> Result<u64>;
    fn rm_breakpoint_by_input(&mut self, input: &str) -> Result<()>;
}

impl BreakpointHelper for Debugger {
    fn set_breakpoint_by_input(&mut self, input: &str) -> Result<u64> {
        let addr = if let Ok(addr) = self.parse_address(input) {
            addr
        } else if let Some(function) = self.functions.iter().find(|f| f.name == input) {
            debug!(
                "Found function, setting bp on {}, addr: {:#x}",
                input, function.offset
            );
            function.offset + self.process.base_addr
        } else {
            bail!("Invalid breakpoint input: {}", input);
        };
        self.breakpoint.set_breakpoint(addr, self.process.pid)?;
        Ok(addr)
    }

    fn rm_breakpoint_by_input(&mut self, input: &str) -> Result<()> {
        let addr = if let Ok(addr) = self.parse_address(input) {
            addr
        } else {
            bail!("Invalid rm breakpoint input: {}", input);
        };

        self.breakpoint.remove_breakpoint(addr, self.process.pid)
    }
}
