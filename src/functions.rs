use goblin::Object;
use std::{borrow, env, error, fs, io::Read};

#[derive(Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub offset: u64, //offset of base
    pub size: u64,
}

impl FunctionInfo {
    pub fn new(path: String, debuger_name: String) -> Vec<FunctionInfo> {
        println!("debuger_name: {}", debuger_name); // Debg. name probably not needed anymore like this
        let buffer = fs::read(path).unwrap();
        let mut ret = Vec::new();
        match Object::parse(&buffer).unwrap() {
            Object::Elf(elf) => {
                for sym in elf.syms.iter() {
                    if sym.is_function() {
                        if let Some(Ok(name)) = elf.strtab.get(sym.st_name){
                            println!("{} {}", name, sym.st_value);
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
