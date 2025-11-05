use crate::{
    le32, mem,
    sys::{
        self,
        mmap::{MmapFlags, MmapProt},
    },
};

/// Phdr, equivalent to Elf32_Phdr, see: https://gabi.xinuos.com/elf/07-pheader.html
///
/// All of its member are u32, be it Elf32_Word, Elf32_Off or Elf32_Addr
#[derive(Debug)]
pub struct Pheader {
    pub r#type: Type,
    /// offset to the segment, starting from file idx
    pub offset: u32,
    /// virtual address where the first byte of the segment lays
    pub vaddr: u32,
    /// On systems for which physical addressing is relevant, this member is reserved for the
    /// segment’s physical address. Because System V ignores physical addressing for application
    /// programs, this member has unspecified contents for executable files and shared objects.
    pub paddr: u32,
    /// This member gives the number of bytes in the file image of the segment; it may be zero.
    pub filesz: u32,
    /// This member gives the number of bytes in the memory image of the segment; it may be zero.
    pub memsz: u32,
    pub flags: Flags,
    /// Loadable process segments must have congruent values for p_vaddr and p_offset, modulo the page
    /// size. This member gives the value to which the segments are aligned in memory and in the
    /// file. Values 0 and 1 mean no alignment is required. Otherwise, p_align should be a
    /// positive, integral power of 2, and p_vaddr should equal p_offset, modulo p_align.
    pub align: u32,
}

impl Pheader {
    /// extracts Pheader from raw, starting from offset
    pub fn from(raw: &[u8], offset: usize) -> Result<Self, String> {
        let end = offset.checked_add(32).ok_or("Offset overflow")?;
        if raw.len() < end {
            return Err("Not enough bytes to parse Elf32_Phdr, need at least 32".into());
        }

        let p_raw = &raw[offset..end];
        let r#type = p_raw[0..4].try_into()?;
        let flags = p_raw[24..28].try_into()?;
        let align = le32!(p_raw[28..32]);

        if align > 1 && !align.is_power_of_two() {
            return Err(format!("Invalid p_align: {}", align));
        }

        Ok(Self {
            r#type,
            offset: le32!(p_raw[4..8]),
            vaddr: le32!(p_raw[8..12]),
            paddr: le32!(p_raw[12..16]),
            filesz: le32!(p_raw[16..20]),
            memsz: le32!(p_raw[20..24]),
            flags,
            align,
        })
    }

    /// mapping applys the configuration of self to the current memory context by creating the
    /// segments with the corresponding permission bits, vaddr, etc
    pub fn map(&self, raw: &[u8], guest_mem: &mut mem::Mem) -> Result<(), String> {
        // zero memory needed case, no clue if this actually ever happens, but we support it
        if self.memsz == 0 {
            return Ok(());
        }

        if self.vaddr == 0 {
            return Err("program header has a zero virtual address".into());
        }

        // we need page alignement, so either Elf32_Phdr.p_align or 4096
        let (_, _, len) = self.alignments();

        // Instead of mapping at the guest vaddr (Linux doesnt't allow for low addresses),
        // we allocate memory wherever the host kernel gives us.
        // This keeps guest memory sandboxed: guest addr != host addr.
        let segment = sys::mmap::mmap(
            None, // let the kernel choose
            len as usize,
            MmapProt::WRITE,
            MmapFlags::ANONYMOUS | MmapFlags::PRIVATE,
            -1,
            0,
        )?;

        let segment_ptr = segment.as_ptr();
        let segment_slice = unsafe { std::slice::from_raw_parts_mut(segment_ptr, len as usize) };

        let file_slice: &[u8] = &raw[self.offset as usize..(self.offset + self.filesz) as usize];

        // copy the segment contents to the mmaped segment
        segment_slice[..file_slice.len()].copy_from_slice(file_slice);

        // we need to zero the remaining bytes
        if self.memsz > self.filesz {
            segment_slice[file_slice.len()..self.memsz as usize].fill(0);
        }

        // record mapping in guest memory table, so CPU can translate guest vaddr to host pointer
        guest_mem.map_region(self.vaddr, len, segment_ptr);

        // we change the permissions for our segment from W to the segments requested bits
        sys::mmap::mprotect(segment, len as usize, self.flags.into())
    }

    /// returns (start, end, len)
    fn alignments(&self) -> (u32, u32, u32) {
        // we need page alignement, so either Elf32_Phdr.p_align or 4096
        let align = match self.align {
            0 => 0x1000,
            _ => self.align,
        };
        let start = self.vaddr & !(align - 1);
        let end = (self.vaddr + self.memsz + align - 1) & !(align - 1);
        let len = end - start;
        (start, end, len)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Type {
    /// The array element is unused; other members’ values are undefined. This type lets the
    /// program header table have ignored entries.
    NULL = 0,
    /// The array element specifies a loadable segment, described by p_filesz and p_memsz. The
    /// bytes from the file are mapped to the beginning of the memory segment. If the segment’s
    /// memory size (p_memsz) is larger than the file size (p_filesz), the “extra” bytes are
    /// defined to hold the value 0 and to follow the segment’s initialized area. The file size may
    /// not be larger than the memory size. Loadable segment entries in the program header table
    /// appear in ascending order, sorted on the p_vaddr member.
    LOAD = 1,
    /// The array element specifies dynamic linking information. See Section 8.3, Dynamic Section,
    /// for more information.
    DYNAMIC = 2,
    /// The array element specifies the location and size of a null-terminated path name to invoke
    /// as an interpreter. This segment type is meaningful only for executable files (though it may
    /// occur for shared objects); it may not occur more than once in a file. If it is present, it
    /// must precede any loadable segment entry. See Section 8.1, Program Interpreter, for more
    /// information.
    INTERP = 3,
    /// The array element specifies the location and size of auxiliary information. See Section
    /// 7.6, Note Sections, for more information.
    NOTE = 4,
    /// This segment type is reserved but has unspecified semantics. Programs that contain an array
    /// element of this type do not conform to the ABI.
    SHLIB = 5,
    /// The array element, if present, specifies the location and size of the program header table
    /// itself, both in the file and in the memory image of the program. This segment type may not
    /// occur more than once in a file. Moreover, it may occur only if the program header table is
    /// part of the memory image of the program. If it is present, it must precede any loadable
    /// segment entry.
    PHDR = 6,
    /// The array element specifies the Thread-Local Storage template. Implementations need not
    /// support this program table entry. See Section 7.7, Thread-Local Storage, for more
    /// information.
    TLS = 7,
    /// Values in this inclusive range are reserved for operating system-specific semantics.
    LOOS = 0x60000000,
    HIOS = 0x6fffffff,
    /// Values in this inclusive range are reserved for processor-specific semantics. If meanings
    /// are specified, the psABI supplement explains them.
    LOPROC = 0x70000000,
    HIPROC = 0x7fffffff,
}

impl TryFrom<&[u8]> for Type {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            return Err("Elf32_Phdr.p_type requires exactly 4 bytes");
        }

        Ok(match le32!(value) {
            0 => Self::NULL,
            1 => Self::LOAD,
            2 => Self::DYNAMIC,
            3 => Self::INTERP,
            4 => Self::NOTE,
            5 => Self::SHLIB,
            6 => Self::PHDR,
            7 => Self::TLS,
            _ => return Err("Bad Elf32_Phdr.p_type value"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/// See 7.4. Segment Permission https://gabi.xinuos.com/elf/07-pheader.html#segment-permissions
pub struct Flags(u32);

impl Flags {
    pub const NONE: Self = Flags(0x0);
    pub const X: Self = Flags(0x1);
    pub const W: Self = Flags(0x2);
    pub const R: Self = Flags(0x4);

    pub fn bits(self) -> u32 {
        self.0
    }
}

impl TryFrom<&[u8]> for Flags {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            return Err("Not enough bytes for Elf32_Phdr.p_flags, need 4");
        }

        Ok(Self(le32!(value)))
    }
}

impl std::ops::BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Flags(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::elf::pheader::{Flags, Pheader, Type};

    fn valid_phdr_bytes() -> [u8; 32] {
        let mut bytes = [0u8; 32];

        // p_type = LOAD
        bytes[0..4].copy_from_slice(&1u32.to_le_bytes());

        // p_offset = 0x1000
        bytes[4..8].copy_from_slice(&0x1000u32.to_le_bytes());

        // p_vaddr = 0x8000
        bytes[8..12].copy_from_slice(&0x8000u32.to_le_bytes());

        // p_paddr = 0x8000
        bytes[12..16].copy_from_slice(&0x8000u32.to_le_bytes());

        // p_filesz = 0x200
        bytes[16..20].copy_from_slice(&0x200u32.to_le_bytes());

        // p_memsz = 0x300
        bytes[20..24].copy_from_slice(&0x300u32.to_le_bytes());

        // p_flags = R | X
        bytes[24..28].copy_from_slice(&(Flags::R | Flags::X).bits().to_le_bytes());

        // p_align = 0x1000
        bytes[28..32].copy_from_slice(&0x1000u32.to_le_bytes());

        bytes
    }

    #[test]
    fn test_valid_phdr() {
        let bytes = valid_phdr_bytes();
        let phdr = Pheader::from(&bytes, 0).expect("should parse valid Elf32_Phdr");

        assert_eq!(phdr.r#type, Type::LOAD);
        assert_eq!(phdr.offset, 0x1000);
        assert_eq!(phdr.vaddr, 0x8000);
        assert_eq!(phdr.paddr, 0x8000);
        assert_eq!(phdr.filesz, 0x200);
        assert_eq!(phdr.memsz, 0x300);
        assert_eq!(phdr.flags, Flags::R | Flags::X);
        assert_eq!(phdr.align, 0x1000);
    }

    #[test]
    fn test_invalid_too_short() {
        let bytes = [0u8; 20];
        assert!(Pheader::from(&bytes, 0).is_err());
    }

    #[test]
    fn test_invalid_type() {
        let mut bytes = valid_phdr_bytes();
        bytes[0..4].copy_from_slice(&99u32.to_le_bytes()); // invalid p_type
        assert!(Pheader::from(&bytes, 0).is_err());
    }

    #[test]
    fn test_invalid_flags_length() {
        // This test triggers via `Flags::try_from` directly.
        let short = [0u8; 3];
        assert!(Flags::try_from(&short[..]).is_err());
    }

    #[test]
    fn test_invalid_offset_index() {
        // offset past end of slice should fail
        let bytes = valid_phdr_bytes();
        assert!(Pheader::from(&bytes, 16).is_err());
    }

    #[test]
    fn test_valid_with_offset() {
        // embed valid phdr at offset 8 in a larger slice
        let mut big = [0u8; 40];
        let inner = valid_phdr_bytes();
        big[8..8 + 32].copy_from_slice(&inner);

        let phdr = Pheader::from(&big, 8).expect("should parse with offset 8");
        assert_eq!(phdr.r#type, Type::LOAD);
        assert_eq!(phdr.align, 0x1000);
    }

    #[test]
    fn test_flags_bitor() {
        let combined = Flags::R | Flags::W;
        assert_eq!(combined.bits(), Flags::R.bits() | Flags::W.bits());
    }
}
