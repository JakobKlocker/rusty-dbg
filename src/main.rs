mod breakpoint;
mod command;
mod core;
mod dwarf;
mod functions;
mod map;
mod memory;
mod process;
mod stack_unwind;

use std::env;
use crate::stack_unwind::load_eh_frame;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <pid|path>", args[0]);
        return;
    }
    
    load_eh_frame(&args[1]);

    let debuger_process_name: &_ = &args[0].rsplit('/').next().unwrap_or("unknown");
    let debugee_pid_path: &_ = &args[1];

    let mut dbg = core::Debugger::new(
        debugee_pid_path.to_string(),
        debuger_process_name.to_string(),
    );

    dbg.process.get_base_addr_from_map();
    dbg.run();
}
