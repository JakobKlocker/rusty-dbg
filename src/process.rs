use nix::unistd::Pid;
use nix::sys::ptrace;

pub struct Process {
    pid: Pid,
}

impl Process {
    pub fn attach(pid: i32) -> Self {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).expect("Failed to attach to process");
        Process { pid }
    }
}
