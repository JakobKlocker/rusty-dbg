use gimli::{BaseAddresses, EhFrame, EndianSlice, RunTimeEndian, UnwindSection};
use gimli::{CfaRule, UnwindContext};
use memmap2::Mmap;
use object::{Object, ObjectSection};
use std::{borrow, fs};

#[derive(Debug)]
pub struct UnwindRowInfo {
    pub cfa_register: u16,
    pub cfa_offset: i64,
    pub ra_offset: i64,
}

pub fn get_unwind_info(
    path: &str,
    target_addr: u64,
) -> Result<UnwindRowInfo, Box<dyn std::error::Error>> {
    // Open the binary and memory-map it
    let file = fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;

    // Determine endianness
    let endian = if object.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };

    // Load .eh_frame section
    let eh_frame_data = object
        .section_by_name(".eh_frame")
        .ok_or("No .eh_frame section found")?
        .uncompressed_data()?;

    // Wrap the section data in an EndianSlice
    let eh_frame_section = object
        .section_by_name(".eh_frame")
        .ok_or("No .eh_frame section found")?;

    let eh_frame_data = eh_frame_section.uncompressed_data()?;
    let eh_frame_address = eh_frame_section.address();
    let bases = BaseAddresses::default().set_eh_frame(eh_frame_address);
    let eh_frame = EhFrame::new(&eh_frame_data, endian);
    // Parse the entries in .eh_frame
    let mut entries = eh_frame.entries(&bases);

    while let Some(entry) = entries.next()? {
        match entry {
            gimli::CieOrFde::Cie(cie) => {
                //println!("CIE: {:?}", cie);
            }
            gimli::CieOrFde::Fde(partial) => {
                let fde = partial
                    .parse(|section, bases, offset| eh_frame.cie_from_offset(bases, offset))
                    .unwrap();

                // Get the start and end PC range this FDE covers
                let start = fde.initial_address();
                let end = start + fde.len();

                println!("FDE for address range: 0x{:x} - 0x{:x}", start, end);

                if target_addr >= start && target_addr < end {
                    println!("Target address 0x{:x} is within this FDE", target_addr);

                    // Get unwind info (instructions) for this address
                    let mut ctx = UnwindContext::new();

                    let row =
                        fde.unwind_info_for_address(&eh_frame, &bases, &mut ctx, target_addr)?;

                    println!("Unwind info for 0x{:x}:", target_addr);
                    println!("CFA: {:?}", row.cfa());

                    for (reg, rule) in row.registers() {
                        println!("Register {:?} = {:?}", reg, rule);
                    }

                    let (cfa_register, cfa_offset) = match row.cfa() {
                        gimli::CfaRule::RegisterAndOffset { register, offset } => {
                            (register.0, *offset)
                        }
                        rule => return Err(format!("Unsupported CFA rule: {:?}", rule).into()),
                    };

                    let ra_offset = match row.register(gimli::X86_64::RA) {
                        gimli::RegisterRule::Offset(off) => off,
                        rule => return Err(format!("Unsupported RA rule: {:?}", rule).into()),
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

    return Err(format!("No FDE found for address 0x{:x}", target_addr).into());
}
