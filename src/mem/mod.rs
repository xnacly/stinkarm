pub mod mmap;

use std::ptr::NonNull;

pub const DEFAULT_GUEST_MEMORY_SIZE: usize = 128 * 1024 * 1024;
const NULL_PAGE_SIZE: u32 = 0x1000;

pub struct Mem {
    ptr: NonNull<u8>,
    len: usize,
    bounds_checks: bool,
}

impl Default for Mem {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem {
    pub fn new() -> Self {
        Self::with_size(DEFAULT_GUEST_MEMORY_SIZE)
    }

    pub fn with_size(size: usize) -> Self {
        Self::with_size_and_bounds_checks(size, true)
    }

    pub fn with_bounds_checks(bounds_checks: bool) -> Self {
        Self::with_size_and_bounds_checks(DEFAULT_GUEST_MEMORY_SIZE, bounds_checks)
    }

    pub fn with_size_and_bounds_checks(size: usize, bounds_checks: bool) -> Self {
        let ptr = mmap::mmap(
            None,
            size,
            mmap::MmapProt::READ | mmap::MmapProt::WRITE,
            mmap::MmapFlags::ANONYMOUS | mmap::MmapFlags::PRIVATE,
            -1,
            0,
        )
        .expect("failed to allocate guest memory");

        Self {
            ptr,
            len: size,
            bounds_checks,
        }
    }

    /// Copy bytes into guest memory at `guest_addr`.
    pub fn map_region(&mut self, guest_addr: u32, data: &[u8]) -> Result<(), String> {
        let dst = self
            .get_slice_mut(guest_addr, data.len())
            .ok_or_else(|| format!("guest region out of bounds at {guest_addr:#010x}"))?;
        dst.copy_from_slice(data);
        Ok(())
    }

    /// Zero a range in guest memory.
    pub fn zero_region(&mut self, guest_addr: u32, len: usize) -> Result<(), String> {
        let dst = self
            .get_slice_mut(guest_addr, len)
            .ok_or_else(|| format!("guest region out of bounds at {guest_addr:#010x}"))?;
        dst.fill(0);
        Ok(())
    }

    /// Translate a guest address to a host pointer.
    pub fn translate(&self, guest_addr: u32) -> Option<*mut u8> {
        self.translate_range(guest_addr, 1)
    }

    /// Translate a guest address range to a host pointer to the first byte.
    pub fn translate_range(&self, guest_addr: u32, len: usize) -> Option<*mut u8> {
        if self.bounds_checks && !self.in_bounds(guest_addr, len) {
            return None;
        }

        Some(self.ptr.as_ptr().wrapping_add(guest_addr as usize))
    }

    pub fn read_u32(&self, guest_addr: u32) -> Option<u32> {
        if self.bounds_checks {
            let bytes = self.get_slice(guest_addr, 4)?;
            return Some(u32::from_le_bytes(bytes.try_into().ok()?));
        }

        let ptr = self.translate_range(guest_addr, 4)?;
        Some(u32::from_le(unsafe {
            (ptr as *const u32).read_unaligned()
        }))
    }

    pub fn write_u32(&mut self, guest_addr: u32, value: u32) -> Result<(), &'static str> {
        if self.bounds_checks {
            let dst = self
                .get_slice_mut(guest_addr, 4)
                .ok_or("Failed compute host addr to write to")?;
            dst.copy_from_slice(&value.to_le_bytes());
            return Ok(());
        }

        let ptr = self
            .translate_range(guest_addr, 4)
            .ok_or("Failed compute host addr to write to")?;
        unsafe { (ptr as *mut u32).write_unaligned(value.to_le()) };
        Ok(())
    }

    fn get_slice(&self, guest_addr: u32, len: usize) -> Option<&[u8]> {
        if self.bounds_checks && !self.in_bounds(guest_addr, len) {
            return None;
        }

        if !self.in_bounds(guest_addr, len) {
            return None;
        }

        Some(unsafe { std::slice::from_raw_parts(self.ptr.as_ptr().add(guest_addr as usize), len) })
    }

    fn get_slice_mut(&mut self, guest_addr: u32, len: usize) -> Option<&mut [u8]> {
        if self.bounds_checks && !self.in_bounds(guest_addr, len) {
            return None;
        }

        if !self.in_bounds(guest_addr, len) {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts_mut(self.ptr.as_ptr().add(guest_addr as usize), len)
        })
    }

    fn in_bounds(&self, guest_addr: u32, len: usize) -> bool {
        if guest_addr < NULL_PAGE_SIZE {
            return false;
        }

        let start = guest_addr as usize;
        let Some(end) = start.checked_add(len) else {
            return false;
        };

        end <= self.len
    }
}

impl Drop for Mem {
    fn drop(&mut self) {
        if let Err(e) = mmap::munmap(self.ptr, self.len) {
            eprintln!("Warning: failed to munmap guest memory: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Mem;

    #[test]
    fn translate_uses_guest_address_as_arena_offset() {
        let mem = Mem::with_size(0x4000);
        let base = mem.translate(0x1000).expect("guest address should map");
        let next = mem.translate(0x1004).expect("guest address should map");

        assert_eq!(unsafe { base.add(4) }, next);
    }

    #[test]
    fn checked_memory_rejects_null_page_and_out_of_bounds_ranges() {
        let mem = Mem::with_size(0x2000);

        assert!(mem.translate(0).is_none());
        assert!(mem.translate(0xfff).is_none());
        assert!(mem.translate_range(0x1ffe, 2).is_some());
        assert!(mem.translate_range(0x1ffe, 3).is_none());
    }

    #[test]
    fn unchecked_translation_skips_bounds_checks() {
        let mem = Mem::with_size_and_bounds_checks(0x2000, false);

        assert!(mem.translate(0).is_some());
        assert!(mem.translate_range(0xffff_ffff, 4).is_some());
    }

    #[test]
    fn read_and_write_u32_are_little_endian() {
        let mut mem = Mem::with_size(0x2000);

        mem.write_u32(0x1000, 0x1234_abcd)
            .expect("write should fit");

        assert_eq!(mem.read_u32(0x1000), Some(0x1234_abcd));
    }
}
