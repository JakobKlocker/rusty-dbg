use std::{fs, borrow};
use gimli::{BaseAddresses, EhFrame, EndianSlice, UnwindSection, RunTimeEndian};
use object::{Object, ObjectSection};
use memmap2::Mmap;

pub fn load_eh_frame(path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    let eh_frame = EhFrame::new(&eh_frame_data, endian);

    // You need the base addresses for relocations
    let bases = BaseAddresses::default();

    // Parse the entries in .eh_frame
    let mut entries = eh_frame.entries(&bases);

    while let Some(entry) = entries.next()? {
        match entry {
            gimli::CieOrFde::Cie(cie) => {
                println!("CIE: {:?}", cie);
            }
            gimli::CieOrFde::Fde(partial) => {
            }
        }
    }

    Ok(())
}