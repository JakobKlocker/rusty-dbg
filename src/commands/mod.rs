pub mod breakpoint;
pub mod command_trait;
pub use breakpoint::BreakpointCommand;
pub use command_trait::DebugCommand;
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
        println!("Hanlde once");
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
