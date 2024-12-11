mod breakpoint;
mod debugger;
mod functions;
mod map;
mod process;

use crate::functions::*;
use addr2line::Context;
use gimli::Reader as _;
use object::{Object, ObjectSection};
use rustc_demangle::*;
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
    // test_addr2line();
    list_symbols("/home/schnee/projects/rusty-dbg/target/debug/test-programm");
    dbg.print_functions();
    dbg.process.get_base_addr_from_map();
    dbg.run();
}

// pub fn test_addr2line() -> Result<(), Box<dyn std::error::Error>> {
//     // Create the Loader for the binary
//     let loader =
//         addr2line::Loader::new("/home/schnee/projects/rusty-dbg/target/debug/test-programm")?;
//     println!("Base Address: {:#x}", loader.relative_address_base());

//     // Define a reasonable address range to search for symbols
//     let start_address = 0; // Start of address space
//     let end_address = u64::MAX; // Max possible address

//     let mut asd: &str = "";
//     // Iterate over a range of addresses to find symbols
//     for addr in (start_address..end_address).step_by(4) {
//         if let Some(symbol) = loader.find_symbol(addr) {
//             // Print symbol name and address
//             if (symbol != asd) {
//                 println!("| Start Address: {}", symbol,);
//                 asd = symbol;
//             }
//         }
//     }

//     Ok(())
// }

use object::ObjectSymbol;

fn list_symbols(binary_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let binary = fs::read(binary_path)?;
    let obj = object::File::parse(&*binary)?;

    for symbol in obj.symbols() {
        if symbol.is_definition() && symbol.kind() == object::SymbolKind::Text {
            let demangled_name = demangle(symbol.name().unwrap());
            println!("{}   {:x}", demangled_name, symbol.address());
            
        }
    }

    Ok(())
}
