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
        println!("add: {}  pid: {}", addr, pid);

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

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::path::Path;

    use crate::debugger::Debugger;

    
    #[test]
    fn test_breakpoint_on_ls(){
        let ls_path = "/bin/ls";
        
        assert!(
            std::path::Path::new(ls_path).exists(),
            "ls doesn't exist {}",
            ls_path
        );
        
        let mut debugger = Debugger::new(ls_path.to_string());
        nix::sys::wait::waitpid(debugger.process.pid, None).unwrap();
        
        // TODO: Parse processes read/write to get a valid address to breakpoint at 
        // debugger.breakpoint.set_breakpoint(0x1234, debugger.process.pid);
    }
}

// #[test]
// fn compile_test_program(){
//     let test_program_path = Path::new("./test-programm");

//     assert!(
//         test_program_path.exists(),
//         "Test program path does not exist: {}",
//         test_program_path.display()
//     );

//     let output = Command::new("cargo")
//         .arg("build")
//         .current_dir(test_program_path)
//         .output()
//         .expect("Failed to execute `cargo build`");

//     if output.status.success() {
//         println!("Test program compiled successfully!");
//     } else {
//         panic!(
//             "Failed to compile test program: {}",
//             String::from_utf8_lossy(&output.stderr)
//         );
//     }
// }