use nix::unistd::Pid;
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use std::fs;
use std::io::{BufRead, BufReader};
use std::error::Error;

pub struct Process {
    pub pid: Pid,
}

impl Process {
    pub fn attach(pid: i32) -> Self {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).expect("Failed to attach to process");
        println!("Successfully attached to PID: {}", pid);

        match getregs(pid) {
            Ok(regs) => println!("Registers: {:?}", regs),
            Err(err) => println!("Failed to get registers: {}", err),
        }
        parse_maps_file(pid);
        Process { pid }
    }


}

pub fn parse_maps_file(pid: Pid) -> Result<(), Box<dyn Error>>{
    let file_path = format!("/proc/{}/maps", pid);
    let buff_reader = BufReader::new(fs::File::open(file_path)?);
    for line in buff_reader.lines(){
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}