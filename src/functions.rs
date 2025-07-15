use goblin::Object;
use std::fs;
use::log::{debug, info};

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
        match Object::parse(&buffer).unwrap() {
            Object::Elf(elf) => {
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
