pub trait DebugCommand{
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &[&'static str] {&[]}
    fn execite(&self, args: &[&str], debugger: &mut Debugger) -> Result<()>;
}