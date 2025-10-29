pub mod ident;
pub mod machine;
pub mod r#type;

use crate::elf::header::ident::Identifier;
use crate::elf::header::machine::Machine;
use crate::elf::header::r#type::Type;
use crate::{le16, le32};
use std::fmt;

/// Representing the ELF Object File Format header in memory, equivalent to
///
/// see Elf32_Ehdr in 2. ELF header in https://gabi.xinuos.com/elf/02-eheader.html
///
/// Types are taken from https://gabi.xinuos.com/elf/01-intro.html#data-representation Table 1.1
/// 32-Bit Data Types:
///
/// | Elf32_ | Rust |
/// | ------ | ---- |
/// | Addr   | u32  |
/// | Off    | u32  |
/// | Half   | u16  |
/// | Word   | u32  |
/// | Sword  | i32  |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// initial bytes mark the file as an object file and provide machine-independent data with
    /// which to decode and interpret the file’s contents
    pub ident: Identifier,
    pub r#type: Type,
    pub machine: Machine,
    /// identifies the object file version, always EV_CURRENT (1)
    pub version: u32,
    /// the virtual address to which the system first transfers control, thus starting
    /// the process. If the file has no associated entry point, this member holds zero
    pub entry: u32,
    /// the program header table’s file offset in bytes. If the file has no program header table,
    /// this member holds zero
    pub phoff: u32,
    /// the section header table’s file offset in bytes. If the file has no section header table, this
    /// member holds zero
    pub shoff: u32,
    /// processor-specific flags associated with the file
    pub flags: u32,
    /// the ELF header’s size in bytes
    pub ehsize: u16,
    /// the size in bytes of one entry in the file’s program header table; all entries are the same
    /// size
    pub phentsize: u16,
    /// the number of entries in the program header table. Thus the product of e_phentsize and e_phnum
    /// gives the table’s size in bytes. If a file has no program header table, e_phnum holds the value
    /// zero
    pub phnum: u16,
    /// section header’s size in bytes. A section header is one entry in the section header table; all
    /// entries are the same size
    pub shentsize: u16,
    /// number of entries in the section header table. Thus the product of e_shentsize and e_shnum
    /// gives the section header table’s size in bytes. If a file has no section header table,
    /// e_shnum holds the value zero.
    pub shnum: u16,
    /// the section header table index of the entry associated with the section name string table.
    /// If the file has no section name string table, this member holds the value SHN_UNDEF
    pub shstrndx: u16,
}

impl TryFrom<&[u8]> for Header {
    type Error = &'static str;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        if b.len() < 52 {
            return Err("not enough bytes for Elf32_Ehdr (ELF header)");
        }

        let header = Self {
            ident: b[0..16].try_into()?,
            r#type: le16!(b[16..18]).try_into()?,
            machine: le16!(b[18..20]).try_into()?,
            version: le32!(b[20..24]),
            entry: le32!(b[24..28]),
            phoff: le32!(b[28..32]),
            shoff: le32!(b[32..36]),
            flags: le32!(b[36..40]),
            ehsize: le16!(b[40..42]),
            phentsize: le16!(b[42..44]),
            phnum: le16!(b[44..46]),
            shentsize: le16!(b[46..48]),
            shnum: le16!(b[48..50]),
            shstrndx: le16!(b[50..52]),
        };

        match header.r#type {
            Type::Executable => (),
            _ => {
                return Err("Unsupported ELF type, only ET_EXEC (static executables) is supported");
            }
        }

        Ok(header)
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  Magic:\t\t\t\t")?;
        for b in &self.ident.magic {
            write!(f, "{:02x} ", b)?;
        }
        writeln!(f)?;

        writeln!(
            f,
            "  Class:\t\t\t\tELF{}",
            if self.ident.class == 1 { "32" } else { "64" }
        )?;
        writeln!(
            f,
            "  Data:\t\t\t\t\t{}",
            match self.ident.data {
                1 => "2's complement, little endian",
                2 => "2's complement, big endian",
                _ => "Unknown",
            }
        )?;
        writeln!(f, "  Version:\t\t\t\t{} (current)", self.ident.version)?;
        writeln!(f, "  OS/ABI:\t\t\t\t{}", self.ident.os_abi)?;
        writeln!(f, "  ABI Version:\t\t\t\t{}", self.ident.abi_version)?;
        writeln!(f, "  Type:\t\t\t\t\t{:?}", self.r#type)?;
        writeln!(f, "  Machine:\t\t\t\t{:?}", self.machine)?;
        writeln!(f, "  Version:\t\t\t\t0x{:X}", self.version)?;
        writeln!(f, "  Entry point address:\t\t\t0x{:X}", self.entry)?;
        writeln!(
            f,
            "  Start of program headers:\t\t{} (bytes into file)",
            self.phoff
        )?;
        writeln!(
            f,
            "  Start of section headers:\t\t{} (bytes into file)",
            self.shoff
        )?;
        writeln!(f, "  Flags:\t\t\t\t0x{:X}", self.flags)?;
        writeln!(f, "  Size of this header:\t\t\t{} (bytes)", self.ehsize)?;
        writeln!(
            f,
            "  Size of program headers:\t\t{} (bytes)",
            self.phentsize
        )?;
        writeln!(f, "  Number of program headers:\t\t{}", self.phnum)?;
        writeln!(
            f,
            "  Size of section headers:\t\t{} (bytes)",
            self.shentsize
        )?;
        writeln!(f, "  Number of section headers:\t\t{}", self.shnum)?;
        write!(f, "  Section header string table index:\t{}", self.shstrndx)
    }
}
