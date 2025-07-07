use libc::{iovec, pid_t, process_vm_readv};
use nix::unistd::Pid;

pub fn read_process_memory(pid: Pid, addr: usize, buf: &mut [u8]) -> bool {
    let local = iovec {
        iov_base: buf.as_mut_ptr() as *mut _,
        iov_len: buf.len(),
    };

    let remote = iovec {
        iov_base: addr as *mut _,
        iov_len: buf.len(),
    };

    let result = unsafe { process_vm_readv(pid.as_raw() as pid_t, &local, 1, &remote, 1, 0) };

    result == buf.len() as isize
}
