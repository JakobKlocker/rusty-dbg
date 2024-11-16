use nix::unistd::Pid;
use nix::sys::ptrace;
use nix::sys::ptrace::getregs;

pub struct Process {
    pub pid: Pid,
}

impl Process {
    pub fn attach(pid: i32) -> Self {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).expect("Failed to attach to process");
        println!("Successfully attached to PID: {}", pid);

        match getregs(pid) {
            Ok(regs) => println!("Registers: {:?}", regs),
            Err(err) => println!("Failed to get registers: {}", err),
        }
        Process { pid }
    }
}
