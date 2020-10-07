use crate::elf::*;

#[derive(Default)]
pub struct ElfHeader {
    pub ident: ElfIdent,
    pub filetype: ElfHalf,
    pub machine: ElfHalf,
    pub version: ElfWord,
    pub entrypoint: ElfAddr,
    pub program_header_offset: ElfOff,
    pub section_header_offset: ElfOff,
    pub flags: ElfWord,
    pub elf_header_size: ElfHalf,
    pub program_header_size: ElfHalf,
    pub program_header_num: ElfHalf,
    pub section_header_size: ElfHalf,
    pub section_header_num: ElfHalf,
    pub string_table_index: ElfHalf,
}

pub enum Class {
    ClassNone = 0,
    Class32 = 1,
    Class64 = 2,
}

pub enum Data {
    DataNone = 0,
    Data2LSB = 1,
    Data2MSB = 2,
}

pub enum OSABI {
    OSABISysV = 0,
}

pub enum Type {
    None = 0,
    Rel = 1,
    Exec = 2,
    Dyn = 3,
    Core = 4,
}

pub enum Machine {
    None = 0,
    X86_64 = 62,
}

impl ElfHeader {
    pub fn new() -> Self {
        let mut hdr: Self = Default::default();
        hdr.ident = 0x7f454c46 << 12 * 8;
        hdr.ident |= 0x1 << 9 * 8; // version
        hdr.version = 0x1;
        hdr.elf_header_size = 64;
        hdr.section_header_size = 64;
        hdr
    }

    pub fn set_class(&mut self, class: Class) {
        self.ident |= (class as u128) << 11 * 8;
    }

    pub fn set_data(&mut self, data: Data) {
        self.ident |= (data as u128) << 10 * 8;
    }

    pub fn set_osabi(&mut self, osabi: OSABI) {
        self.ident |= (osabi as u128) << 8 * 8;
    }

    pub fn set_filetype(&mut self, typ: Type) {
        self.filetype = typ as u16;
    }

    pub fn set_machine(&mut self, machine: Machine) {
        self.machine = machine as u16;
    }

    pub fn set_entrypoint(&mut self, addr: u64) {
        self.entrypoint = addr;
    }

    pub fn set_program_header_offset(&mut self, offset: u64) {
        self.program_header_offset = offset;
    }

    pub fn set_section_header_offset(&mut self, offset: u64) {
        self.section_header_offset = offset;
    }

    pub fn set_program_header_size(&mut self, size: u16) {
        self.program_header_size = size;
    }

    pub fn set_program_header_num(&mut self, num: u16) {
        self.program_header_num = num;
    }

    pub fn set_section_header_size(&mut self, size: u16) {
        self.section_header_size = size;
    }

    pub fn set_section_header_num(&mut self, num: u16) {
        self.section_header_num = num;
    }

    pub fn string_header_num(&mut self, index: u16) {
        self.string_table_index = index;
    }

    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend(&self.ident.to_be_bytes());
        buf.extend(&self.filetype.to_le_bytes());
        buf.extend(&self.machine.to_le_bytes());
        buf.extend(&self.version.to_le_bytes());
        buf.extend(&self.entrypoint.to_le_bytes());
        buf.extend(&self.program_header_offset.to_le_bytes());
        buf.extend(&self.section_header_offset.to_le_bytes());
        buf.extend(&self.flags.to_le_bytes());
        buf.extend(&self.elf_header_size.to_le_bytes());
        buf.extend(&self.program_header_size.to_le_bytes());
        buf.extend(&self.program_header_num.to_le_bytes());
        buf.extend(&self.section_header_size.to_le_bytes());
        buf.extend(&self.section_header_num.to_le_bytes());
        buf.extend(&self.string_table_index.to_le_bytes());
    }
}
