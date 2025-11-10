/// 2.2 ELF Identification: https://gabi.xinuos.com/elf/02-eheader.html#elf-identification
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifier {
    /// 0x7F, 'E', 'L', 'F'
    pub magic: [u8; 4],
    /// file class or capacity
    ///
    /// | Name          | Value | Meaning       |
    /// | ------------- | ----- | ------------- |
    /// | ELFCLASSNONE  | 0     | Invalid class |
    /// | ELFCLASS32    | 1     | 32-bit        |
    /// | ELFCLASS64    | 2     | 64-bit        |
    pub class: u8,
    /// data encoding, endian
    ///
    /// | Name         | Value |
    /// | ------------ | ----- |
    /// | ELFDATANONE  | 0     |
    /// | ELFDATA2LSB  | 1     |
    /// | ELFDATA2MSB  | 2     |
    pub data: u8,
    /// file version, always EV_CURRENT (1)
    pub version: u8,
    /// operating system identification
    ///
    /// - if no extensions are used: 0
    /// - meaning depends on e_machine
    pub os_abi: u8,
    /// value depends on os_abi
    pub abi_version: u8,
    // padding bytes (9-15)
    _pad: [u8; 7],
}

impl TryFrom<&[u8]> for Identifier {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 16 {
            return Err("e_ident too short for ELF");
        }

        // I dont want to cast via unsafe as_ptr and as Header because the header could outlive the
        // source slice, thus we just do it the old plain indexing way
        let ident = Self {
            magic: bytes[0..4].try_into().unwrap(),
            class: bytes[4],
            data: bytes[5],
            version: bytes[6],
            os_abi: bytes[7],
            abi_version: bytes[8],
            _pad: bytes[9..16].try_into().unwrap(),
        };

        if ident.magic != [0x7f, b'E', b'L', b'F'] {
            return Err("Unexpected EI_MAG0 to EI_MAG3, wanted 0x7f E L F");
        }

        const ELFCLASS32: u8 = 1;
        const ELFDATA2LSB: u8 = 1;
        const EV_CURRENT: u8 = 1;

        if ident.version != EV_CURRENT {
            return Err("Unsupported EI_VERSION value");
        }

        if ident.class != ELFCLASS32 {
            return Err("Unexpected EI_CLASS: ELFCLASS64, wanted ELFCLASS32 (ARMv7)");
        }

        if ident.data != ELFDATA2LSB {
            return Err("Unexpected EI_DATA: big-endian, wanted little");
        }

        Ok(ident)
    }
}
