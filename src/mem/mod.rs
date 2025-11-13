use std::collections::BTreeMap;

pub mod mmap;

struct MappedSegment {
    host_ptr: *mut u8,
    len: u32,
}

pub struct Mem {
    maps: BTreeMap<u32, MappedSegment>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            maps: BTreeMap::new(),
        }
    }

    pub fn map_region(&mut self, guest_addr: u32, len: u32, host_ptr: *mut u8) {
        self.maps
            .insert(guest_addr, MappedSegment { host_ptr, len });
    }

    /// translate a guest addr to a host addr we can write and read from
    ///
    /// ```text
    /// +--guest--+
    /// | 0x80000 | ------------+
    /// +---------+             |
    ///                         |
    ///                     Mem::translate
    ///                         |
    /// +------host------+      |
    /// | 0x7f5b4b8f8000 | <----+
    /// +----------------+
    /// ```
    pub fn translate(&self, guest_addr: u32) -> Option<*mut u8> {
        // Find the greatest key <= guest_addr.
        let (&base, seg) = self.maps.range(..=guest_addr).next_back()?;
        if guest_addr < base.wrapping_add(seg.len) {
            let offset = guest_addr.wrapping_sub(base);
            Some(unsafe { seg.host_ptr.add(offset as usize) })
        } else {
            None
        }
    }

    pub fn read_u32(&self, guest_addr: u32) -> Option<u32> {
        let ptr = self.translate(guest_addr)?;
        unsafe { Some(u32::from_le(*(ptr as *const u32))) }
    }

    pub fn write_u32(&mut self, guest_addr: u32, value: u32) -> Result<(), &'static str> {
        let ptr = self
            .translate(guest_addr)
            .ok_or_else(|| "Failed compute host addr to write to")?;
        unsafe { *(ptr as *mut u32) = value.to_le() }
        Ok(())
    }

    /// dropping all segments, consumes self to make it single use and not allow any self usages
    /// after dropping
    pub fn destroy(self) {
        for (guest_addr, seg) in self.maps {
            if let Some(nnptr) = std::ptr::NonNull::new(seg.host_ptr) {
                if let Err(e) = mmap::munmap(nnptr, seg.len as usize) {
                    eprintln!(
                        "Warning: failed to munmap guest segment @ {:#010x} (len={}): {:?}",
                        guest_addr, seg.len, e
                    );
                }
            }
        }
    }
}
