use crate::map::Map;
use nix::unistd::Pid;
use nix::sys::ptrace;
use std::fs;
use std::io::{BufRead, BufReader};
use std::error::Error;

pub struct Process {
    pub pid: Pid,
    pub maps: Vec<Map>,
}

impl Process {
    pub fn attach(pid: i32) -> Self {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).expect("Failed to attach to process");
        println!("Successfully attached to PID: {}", pid);
        let maps = Self::maps::

        Process { pid, maps }
    }
}

