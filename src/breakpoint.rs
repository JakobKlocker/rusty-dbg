use nix::{sys::ptrace, unistd::Pid};
use nix::libc;

pub struct Breakpoint{
    breakpoint: Vec<u64>,
}

impl Breakpoint{
    pub fn new() -> Self{
        Breakpoint{
            breakpoint: Vec::new(),
        }
    }

    pub fn set_breakpoint(&mut self, addr: u64, pid: Pid){
        self.breakpoint.push(addr);
        ptrace::write(pid, addr as *mut libc::c_void, 0xCC);
        println!("added bp {}", addr);
    
        match ptrace::read(pid, addr as *mut libc::c_void) {
            Ok(breakpoint_check) => {
                if (breakpoint_check & 0xFF) != 0xCC {
                    println!("Breakpoint was not written correctly: 0x{:x}", breakpoint_check);
                } else {
                    println!("Breakpoint is correctly set.");
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    pub fn remove_breakpoint(mut self, addr: u64) -> bool {
        if let Some(index) = self.breakpoint.iter().position(|&x| x == addr){
            self.breakpoint.remove(index);
            println!("found bp, removed {}", addr);
            return true;
        }
        false
    }
}