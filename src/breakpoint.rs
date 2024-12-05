use nix::libc;
use nix::{sys::ptrace, unistd::Pid};

#[derive(Debug)]
pub struct Breakpoint {
    breakpoint: Vec<(u64, u8)>,
}

impl Breakpoint {
    pub fn new() -> Self {
        Breakpoint {
            breakpoint: Vec::new(),
        }
    }

    pub fn set_breakpoint(&mut self, addr: u64, pid: Pid) {
        println!("add: {}  pid: {}", addr, pid);

        let original_byte = ptrace::read(pid, addr as *mut libc::c_void).unwrap() as u8;

        println!("Original byte: {:x}", original_byte);

        ptrace::write(pid, addr as *mut libc::c_void, 0xCC).unwrap();

        match ptrace::read(pid, addr as *mut libc::c_void) {
            Ok(breakpoint_check) => {
                if (breakpoint_check & 0xFF) != 0xCC {
                    println!(
                        "Breakpoint was not written correctly: 0x{:x}",
                        breakpoint_check
                    );
                } else {
                    println!("Breakpoint is correctly set.");
                    self.breakpoint.push((addr, original_byte));
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    pub fn remove_breakpoint(&self, addr: u64, pid: Pid) -> bool {
        if let Some((saved_addr, saved_byte)) = self.breakpoint.iter().find(|(a, _)| *a == addr) {
            ptrace::write(pid, *saved_addr as *mut libc::c_void, *saved_byte as i64).unwrap();
            println!(
                "Breakpoint removed and original byte restored at: {:#x}",
                saved_addr
            );
            true;
        }
        return false;
    }

    pub fn show_breakpoints(&self) {
        for bp in self.breakpoint.iter() {
            println!("addr: {:x}  original byte: {:x}", bp.0, bp.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::debugger::Debugger;
    use nix::libc;
    use nix::sys::ptrace;

    #[test]
    fn test_breakpoint_on_ls() {
        let ls_path = "/bin/ls";

        assert!(
            std::path::Path::new(ls_path).exists(),
            "ls doesn't exist {}",
            ls_path
        );
        let mut debugger = Debugger::new(ls_path.to_string(), "".to_string());
        nix::sys::wait::waitpid(debugger.process.pid, None).unwrap();
        let addr: u64 = debugger.process.get_random_rw_memory().unwrap();
        println!("Found random address: {:x}", addr);
        let original_byte = ptrace::read(debugger.process.pid, addr as *mut libc::c_void).unwrap();
        println!("Original Byte: {:x}", original_byte);
        debugger
            .breakpoint
            .set_breakpoint(addr, debugger.process.pid);
        let patched_byte = ptrace::read(debugger.process.pid, addr as *mut libc::c_void).unwrap();
        println!("Patched Byte: {:x}", patched_byte);
        if original_byte == patched_byte {
            panic!(
                "Original and patched bytes are the same at address {:x}, something went wrong with the breakpoint",
                addr
            );
        }
    }
}
