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
        let maps = Self::get_maps_info(pid);


        Process { pid }
    }

    fn parse_maps_info(maps: Vec<String>){
        // NEXT
    }

    fn get_maps_info(pid: Pid) -> Result<Vec<String>, Box<dyn Error>>{
        let file_path = format!("/proc/{}/maps", pid);
        let buff_reader = BufReader::new(fs::File::open(file_path)?);
        let mut maps = Vec::new();
        for line in buff_reader.lines(){
            let line = line?;
            println!("{}", line);
            maps.push(line);
        }
        Ok(maps)
    }

}

