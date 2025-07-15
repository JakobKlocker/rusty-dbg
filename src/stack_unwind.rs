use gimli::{BaseAddresses, EhFrame, RunTimeEndian, UnwindSection};
use gimli::UnwindContext;
use memmap2::Mmap;
use object::{Object, ObjectSection};
use std::fs;
use anyhow::Result;
use log::debug;

#[derive(Debug)]
pub struct UnwindRowInfo {
    pub cfa_register: u16,
    pub cfa_offset: i64,
    pub ra_offset: i64,
}

pub fn get_unwind_info(
    path: &str,
    target_addr: u64,
) -> Result<UnwindRowInfo> {
    let file = fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;

    let endian = if object.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };
    let eh_frame_section = object
        .section_by_name(".eh_frame")
        .ok_or_else(|| anyhow::anyhow!("No .eh_frame section found"))?;

    let eh_frame_data = eh_frame_section.uncompressed_data()?;
    let eh_frame_address = eh_frame_section.address();
    let bases = BaseAddresses::default().set_eh_frame(eh_frame_address);
    let eh_frame = EhFrame::new(&eh_frame_data, endian);
    let mut entries = eh_frame.entries(&bases);

    while let Some(entry) = entries.next()? {
        match entry {
            gimli::CieOrFde::Cie(_cie) => {
            }
            gimli::CieOrFde::Fde(partial) => {
                let fde = partial
                    .parse(|_section, bases, offset| eh_frame.cie_from_offset(bases, offset))
                    .unwrap();

                let start = fde.initial_address();
                let end = start + fde.len();


                if target_addr >= start && target_addr < end {
                    let mut ctx = UnwindContext::new();

                    let row =
                        fde.unwind_info_for_address(&eh_frame, &bases, &mut ctx, target_addr)?;

                    for (reg, rule) in row.registers() {
                        debug!("Register {:?} = {:?}", reg, rule);
                    }

                    let (cfa_register, cfa_offset) = match row.cfa() {
                        gimli::CfaRule::RegisterAndOffset { register, offset } => {
                            (register.0, *offset)
                        }
                        rule => return Err(anyhow::anyhow!("Unsupported CFA rule: {:?}", rule).into()),
                    };

                    let ra_offset = match row.register(gimli::X86_64::RA) {
                        gimli::RegisterRule::Offset(off) => off,
                        rule => return Err(anyhow::anyhow!("Unsupported RA rule: {:?}", rule).into()),
                    };

                    let info = UnwindRowInfo {
                        cfa_register,
                        cfa_offset,
                        ra_offset,
                    };

                    return Ok(info);
                }
            }
        }
    }
    return Err(anyhow::anyhow!("No FDE found for address 0x{:x}", target_addr).into()); //need better way to detect end of backtrace
}
