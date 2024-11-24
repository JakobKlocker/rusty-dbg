mod process; 
mod debugger;
mod breakpoint;

use std::env;

// try attach first
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2{
        println!("Usage: {} <pid|path>", args[0]);
        return;
    }

    let input = &args[1];

    let mut dbg = debugger::Debugger::new(input.to_string());
    dbg.run();
}
