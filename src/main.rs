mod breakpoint;
mod debugger;
mod functions;
mod map;
mod process;

use crate::functions::*;
use gimli::Reader as _;
use object::{Object, ObjectSection};
use std::{borrow, env, error, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <pid|path>", args[0]);
        return;
    }

    let debuger_process_name: &_ = &args[0].rsplit('/').next().unwrap_or("unknown");
    let debugee_pid_path: &_ = &args[1];

    let mut dbg = debugger::Debugger::new(
        debugee_pid_path.to_string(),
        debuger_process_name.to_string(),
    );
    // dbg.process.print_map_infos();
    dbg.print_functions();
    dbg.run();
}
