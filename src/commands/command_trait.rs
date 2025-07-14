use crate::core::Debugger;
use anyhow::Result;

pub trait DebugCommand: DebugCommandClone {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &[&'static str] {
        &[]
    }
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> Result<()>;
}

pub trait DebugCommandClone {
    fn clone_box(&self) -> Box<dyn DebugCommand>;
}

impl<T> DebugCommandClone for T
where
    T: 'static + DebugCommand + Clone,
{
    fn clone_box(&self) -> Box<dyn DebugCommand> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DebugCommand> {
    fn clone(&self) -> Box<dyn DebugCommand> {
        self.clone_box()
    }
}
