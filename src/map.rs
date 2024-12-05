use nix::unistd::Pid;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::fs;
use std::fmt;

#[derive(Debug)]
pub struct Map{
    pub addr_start: u64,
    pub addr_end: u64,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub shared: bool,
    pub private: bool,
}

impl Map{
    pub fn new(pid: Pid) -> Result<Vec<Self>, Box<dyn Error>> {
        let maps_info = Self::get_maps_info(pid)?;
        let mut map_objects = Vec::new();
    
        for map in maps_info {
            let map_object = Self::parse_maps_info(map)?;
            map_objects.push(map_object);
        }
        Ok(map_objects)
    }

        //maps line example: 622b53609000-622b5360d000 r--p 00000000 103:05 4327957                   /usr/bin/ls
        pub fn parse_maps_info(map: String) -> Result<Self, Box<dyn Error>> {
            let parts: Vec<&str> = map.split_whitespace().collect();
            println!("{:?}", parts);
            if parts.len() < 5 { // should check for 6, sometimes it's below 5 thought. RECHECK THIS
                return Err("Failed parsing maps, len below 5".into());
            }
    
            let addr_range: Vec<&str> = parts[0].split('-').collect();
            let addr_start = u64::from_str_radix(addr_range[0], 16)?;
            let addr_end = u64::from_str_radix(addr_range[1], 16)?;
    
            let permissions = parts[1];
            let mut read = false;
            let mut write = false;
            let mut execute = false;
            let mut shared = false;
            let mut private = false;
    
            for ch in permissions.chars() {
                match ch {
                    'r' => read = true,
                    'w' => write = true,
                    'x' => execute = true,
                    's' => shared = true,
                    'p' => private = true,
                    _ => (),
                }
            }
    
            Ok(Map {
                addr_start,
                addr_end,
                read,
                write,
                execute,
                shared,
                private,
            })
        }

        pub fn get_maps_info(pid: Pid) -> Result<Vec<String>, Box<dyn Error>>{
            let file_path = format!("/proc/{}/maps", pid);
            let buff_reader = BufReader::new(fs::File::open(file_path)?);
            let mut maps = Vec::new();
            for line in buff_reader.lines(){
                let line = line?;
                maps.push(line);
            }
            Ok(maps)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Map {{ addr_start: 0x{:x}, addr_end: 0x{:x}, read: {}, write: {}, execute: {}, shared: {}, private: {} }}",
            self.addr_start, 
            self.addr_end,
            self.read,
            self.write,
            self.execute,
            self.shared,
            self.private
        )
    }
}
