mod breakpoint;
mod debugger;
mod gimliTesting;
mod map;
mod process;

use crate::gimliTesting::*;
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

    gimli_test(input.to_string());

    let mut dbg = debugger::Debugger::new(input.to_string());
    dbg.process.print_map_infos();
    dbg.run();
}
