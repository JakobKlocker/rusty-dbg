use crate::core::Debugger;
use anyhow::Result;
use gimli::{BaseAddresses, EhFrame, RunTimeEndian, UnwindContext, UnwindSection};
use goblin::Object as GoblinObject;
use log::{debug, info};
use memmap2::Mmap;
use object::{Object, ObjectSection};
use std::{borrow, error, fs, path::PathBuf};

#[derive(Debug)]
#[allow(dead_code)]
pub struct DwarfContext {
    pub mmap: Mmap,
    pub endian: RunTimeEndian,
    pub object: object::File<'static>,
}

impl DwarfContext {
    pub fn new(path: &str) -> Result<Self, Box<dyn error::Error>> {
        let file = fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let mmap_static: &'static [u8] = unsafe { std::mem::transmute(&*mmap) };
        let object = object::File::parse(mmap_static)?;

        let endian = if object.is_little_endian() {
            RunTimeEndian::Little
        } else {
            RunTimeEndian::Big
        };

        Ok(Self {
            mmap,
            endian,
            object,
        })
    }

    pub fn get_line_and_file(&self, target_addr: u64) {
        let load_section =
            |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, Box<dyn error::Error>> {
                Ok(match self.object.section_by_name(id.name()) {
                    Some(section) => section.uncompressed_data()?,
                    None => borrow::Cow::Borrowed(&[]),
                })
            };

        let borrow_section =
            |section| gimli::EndianSlice::new(borrow::Cow::as_ref(section), self.endian);

        let dwarf_sections = gimli::DwarfSections::load(&load_section).unwrap();

        let dwarf = dwarf_sections.borrow(borrow_section);

        let mut units = dwarf.units();
        while let Some(header) = units.next().unwrap() {
            let unit = dwarf.unit(header).unwrap();
            let unit = unit.unit_ref(&dwarf);

            if let Some(program) = unit.line_program.clone() {
                let comp_dir = unit
                    .comp_dir
                    .as_ref()
                    .map(|d| PathBuf::from(d.to_string_lossy().into_owned()))
                    .unwrap_or_default();

                let mut rows = program.rows();
                let mut last_match = None;

                while let Some((header, row)) = rows.next_row().unwrap() {
                    if row.end_sequence() {
                        continue;
                    }

                    if row.address() > target_addr {
                        break;
                    }

                    last_match = Some((header.clone(), row.clone()));
                }

                if let Some((header, row)) = last_match {
                    if let Some(file) = row.file(&header) {
                        let mut path = comp_dir;

                        if file.directory_index() != 0 {
                            if let Some(dir) = file.directory(&header) {
                                path.push(
                                    unit.attr_string(dir).unwrap().to_string_lossy().as_ref(),
                                );
                            }
                        }

                        path.push(
                            unit.attr_string(file.path_name())
                                .unwrap()
                                .to_string_lossy()
                                .as_ref(),
                        );

                        let line = row.line().map(|l| l.get()).unwrap_or(0);
                        println!("{}:{}", path.display(), line);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FunctionInfo {
    pub name: String,
    pub offset: u64, //offset of base
    pub size: u64,
}

impl FunctionInfo {
    pub fn new(path: &String, debuger_name: String) -> Vec<FunctionInfo> {
        info!("debuger_name: {}", debuger_name); // Debg. name probably not needed anymore like this
        let buffer = fs::read(path).unwrap();
        let mut ret = Vec::new();
        match GoblinObject::parse(&buffer).unwrap() {
            GoblinObject::Elf(elf) => {
                for sym in elf.syms.iter() {
                    if sym.is_function() {
                        if let Some(name) = elf.strtab.get_at(sym.st_name) {
                            debug!("{} {}", name, sym.st_value);
                            ret.push(FunctionInfo {
                                name: name.to_string(),
                                offset: sym.st_value,
                                size: sym.st_size,
                            })
                        }
                    }
                }
            }
            _ => {}
        }
        return ret;
    }
}

#[derive(Debug)]
pub struct UnwindRowInfo {
    pub cfa_register: u16,
    pub cfa_offset: i64,
    pub ra_offset: i64,
}

pub fn get_unwind_info(path: &str, target_addr: u64) -> Result<UnwindRowInfo> {
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
            gimli::CieOrFde::Cie(_cie) => {}
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
                        rule => {
                            return Err(anyhow::anyhow!("Unsupported CFA rule: {:?}", rule).into())
                        }
                    };

                    let ra_offset = match row.register(gimli::X86_64::RA) {
                        gimli::RegisterRule::Offset(off) => off,
                        rule => {
                            return Err(anyhow::anyhow!("Unsupported RA rule: {:?}", rule).into())
                        }
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
    return Err(anyhow::anyhow!("No FDE found for address 0x{:x}", target_addr).into());
    //need better way to detect end of backtrace
}

pub trait Symbols {
    fn print_sections(&self) -> Result<()>;
}

impl Symbols for Debugger {
    fn print_sections(&self) -> Result<()> {
        let data = fs::read(&self.path)?;
        let obj_file = object::File::parse(&*data)?;
        for section in obj_file.sections() {
            println!(
                "Section: {:<20} Addr: 0x{:08x}, Size: 0x{:x}",
                section.name().unwrap_or("<unnamed>"),
                section.address(),
                section.size(),
            );
        }
        Ok(())
    }
}
