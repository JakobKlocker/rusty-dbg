use crate::commands::CommandRouter;
use crate::core::process_control::ProcessControl;
use crate::core::{Debugger, DebuggerState};
use log::info;
use rustyline::{error::ReadlineError, DefaultEditor};

pub struct Repl<'a> {
    pub debugger: &'a mut Debugger,
}

impl<'a> Repl<'a> {
    pub fn run(&mut self) {
        let mut rl = DefaultEditor::new().unwrap();
        let _ = rl.load_history(".history");

        loop {
            let state = self.debugger.state.clone();
            match state {
                DebuggerState::AwaitingTrap => self.debugger.resume_and_wait(),
                DebuggerState::Interactive => match rl.readline("rusty-dbg> ") {
                    Ok(line) => {
                        let _ = rl.add_history_entry(&line);
                        let _ = rl.save_history(".history");
                        self.handle_command(&line);
                    }
                    Err(ReadlineError::Interrupted) => {
                        println!("^C");
                        std::process::exit(0);
                    }
                    Err(err) => {
                        eprintln!("Unexpected error: {:?}", err);
                        self.debugger.state = DebuggerState::Exit;
                    }
                },
                DebuggerState::Exit => break
            }
            info!("state: {:?}", self.debugger.state);
        }
    }

    pub fn handle_command(&mut self, command: &str) {
        let router = CommandRouter::new();
        router.handle(command, self.debugger);
    }
}
