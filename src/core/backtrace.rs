use crate::core::Debugger;
use anyhow::Result;
use nix::sys::ptrace;
use crate::core::memory;
use log::debug;
pub trait Backtrace {
    fn backtrace(&self) -> Result<()>;
}

impl Backtrace for Debugger {
    fn backtrace(&self) -> Result<()> {
        let regs = getregs(self.process.pid)?;
        let mut rip = regs.rip;
        let mut rsp = regs.rsp;
        let mut rbp = regs.rbp;
        let mut first = true;

        loop {
            let mut func_offset = rip - self.process.base_addr;

            let info = get_unwind_info(&self.path, func_offset)?;
            debug!("{:?}", info);

            let cfa_base = match info.cfa_register {
                6 => rbp,
                7 => rsp,
                16 => rip,
                other => panic!("unsupported cfa reg{}", other),
            };

            let cfa = (cfa_base as i64 + info.cfa_offset) as u64;
            debug!("CFA: 0x{:016x}", cfa);

            let ret_addr_addr = (cfa as i64 + info.ra_offset) as u64;

            let ret_addr =
                ptrace::read(self.process.pid, ret_addr_addr as ptrace::AddressType)? as u64;

            debug!(
                "Return address (caller RIP): 0x{:016x}",
                ret_addr - self.process.base_addr
            );

            func_offset = ret_addr - self.process.base_addr;
            let name: String = self
                .get_function_name(func_offset)
                .unwrap_or_else(|| "_start".to_string());
            if first != true {
                print!("-> ");
            }
            print!("{}", name);
            rip = ret_addr;
            rsp = cfa;
            if info.cfa_register == 6 {
                let saved_rbp_addr = (cfa as i64 - 16) as u64;
                rbp = ptrace::read(self.process.pid, saved_rbp_addr as ptrace::AddressType)? as u64;
            }
            first = false;
        }
    }
}
