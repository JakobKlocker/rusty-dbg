use nix::unistd::Pid;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::fs;

pub struct Map{
    addr_start: u64,
    addr_end: u64,
    read: bool,
    write: bool,
    execute: bool,
    shared: bool,
    private: bool,
}

impl Map{
    pub fn new(pid: Pid) -> Result<Self, Box<dyn Error>> {
        let maps = Self::get_maps_info(pid)?;
        let map_info = Self::parse_maps_info(maps)?;

        Ok(map_info)
    }

        //maps line example: 622b53609000-622b5360d000 r--p 00000000 103:05 4327957                   /usr/bin/ls
        pub fn parse_maps_info(maps: Vec<String>) -> Result<Self, Box<dyn Error>> {
            for map in maps {
                println!("{}", map);
                let parts: Vec<&str> = map.split_whitespace().collect();
                if parts.len() < 6 {
                    return Err("Failed parsing maps, len below 6".into());
                }
    
                let addr_range: Vec<&str> = parts[0].split('-').collect();
                println!("{:?}", addr_range);
    
                let addr_start = u64::from_str_radix(addr_range[0], 16).unwrap();
                let addr_end = u64::from_str_radix(addr_range[1], 16).unwrap();
                
                let mut read = false;
                let mut write = false;
                let mut execute = false;
                let mut shared = false;
                let mut private = false;
    
                for ch in parts[1].chars() {
                    match ch {
                        'r' => read = true,
                        'w' => write = true,
                        'x' => execute = true,
                        's' => shared = true,
                        'p' => private = true,
                        _ => (),
                    }
                }
    
                return Ok(Map {
                    addr_start,
                    addr_end,
                    read,
                    write,
                    execute,
                    shared,
                    private,
                });
            }
    
            Err("No valid maps found".into())
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