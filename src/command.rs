use crate::core::{Debugger, DebuggerState};
use anyhow::{bail, Result};
use capstone::prelude::*;
use log::debug;
use nix::sys::ptrace::{self, getregs};
use object::Object;
use object::ObjectSection;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::fs;

use crate::commands::CommandRouter;
use crate::memory::read_process_memory;
use crate::stack_unwind::*;
pub struct CommandHandler<'a> {
    pub debugger: &'a mut Debugger,
}

impl<'a> CommandHandler<'a> {
    pub fn get_command(&self) -> String {
        let mut rl = DefaultEditor::new().unwrap();
        let _ = rl.load_history(".history");

        match rl.readline("Enter Command: ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line).unwrap();
                let _ = rl.save_history(".history");
                line
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                std::process::exit(0);
            }

            Err(err) => {
                eprintln!("Unexpected error: {:?}", err);
                String::new()
            }
        }
    }

    pub fn handle_command(&mut self, command: &str) {
        let router = CommandRouter::new();
        router.handle(&command, self.debugger);
        return;
        let mut parts = command.split_whitespace();
        let command_word = parts.next();

        match command_word {
            _ => println!("command not found {}", command),
        }
    }


    fn write(&self, addr: u64, value: i64) -> Result<()> {
        ptrace::write(
            self.debugger.process.pid,
            addr as ptrace::AddressType,
            value,
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    fn print_file_and_line(&self) {
        let regs = getregs(self.debugger.process.pid).unwrap();

        let rip = regs.rip;

        self.debugger
            .dwarf
            .get_line_and_file(rip - self.debugger.process.base_addr);
    }
}
