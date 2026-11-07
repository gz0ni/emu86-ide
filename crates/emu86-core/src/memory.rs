//! Flat 1 MiB real-mode address space.

/// Size of the 8086 address space in bytes.
pub const MEMORY_SIZE: usize = 1024 * 1024;

/// 1 MiB of memory addressed through `segment:offset` pairs.
#[derive(Clone)]
pub struct Memory {
    bytes: Vec<u8>,
}

impl core::fmt::Debug for Memory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Memory")
            .field("size", &self.bytes.len())
            .finish()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    /// Allocates a zero-filled address space.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytes: vec![0; MEMORY_SIZE],
        }
    }

    /// Converts a real-mode `segment:offset` pair into a physical index,
    /// wrapping at the 1 MiB boundary like an original 8086 does.
    #[must_use]
    pub fn physical(segment: u16, offset: u16) -> usize {
        let linear = (u32::from(segment) << 4).wrapping_add(u32::from(offset));
        (linear as usize) & (MEMORY_SIZE - 1)
    }

    /// Raw view of the whole address space, for the memory window.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// Raw mutable view of the whole address space, for debugger writes.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Reads one byte.
    #[must_use]
    pub fn read8(&self, segment: u16, offset: u16) -> u8 {
        self.bytes[Self::physical(segment, offset)]
    }

    /// Writes one byte.
    pub fn write8(&mut self, segment: u16, offset: u16, value: u8) {
        let at = Self::physical(segment, offset);
        self.bytes[at] = value;
    }

    /// Reads a little-endian word.
    #[must_use]
    pub fn read16(&self, segment: u16, offset: u16) -> u16 {
        let lo = u16::from(self.read8(segment, offset));
        let hi = u16::from(self.read8(segment, offset.wrapping_add(1)));
        lo | (hi << 8)
    }

    /// Writes a little-endian word.
    pub fn write16(&mut self, segment: u16, offset: u16, value: u16) {
        self.write8(segment, offset, value as u8);
        self.write8(segment, offset.wrapping_add(1), (value >> 8) as u8);
    }

    /// Copies `data` into memory at a linear address, for loaders.
    ///
    /// # Panics
    ///
    /// Panics when the range does not fit into the address space. Loaders are
    /// expected to validate sizes before calling this.
    pub fn load_linear(&mut self, address: usize, data: &[u8]) {
        assert!(
            address + data.len() <= MEMORY_SIZE,
            "image does not fit in memory"
        );
        self.bytes[address..address + data.len()].copy_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_offset_translation() {
        assert_eq!(Memory::physical(0x0000, 0x0100), 0x0100);
        assert_eq!(Memory::physical(0x1234, 0x0005), 0x1234 * 16 + 5);
    }

    #[test]
    fn addresses_wrap_at_one_megabyte() {
        assert_eq!(Memory::physical(0xFFFF, 0xFFFF), 0x0FFEF);
    }

    #[test]
    fn words_are_little_endian() {
        let mut mem = Memory::new();
        mem.write16(0x1000, 0x0010, 0xBEEF);
        assert_eq!(mem.read8(0x1000, 0x0010), 0xEF);
        assert_eq!(mem.read8(0x1000, 0x0011), 0xBE);
        assert_eq!(mem.read16(0x1000, 0x0010), 0xBEEF);
    }

    #[test]
    fn load_linear_copies_the_image() {
        let mut mem = Memory::new();
        mem.load_linear(0x100, &[0x90, 0xCC]);
        assert_eq!(mem.read8(0x0000, 0x0100), 0x90);
        assert_eq!(mem.read8(0x0000, 0x0101), 0xCC);
    }
}
