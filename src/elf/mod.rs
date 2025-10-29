pub mod header;

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
