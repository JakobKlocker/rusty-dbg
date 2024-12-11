mod breakpoint;
mod debugger;
mod functions;
mod map;
mod process;

use crate::functions::*;
use addr2line::Context;
use gimli::Reader as _;
use gimli::{EndianSlice, LittleEndian};
use object::{Object, ObjectSection};
use rustc_demangle::*;
use std::borrow::Cow;
use std::path::Path;
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

    dbg.print_functions();
    dbg.process.get_base_addr_from_map();
    dbg.run();
}
