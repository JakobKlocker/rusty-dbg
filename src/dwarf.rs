use gimli::RunTimeEndian;
use memmap2::Mmap;
use object::{Object, ObjectSection};
use std::{borrow, error, fs};
use std::path::PathBuf;

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

        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section =
            |section| gimli::EndianSlice::new(borrow::Cow::as_ref(section), self.endian);

        // Load all of the sections.
        let dwarf_sections = gimli::DwarfSections::load(&load_section).unwrap();

        // Create `EndianSlice`s for all of the sections.
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
                                path.push(unit.attr_string(dir).unwrap().to_string_lossy().as_ref());
                            }
                        }

                        path.push(
                            unit.attr_string(file.path_name()).unwrap()
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
