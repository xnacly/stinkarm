pub mod header;
pub mod pheader;

/// Representing an ELF32 binary in memory
///
/// This does not include section headers (Elf32_Shdr), but only program headers (Elf32_Phdr), see either `man elf` and/or https://gabi.xinuos.com/elf/03-sheader.html
#[derive(Debug)]
pub struct Elf {
    pub header: header::Header,
    pub pheaders: Vec<pheader::Pheader>,
}

impl TryFrom<&[u8]> for Elf {
    type Error = String;

    fn try_from(b: &[u8]) -> Result<Self, String> {
        let header = header::Header::try_from(b).map_err(|e| e.to_string())?;

        let mut pheaders = Vec::with_capacity(header.phnum as usize);
        for i in 0..header.phnum {
            let offset = header.phoff as usize + i as usize * header.phentsize as usize;
            let ph = pheader::Pheader::from(b, offset)?;
            pheaders.push(ph);
        }

        Ok(Elf { header, pheaders })
    }
}

use std::fmt;

impl fmt::Display for Elf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = &self.header;

        writeln!(f, "ELF Header:")?;
        writeln!(f, "  Magic:              {:02x?}", h.ident.magic)?;
        writeln!(f, "  Class:              ELF32")?;
        writeln!(f, "  Data:               Little endian")?;
        writeln!(f, "  Type:               {:?}", h.r#type)?;
        writeln!(f, "  Machine:            {:?}", h.machine)?;
        writeln!(f, "  Version:            {}", h.version)?;
        writeln!(f, "  Entry point:        0x{:x}", h.entry)?;
        writeln!(
            f,
            "  Program hdr offset: {} ({} bytes each)",
            h.phoff, h.phentsize
        )?;
        writeln!(f, "  Section hdr offset: {}", h.shoff)?;
        writeln!(f, "  Flags:              0x{:08x}", h.flags)?;
        writeln!(f, "  EH size:            {}", h.ehsize)?;
        writeln!(f, "  # Program headers:  {}", h.phnum)?;
        writeln!(f, "  # Section headers:  {}", h.shnum)?;
        writeln!(f, "  Str tbl index:      {}", h.shstrndx)?;
        writeln!(f)?;

        if self.pheaders.is_empty() {
            writeln!(f, "No program headers")?;
            return Ok(());
        }

        writeln!(f, "Program Headers:")?;
        writeln!(
            f,
            "  {:<8} {:>8} {:>10} {:>10} {:>8} {:>8} {:>6} {:>6}",
            "Type", "Offset", "VirtAddr", "PhysAddr", "FileSz", "MemSz", "Flags", "Align"
        )?;

        for ph in &self.pheaders {
            writeln!(
                f,
                "  {:<8} 0x{:06x} 0x{:08x} 0x{:08x} 0x{:06x} 0x{:06x} {:>6} 0x{:x}",
                format!("{:?}", ph.r#type),
                ph.offset,
                ph.vaddr,
                ph.paddr,
                ph.filesz,
                ph.memsz,
                match ph.flags.bits() {
                    0 => "NONE",
                    1 => "X",
                    2 => "W",
                    3 => "W|X",
                    4 => "R",
                    5 => "R|X",
                    6 => "R|W",
                    7 => "R|W|X",
                    _ => "???",
                },
                ph.align
            )?;
        }

        Ok(())
    }
}
