use anyhow::Result;
use log::debug;
use nix::libc;
use nix::{sys::ptrace, unistd::Pid};

#[derive(Debug)]
pub struct Breakpoint {
    breakpoint: Vec<(u64, u8)>,
}

impl Breakpoint {
    pub fn new() -> Self {
        Breakpoint {
            breakpoint: Vec::new(),
        }
    }

    pub fn set_breakpoint(&mut self, addr: u64, pid: Pid) -> Result<()> {
        let aligned_addr = addr & !0x7;
        let byte_offset = addr % 8;

        let original_word = ptrace::read(pid, aligned_addr as *mut libc::c_void)? as u64;

        let original_byte = ((original_word >> (byte_offset * 8)) & 0xFF) as u8;

        debug!("[SET BP] Target addr:     {:#x}", addr);
        debug!("[SET BP] Aligned addr:    {:#x}", aligned_addr);
        debug!("[SET BP] Word read:       {:#018x}", original_word);
        debug!("[SET BP] Byte offset:     {}", byte_offset);
        debug!("[SET BP] Original byte:   {:#x}", original_byte);

        let patched_word =
            (original_word & !(0xFF << (byte_offset * 8))) | ((0xCCu64) << (byte_offset * 8));

        debug!("[SET BP] Patched word:    {:#018x}", patched_word);

        ptrace::write(pid, aligned_addr as *mut libc::c_void, patched_word as i64)?;

        self.breakpoint.push((addr, original_byte));
        debug!("[SET BP] Breakpoint set.\n");
        println!("breakpoint set on 0x{:x}", addr);
        Ok(())
    }

    pub fn remove_breakpoint(&mut self, addr: u64, pid: Pid) -> Result<()> {
        if let Some(pos) = self.breakpoint.iter().position(|(a, _)| *a == addr) {
            let (_, saved_byte) = self.breakpoint[pos];
            let aligned_addr = addr & !0x7;
            let byte_offset = addr % 8;

            let current_word = ptrace::read(pid, aligned_addr as *mut libc::c_void)? as u64;

            debug!("[REMOVE BP] Target addr:     {:#x}", addr);
            debug!("[REMOVE BP] Aligned addr:    {:#x}", aligned_addr);
            debug!("[REMOVE BP] Word read:       {:#018x}", current_word);
            debug!("[REMOVE BP] Byte offset:     {}", byte_offset);
            debug!("[REMOVE BP] Saved byte:      {:#x}", saved_byte);

            let restored_word = (current_word & !(0xFF << (byte_offset * 8)))
                | ((saved_byte as u64) << (byte_offset * 8));

            debug!("[REMOVE BP] Restored word:   {:#018x}", restored_word);

            ptrace::write(pid, aligned_addr as *mut libc::c_void, restored_word as i64)?;

            self.breakpoint.remove(pos);
            println!("breakpoint removed at 0x{:x}", addr);
        } else {
            println!("No breakpoint found at {:#x}", addr);
        }
        Ok(())
    }

    pub fn is_breakpoint(&self, addr: u64) -> bool {
        self.breakpoint.iter().any(|(a, _)| *a == addr)
    }

    pub fn show_breakpoints(&self) {
        for bp in self.breakpoint.iter() {
            println!("addr: {:#x}  original byte: {:x}", bp.0, bp.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Debugger;
    use nix::libc;
    use nix::sys::ptrace;

    #[test]
    fn test_breakpoint_on_ls() {
        let ls_path = "/bin/ls";

        assert!(
            std::path::Path::new(ls_path).exists(),
            "ls doesn't exist {}",
            ls_path
        );
        let mut debugger = Debugger::new(ls_path.to_string(), "".to_string());
        nix::sys::wait::waitpid(debugger.process.pid, None).unwrap();
        let addr: u64 = debugger.process.get_random_rw_memory().unwrap();
        println!("Found random address: {:x}", addr);
        let original_byte = ptrace::read(debugger.process.pid, addr as *mut libc::c_void).unwrap();
        println!("Original Byte: {:x}", original_byte);
        debugger
            .breakpoint
            .set_breakpoint(addr, debugger.process.pid);
        let patched_byte = ptrace::read(debugger.process.pid, addr as *mut libc::c_void).unwrap();
        println!("Patched Byte: {:x}", patched_byte);
        if original_byte == patched_byte {
            panic!(
                "Original and patched bytes are the same at address {:x}, something went wrong with the breakpoint",
                addr
            );
        }
    }
}
