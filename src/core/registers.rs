use libc::user_regs_struct;

pub trait Registers {
    fn get_registers(&self) -> Result<user_regs_struct>;
    fn set_register(&self, reg: &str, value_str: &str) -> Result<()>;
    fn get_register_value(&self, name: &str) -> Result<u64>;
}

impl Registers for Debugger {
  fn get_registers(&self) -> Result<user_regs_struct> {
        Ok(getregs(self.process.pid)?)
    }
    fn set_register(&self, reg: &str, value_str: &str) -> Result<()> {
        let value = self.parse_address(value_str)?;

        let mut regs = ptrace::getregs(self.process.pid)?;
        match reg {
            "rip" => regs.rip = value,
            "rax" => regs.rax = value,
            "rbx" => regs.rbx = value,
            "rcx" => regs.rcx = value,
            "rdx" => regs.rdx = value,
            "rsi" => regs.rsi = value,
            "rdi" => regs.rdi = value,
            "rsp" => regs.rsp = value,
            "rbp" => regs.rbp = value,
            "r8" => regs.r8 = value,
            "r9" => regs.r9 = value,
            "r10" => regs.r10 = value,
            "r11" => regs.r11 = value,
            "r12" => regs.r12 = value,
            "r13" => regs.r13 = value,
            "r14" => regs.r14 = value,
            "r15" => regs.r15 = value,
            "eflags" => regs.eflags = value,
            _ => bail!("Unknown register: {}", reg),
        }
        ptrace::setregs(self.process.pid, regs)?;
        Ok(())
    }

    fn get_register_value(&self, name: &str) -> Result<u64> {
        let regs = getregs(self.process.pid)?;
        let value = match name {
            "rip" => Some(regs.rip),
            "rax" => Some(regs.rax),
            "rbx" => Some(regs.rbx),
            "rcx" => Some(regs.rcx),
            "rdx" => Some(regs.rdx),
            "rsi" => Some(regs.rsi),
            "rdi" => Some(regs.rdi),
            "rsp" => Some(regs.rsp),
            "rbp" => Some(regs.rbp),
            "r8" => Some(regs.r8),
            "r9" => Some(regs.r9),
            "r10" => Some(regs.r10),
            "r11" => Some(regs.r11),
            "r12" => Some(regs.r12),
            "r13" => Some(regs.r13),
            "r14" => Some(regs.r14),
            "r15" => Some(regs.r15),
            "eflags" => Some(regs.eflags),
            _ => None,
        };
        value.ok_or_else(|| anyhow::anyhow!("Unkown Register: {}", name))
    }
}
