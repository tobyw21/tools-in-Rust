use object::{Object, ObjectSection};
use std::fs;
use capstone::prelude::*;

pub struct DisassembleObject {
    dis_engine: Capstone,
    filedump: Vec<u8>,
}

impl DisassembleObject {
    pub fn new(filename: &str) -> Self {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create capstone object");


            let bytes = fs::read(filename).expect("error reading file");
            
            DisassembleObject {
                dis_engine: cs,
                filedump: bytes,
            }
            
    }

    pub fn disassemble(&self) {
        let objfile = object::File::parse(&*self.filedump).expect("error on parsing file");
        if let Some(section) = objfile.section_by_name(".text") {
            
            
            let base_address = section.address();
            let insns = self.dis_engine.disasm_all(&section.data(), base_address)
                .expect("Failed to disassemble");
            
            for instruction in insns.as_ref() {
                println!("{}", instruction);
            }

        } else {
            eprintln!("Section not avaliable or incorrect section!");
        }
    }
}