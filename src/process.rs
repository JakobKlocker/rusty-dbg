use crate::map::Map;
use nix::sys::ptrace;
use nix::unistd::Pid;
use std::fs;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub maps: Vec<Map>,
    pub base_addr: u64,
}

impl Process {
    pub fn attach(pid: i32) -> Self {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).expect("Failed to attach to process");
        println!("Successfully attached to PID: {}", pid);
        let maps = Map::new(pid).expect("Failed to get maps");
        Process {
            pid,
            maps,
            base_addr: 0,
        }
    }

    pub fn print_map_infos(&self) {
        for map in &self.maps {
            println!("{}", map);
        }
    }

    pub fn get_random_rw_memory(&self) -> Result<u64, String> {
        for map in &self.maps {
            if map.read && map.write {
                return Ok(map.addr_start);
            }
        }
        Err("no r/w mem found".to_string())
    }

    pub fn get_program_name_from_pid(&self, pid: Pid) -> io::Result<String> {
        let file_path = format!("/proc/{}/comm", pid);
        let file = fs::File::open(file_path)?;
        let mut buff_reader = BufReader::new(file);

        let mut process_name = String::new();
        buff_reader.read_line(&mut process_name)?;

        Ok(process_name.trim_end().to_string())
    }

    pub fn get_base_addr_from_map(&mut self) -> Option<u64> {
        match self.get_program_name_from_pid(self.pid) {
            Ok(programm_name) => self
                .maps
                .iter()
                .find(|map| map.file_path.contains(&programm_name))
                .map(|map| {
                    println!("Base is probably: {:x}", map.addr_start);
                    self.base_addr = map.addr_start;
                    map.addr_start
                }),
            Err(e) => {
                eprintln!("Failed to get program name: {}", e);
                None
            }
        }
    }
}
