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

    let input: &_ = &args[1];

    let mut dbg = debugger::Debugger::new(input.to_string());
    dbg.process.print_map_infos();
    dbg.run();
}
