use gimli::Reader as _;
use object::{Object, ObjectSection};
use std::{borrow, env, error, fs};

#[derive(Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub start_addr: u64,
    pub end_addr: u64,
}

impl FunctionInfo {
    pub fn new(path: String, debuger_name: String) -> Vec<FunctionInfo> {
        let file = fs::File::open(path).unwrap();
        let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };
        let object = object::File::parse(&*mmap).unwrap();
        let endian = if object.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };
        match Self::dump_file(&object, None, endian) {
            Ok(ret) => ret,
            Err(_) => Vec::new(),
        }
    }

    fn dump_file(
        object: &object::File,
        dwp_object: Option<&object::File>,
        endian: gimli::RunTimeEndian,
    ) -> Result<Vec<FunctionInfo>, Box<dyn error::Error>> {
        // Load a `Section` that may own its data.
        fn load_section<'data>(
            object: &object::File<'data>,
            name: &str,
        ) -> Result<Section<'data>, Box<dyn error::Error>> {
            Ok(match object.section_by_name(name) {
                Some(section) => Section {
                    data: section.uncompressed_data()?,
                    relocations: section.relocation_map().map(RelocationMap)?,
                },
                None => Default::default(),
            })
        }

        // Borrow a `Section` to create a `Reader`.
        fn borrow_section<'data>(
            section: &'data Section<'data>,
            endian: gimli::RunTimeEndian,
        ) -> Reader<'data> {
            let slice = gimli::EndianSlice::new(borrow::Cow::as_ref(&section.data), endian);
            gimli::RelocateReader::new(slice, &section.relocations)
        }

        // Load all of the sections.
        let dwarf_sections = gimli::DwarfSections::load(|id| load_section(object, id.name()))?;
        let dwp_sections = dwp_object
            .map(|dwp_object| {
                gimli::DwarfPackageSections::load(|id| {
                    load_section(dwp_object, id.dwo_name().unwrap())
                })
            })
            .transpose()?;

        let empty_relocations = RelocationMap::default();
        let empty_section =
            gimli::RelocateReader::new(gimli::EndianSlice::new(&[], endian), &empty_relocations);

        // Create `Reader`s for all of the sections and do preliminary parsing.
        // Alternatively, we could have used `Dwarf::load` with an owned type such as `EndianRcSlice`.
        let dwarf = dwarf_sections.borrow(|section| borrow_section(section, endian));
        let dwp = dwp_sections
            .as_ref()
            .map(|dwp_sections| {
                dwp_sections.borrow(|section| borrow_section(section, endian), empty_section)
            })
            .transpose()?;

        // Iterate over the compilation units.

        let mut all_functions = Vec::new();

        let mut iter = dwarf.units();
        while let Some(header) = iter.next()? {
            let unit = dwarf.unit(header)?;
            let unit_ref = unit.unit_ref(&dwarf);
            all_functions.extend(Self::dump_unit(unit_ref)?);
        }
        Ok(all_functions)
    }

    fn dump_unit(unit: gimli::UnitRef<Reader>) -> Result<Vec<FunctionInfo>, gimli::Error> {
        let mut functions = Vec::new();
        let mut entries = unit.entries();

        while let Some((delta_depth, entry)) = entries.next_dfs()? {
            if entry.tag() == gimli::DW_TAG_subprogram {
                let mut attrs = entry.attrs();
                let mut linkage_name_found = false;
                let mut linkage_name_string = String::new();
                let mut name = String::new();
                let mut start_address = None;
                let mut end_address = None;

                while let Some(attr) = attrs.next()? {
                    match attr.name() {
                        gimli::DW_AT_linkage_name => {
                            if let Ok(s) = unit.attr_string(attr.value()) {
                                linkage_name_string = s.to_string_lossy()?.to_string();
                                linkage_name_found = true;
                            }
                        }
                        gimli::DW_AT_name => {
                            if let Ok(s) = unit.attr_string(attr.value()) {
                                name = s.to_string_lossy()?.to_string();
                            }
                        }
                        gimli::DW_AT_low_pc => {
                            if let gimli::AttributeValue::Addr(addr) = attr.value() {
                                start_address = Some(addr);
                            }
                        }
                        gimli::DW_AT_high_pc => match attr.value() {
                            gimli::AttributeValue::Addr(addr) => end_address = Some(addr),
                            gimli::AttributeValue::Udata(size) => {
                                if let Some(start) = start_address {
                                    end_address = Some(start + size);
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                // If the linkage name contains "test_program", collect the data
                if linkage_name_found && linkage_name_string.contains("test_program") {
                    if let (Some(start), Some(end)) = (start_address, end_address) {
                        functions.push(FunctionInfo {
                            name: name.clone(),
                            start_addr: start,
                            end_addr: end,
                        });
                    }
                }
            }
        }
        Ok(functions)
    }
}

// The section data that will be stored in `DwarfSections` and `DwarfPackageSections`.
#[derive(Default)]
struct Section<'data> {
    data: borrow::Cow<'data, [u8]>,
    relocations: RelocationMap,
}

// The reader type that will be stored in `Dwarf` and `DwarfPackage`.
// If you don't need relocations, you can use `gimli::EndianSlice` directly.
type Reader<'data> =
    gimli::RelocateReader<gimli::EndianSlice<'data, gimli::RunTimeEndian>, &'data RelocationMap>;

#[derive(Debug, Default)]
struct RelocationMap(object::read::RelocationMap);

impl<'a> gimli::read::Relocate for &'a RelocationMap {
    fn relocate_address(&self, offset: usize, value: u64) -> gimli::Result<u64> {
        Ok(self.0.relocate(offset as u64, value))
    }

    fn relocate_offset(&self, offset: usize, value: usize) -> gimli::Result<usize> {
        <usize as gimli::ReaderOffset>::from_u64(self.0.relocate(offset as u64, value as u64))
    }
}
