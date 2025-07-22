use crate::core::Debugger;
use anyhow::Result;
use libc::{iovec, pid_t, process_vm_readv};
use nix::sys::ptrace;
use nix::unistd::Pid;
use std::io::Error;

pub trait Memory {
    fn patch(&self, addr_str: &str, value_str: &str) -> Result<()>;
    fn get_address_value(&self, addr_str: &str) -> Result<i64>;
    fn dump_hex(&mut self, addr_str: &str, size: usize) -> Result<Vec<u8>>;
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

    fn dump_hex(&mut self, addr_str: &str, size: usize) -> Result<Vec<u8>> {
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
        Ok(buf)
    }
}

pub fn read_process_memory(pid: Pid, addr: usize, buf: &mut [u8]) -> Result<usize> {
    let local = iovec {
        iov_base: buf.as_mut_ptr() as *mut _,
        iov_len: buf.len(),
    };

    let remote = iovec {
        iov_base: addr as *mut _,
        iov_len: buf.len(),
    };

    let result = unsafe { process_vm_readv(pid.as_raw() as pid_t, &local, 1, &remote, 1, 0) };

    if result == -1 {
        Err(Error::last_os_error().into())
    } else {
        Ok(result as usize)
    }
}
