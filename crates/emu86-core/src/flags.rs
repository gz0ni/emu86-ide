//! The 8086 flags register.

/// Architectural flags register.
///
/// Only the nine documented bits are exposed as constants; the remaining bits
/// are kept as they are because real 8086 always reads them as `1` and some
/// programs rely on that when they do `pushf` / `popf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags(pub u16);

impl Flags {
    /// Carry flag.
    pub const CF: u16 = 0x0001;
    /// Parity flag.
    pub const PF: u16 = 0x0004;
    /// Auxiliary carry flag.
    pub const AF: u16 = 0x0010;
    /// Zero flag.
    pub const ZF: u16 = 0x0040;
    /// Sign flag.
    pub const SF: u16 = 0x0080;
    /// Trap (single-step) flag.
    pub const TF: u16 = 0x0100;
    /// Interrupt enable flag.
    pub const IF: u16 = 0x0200;
    /// Direction flag.
    pub const DF: u16 = 0x0400;
    /// Overflow flag.
    pub const OF: u16 = 0x0800;

    /// Bits 1, 12-15 are hard-wired to `1` on every member of the 8086 family.
    pub const RESERVED: u16 = 0xF002;

    /// Flags in the order a debugger should present them.
    pub const NAMED: [(&'static str, u16); 9] = [
        ("OF", Self::OF),
        ("DF", Self::DF),
        ("IF", Self::IF),
        ("TF", Self::TF),
        ("SF", Self::SF),
        ("ZF", Self::ZF),
        ("AF", Self::AF),
        ("PF", Self::PF),
        ("CF", Self::CF),
    ];

    /// Value of the flags register right after a hardware reset.
    #[must_use]
    pub const fn power_on() -> Self {
        Self(Self::RESERVED)
    }

    /// Raw 16-bit value.
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Builds a flags register from a raw value, keeping reserved bits set.
    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits | Self::RESERVED)
    }

    /// Returns `true` when every bit in `mask` is set.
    #[must_use]
    pub const fn contains(self, mask: u16) -> bool {
        self.0 & mask == mask
    }

    /// Turns the bits in `mask` on or off.
    pub fn set(&mut self, mask: u16, on: bool) {
        if on {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::power_on()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_bits_are_always_set() {
        let f = Flags::from_bits(0);
        assert_eq!(f.bits(), Flags::RESERVED);
    }

    #[test]
    fn set_and_clear_a_flag() {
        let mut f = Flags::power_on();
        f.set(Flags::ZF, true);
        assert!(f.contains(Flags::ZF));
        f.set(Flags::ZF, false);
        assert!(!f.contains(Flags::ZF));
    }

    #[test]
    fn all_named_flags_are_distinct() {
        let all: u16 = Flags::NAMED.iter().fold(0, |acc, (_, bit)| acc | bit);
        assert_eq!(all, 0x0FD5);
    }
}
