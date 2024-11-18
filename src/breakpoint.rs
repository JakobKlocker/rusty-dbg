use nix::{sys::ptrace, unistd::Pid};
use nix::libc;

pub struct Breakpoint{
    breakpoint: Vec<(u64, u8)>,
}

impl Breakpoint{
    pub fn new() -> Self{
        Breakpoint{
            breakpoint: Vec::new(),
        }
    }

    pub fn set_breakpoint(&mut self, addr: u64, pid: Pid){

        let original_byte = ptrace::read(pid, addr as *mut libc::c_void).unwrap() as u8;

        ptrace::write(pid, addr as *mut libc::c_void, 0xCC);
    
        match ptrace::read(pid, addr as *mut libc::c_void) {
            Ok(breakpoint_check) => {
                if (breakpoint_check & 0xFF) != 0xCC {
                    println!("Breakpoint was not written correctly: 0x{:x}", breakpoint_check);
                } else {
                    println!("Breakpoint is correctly set.");
                    self.breakpoint.push((addr, original_byte));
                    self.show_breakpoints();
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
            println!("Breakpoint removed and original byte restored at: {:#x}", saved_addr);
            true;
        }
        return false;
    }

    pub fn show_breakpoints(&self){
        for bp in self.breakpoint.iter(){
            println!("addr: {}  byte: {}", bp.0, bp.1);
        }
    }
}