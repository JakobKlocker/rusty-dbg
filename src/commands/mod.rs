pub mod backtrace;
pub mod breakpoint;
pub mod command_trait;
pub mod control;
pub mod disassemble;
pub mod dump_hex;
pub mod exit;
pub mod offset;
pub mod patch;
pub mod registers;
pub mod sections;

use crate::commands::backtrace::BacktraceCommand;
use crate::commands::breakpoint::ShowBreakpointsCommand;
use crate::commands::control::ContinueCommand;
use crate::commands::control::StepOverCommand;
use crate::commands::disassemble::DisassembleCommand;
use crate::commands::exit::ExitCommand;
use crate::commands::offset::OffsetCommand;
use crate::commands::registers::GetAllRegistersCommand;
use crate::commands::registers::GetRegisterCommand;
use crate::commands::registers::SetRegisterCommand;
use crate::commands::sections::SectionsCommand;
use crate::core::Debugger;
pub use breakpoint::BreakpointCommand;
pub use breakpoint::RemoveBreakpointCommand;
pub use command_trait::DebugCommand;
pub use control::SingleStepCommand;
pub use dump_hex::DumpHexCommand;

use std::collections::HashMap;

pub struct CommandRouter {
    commands: HashMap<String, Box<dyn DebugCommand>>,
}

impl CommandRouter {
    pub fn new() -> Self {
        let mut router = CommandRouter {
            commands: HashMap::new(),
        };

        let all_commands: Vec<Box<dyn DebugCommand>> = vec![
            Box::new(BreakpointCommand),
            Box::new(RemoveBreakpointCommand),
            Box::new(DumpHexCommand),
            Box::new(SingleStepCommand),
            Box::new(ContinueCommand),
            Box::new(StepOverCommand),
            Box::new(DisassembleCommand),
            Box::new(SectionsCommand),
            Box::new(OffsetCommand),
            Box::new(ShowBreakpointsCommand),
            Box::new(ExitCommand),
            Box::new(BacktraceCommand),
            Box::new(SetRegisterCommand),
            Box::new(GetRegisterCommand),
            Box::new(GetAllRegistersCommand),
        ];

        for cmd in all_commands {
            router.commands.insert(cmd.name().into(), cmd.clone());
            for &alias in cmd.aliases() {
                router.commands.insert(alias.into(), cmd.clone());
            }
        }
        router
    }

    pub fn handle(&self, input: &str, dbg: &mut Debugger) {
        let mut parts = input.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => return,
        };

        let args: Vec<&str> = parts.collect();

        if let Some(command) = self.commands.get(cmd) {
            if let Err(e) = command.execute(&args, dbg) {
                println!("Error: {}", e);
            }
        } else {
            println!("Unknown command: {}", cmd);
        }
    }
}
