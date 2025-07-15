use crate::core::Debugger;
use anyhow::Result;
use nix::sys::ptrace;

pub trait Memory {
    fn patch(&self, addr_str: &str, value_str: &str) -> Result<()>;
    fn get_address_value(&self, addr_str: &str) -> Result<i64>;
    fn dump_hex(&mut self, addr_str: &str) -> Result<()>;
}

impl Memory for Debugger {
    fn patch(&self, addr_str: &str, value_str: &str) -> Result<()> {
        let addr = self.parse_address(addr_str)?;
        let value = self.parse_address(value_str)?;

        ptrace::write(self.process.pid, addr as ptrace::AddressType, value as i64)?;
        Ok(())
    }

    fn get_address_value(&self, addr_str: &str) -> Result<i64> {
        let addr = self.parse_address(addr_str)?;
        Ok(ptrace::read(self.process.pid, addr as ptrace::AddressType)?)
    }

    fn dump_hex(&mut self, addr_str: &str, size: usize) -> Result<()> {
        let addr = self.parse_address(addr_str)?;
        let mut buf = vec![0u8; size];
        read_process_memory(self.process.pid, addr as usize, &mut buf)?;

        for (i, chunk) in buf.chunks(16).enumerate() {
            print!("0x{:08X}: ", addr as usize + i * 16);

            for byte in chunk {
                print!("{:02X} ", byte);
            }
            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }
            print!("|");

            for byte in chunk {
                let c = *byte as char;
                if c.is_ascii_graphic() || c == ' ' {
                    print!("{}", c);
                } else {
                    print!(".");
                }
            }
            println!("|");
        }
        Ok(())
    }
}
