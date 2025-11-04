pub mod ident;
pub mod machine;
pub mod r#type;

use crate::elf::header::ident::Identifier;
use crate::elf::header::machine::Machine;
use crate::elf::header::r#type::Type;
use crate::{le16, le32};

/// Representing the ELF Object File Format header in memory, equivalent to Elf32_Ehdr in 2. ELF
/// header in https://gabi.xinuos.com/elf/02-eheader.html
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

#[cfg(test)]
mod tests {
    fn valid_armv7_header_bytes() -> [u8; 52] {
        let mut bytes = [0u8; 52];

        // e_ident (16 bytes)
        bytes[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']); // magic
        bytes[4] = 1; // ELFCLASS32
        bytes[5] = 1; // ELFDATA2LSB
        bytes[6] = 1; // EV_CURRENT
        bytes[7] = 0; // OSABI_NONE
        bytes[8] = 0; // ABI version
        bytes[9..16].fill(0); // padding

        // e_type (16..18) = ET_EXEC
        bytes[16..18].copy_from_slice(&2u16.to_le_bytes());

        // e_machine (18..20) = EM_ARM
        bytes[18..20].copy_from_slice(&40u16.to_le_bytes());

        // e_version (20..24) = 1
        bytes[20..24].copy_from_slice(&1u32.to_le_bytes());

        // e_entry (24..28) = 0x8000
        bytes[24..28].copy_from_slice(&0x8000u32.to_le_bytes());

        // e_phoff (28..32) = 0
        bytes[28..32].copy_from_slice(&0u32.to_le_bytes());

        // e_shoff (32..36) = 0
        bytes[32..36].copy_from_slice(&0u32.to_le_bytes());

        // e_flags (36..40) = 0
        bytes[36..40].copy_from_slice(&0u32.to_le_bytes());

        // e_ehsize (40..42) = 52
        bytes[40..42].copy_from_slice(&52u16.to_le_bytes());

        // e_phentsize (42..44) = 32 (common)
        bytes[42..44].copy_from_slice(&32u16.to_le_bytes());

        // e_phnum (44..46) = 1
        bytes[44..46].copy_from_slice(&1u16.to_le_bytes());

        // e_shentsize (46..48) = 40 (common)
        bytes[46..48].copy_from_slice(&40u16.to_le_bytes());

        // e_shnum (48..50) = 0
        bytes[48..50].copy_from_slice(&0u16.to_le_bytes());

        // e_shstrndx (50..52) = 0
        bytes[50..52].copy_from_slice(&0u16.to_le_bytes());

        bytes
    }

    #[test]
    fn test_valid_header() {
        let bytes = valid_armv7_header_bytes();
        let header = crate::elf::header::Header::try_from(&bytes[..])
            .expect("should parse valid ARMv7 ELF header");
        assert_eq!(header.r#type, crate::elf::header::r#type::Type::Executable);
        assert_eq!(header.machine, crate::elf::header::machine::Machine::EM_ARM);
        assert_eq!(header.entry, 0x8000);
        assert_eq!(header.ehsize, 52);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = valid_armv7_header_bytes();
        bytes[0] = 0x00;
        assert!(crate::elf::header::Header::try_from(&bytes[..]).is_err());
    }

    #[test]
    fn test_invalid_class() {
        let mut bytes = valid_armv7_header_bytes();
        bytes[4] = 2; // ELFCLASS64
        assert!(crate::elf::header::Header::try_from(&bytes[..]).is_err());
    }

    #[test]
    fn test_invalid_machine() {
        let mut bytes = valid_armv7_header_bytes();
        bytes[18..20].copy_from_slice(&62u16.to_le_bytes()); // X86_64
        assert!(crate::elf::header::Header::try_from(&bytes[..]).is_err());
    }

    #[test]
    fn test_invalid_type() {
        let mut bytes = valid_armv7_header_bytes();
        bytes[16..18].copy_from_slice(&1u16.to_le_bytes()); // ET_REL
        assert!(crate::elf::header::Header::try_from(&bytes[..]).is_err());
    }

    #[test]
    fn test_too_short() {
        let bytes = [0u8; 10];
        assert!(crate::elf::header::Header::try_from(&bytes[..]).is_err());
    }
}
