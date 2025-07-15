pub mod breakpoint;
pub mod command_trait;
pub mod dump_hex;
pub mod patch;
pub mod control;

pub use breakpoint::BreakpointCommand;
pub use breakpoint::RemoveBreakpointCommand;
pub use command_trait::DebugCommand;
pub use dump_hex::DumpHexCommand;
pub use control::SingleStepCommand;
use crate::commands::control::ContinueCommand;
use crate::commands::control::StepOverCommand;
use crate::core::Debugger;


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
            Box::new(StepOverCommand)
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
