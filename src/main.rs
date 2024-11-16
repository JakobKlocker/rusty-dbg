mod process; 
mod debugger;

use std::env;
use std::path::Path;
// use crate::debugger;


// try attach first
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2{
        println!("Usage: {} <pid|path>", args[0]);
        return;
    }

    let input = &args[1];

    if Path::new(&format!("/proc/{}", input)).is_dir() {
        println!("{} is a pid", input);
    }
    else if Path::new(input).is_file() {
        println!("{} is a path", input);
    }
    else if Path::new(input).is_dir() {
        print!("Invalid: {} is a dir", input);
        return;
    }
    else{
        println!("{} is neither path nor dir", input);
        return;
    }

    let pid: i32 = input.parse().expect("Failed to parse PID"); 
    let dbg = debugger::Debugger::new(pid);
    dbg.run();
}
