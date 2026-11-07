//! Architectural register file.

use crate::flags::Flags;

/// 16-bit register selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reg16 {
    /// Accumulator.
    Ax,
    /// Counter.
    Cx,
    /// Data.
    Dx,
    /// Base.
    Bx,
    /// Stack pointer.
    Sp,
    /// Base pointer.
    Bp,
    /// Source index.
    Si,
    /// Destination index.
    Di,
    /// Code segment.
    Cs,
    /// Data segment.
    Ds,
    /// Extra segment.
    Es,
    /// Stack segment.
    Ss,
    /// Instruction pointer.
    Ip,
}

impl Reg16 {
    /// All 16-bit registers in the order a debugger should show them.
    pub const ALL: [Self; 13] = [
        Self::Ax,
        Self::Bx,
        Self::Cx,
        Self::Dx,
        Self::Sp,
        Self::Bp,
        Self::Si,
        Self::Di,
        Self::Cs,
        Self::Ds,
        Self::Es,
        Self::Ss,
        Self::Ip,
    ];

    /// Three-letter name used by the debugger UI.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ax => "AX",
            Self::Bx => "BX",
            Self::Cx => "CX",
            Self::Dx => "DX",
            Self::Sp => "SP",
            Self::Bp => "BP",
            Self::Si => "SI",
            Self::Di => "DI",
            Self::Cs => "CS",
            Self::Ds => "DS",
            Self::Es => "ES",
            Self::Ss => "SS",
            Self::Ip => "IP",
        }
    }
}

/// 8-bit register selector, including the high halves of `AX`-`DX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reg8 {
    /// Low half of `AX`.
    Al,
    /// Low half of `CX`.
    Cl,
    /// Low half of `DX`.
    Dl,
    /// Low half of `BX`.
    Bl,
    /// High half of `AX`.
    Ah,
    /// High half of `CX`.
    Ch,
    /// High half of `DX`.
    Dh,
    /// High half of `BX`.
    Bh,
}

impl Reg8 {
    /// All 8-bit registers in encoding order (`AL`..`BH`).
    pub const ALL: [Self; 8] = [
        Self::Al,
        Self::Cl,
        Self::Dl,
        Self::Bl,
        Self::Ah,
        Self::Ch,
        Self::Dh,
        Self::Bh,
    ];

    /// Two-letter name used by the debugger UI.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Al => "AL",
            Self::Cl => "CL",
            Self::Dl => "DL",
            Self::Bl => "BL",
            Self::Ah => "AH",
            Self::Ch => "CH",
            Self::Dh => "DH",
            Self::Bh => "BH",
        }
    }
}

/// The complete 8086 register file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Registers {
    /// Accumulator.
    pub ax: u16,
    /// Base.
    pub bx: u16,
    /// Counter.
    pub cx: u16,
    /// Data.
    pub dx: u16,
    /// Stack pointer.
    pub sp: u16,
    /// Base pointer.
    pub bp: u16,
    /// Source index.
    pub si: u16,
    /// Destination index.
    pub di: u16,
    /// Code segment.
    pub cs: u16,
    /// Data segment.
    pub ds: u16,
    /// Extra segment.
    pub es: u16,
    /// Stack segment.
    pub ss: u16,
    /// Instruction pointer.
    pub ip: u16,
    /// Flags.
    pub flags: Flags,
}

impl Registers {
    /// Reads a 16-bit register.
    #[must_use]
    pub const fn get(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::Ax => self.ax,
            Reg16::Bx => self.bx,
            Reg16::Cx => self.cx,
            Reg16::Dx => self.dx,
            Reg16::Sp => self.sp,
            Reg16::Bp => self.bp,
            Reg16::Si => self.si,
            Reg16::Di => self.di,
            Reg16::Cs => self.cs,
            Reg16::Ds => self.ds,
            Reg16::Es => self.es,
            Reg16::Ss => self.ss,
            Reg16::Ip => self.ip,
        }
    }

    /// Writes a 16-bit register.
    pub fn set(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::Ax => self.ax = value,
            Reg16::Bx => self.bx = value,
            Reg16::Cx => self.cx = value,
            Reg16::Dx => self.dx = value,
            Reg16::Sp => self.sp = value,
            Reg16::Bp => self.bp = value,
            Reg16::Si => self.si = value,
            Reg16::Di => self.di = value,
            Reg16::Cs => self.cs = value,
            Reg16::Ds => self.ds = value,
            Reg16::Es => self.es = value,
            Reg16::Ss => self.ss = value,
            Reg16::Ip => self.ip = value,
        }
    }

    /// Reads an 8-bit register (low or high half of `AX`-`DX`).
    #[must_use]
    pub const fn get8(&self, reg: Reg8) -> u8 {
        let wide = match reg {
            Reg8::Al | Reg8::Ah => self.ax,
            Reg8::Cl | Reg8::Ch => self.cx,
            Reg8::Dl | Reg8::Dh => self.dx,
            Reg8::Bl | Reg8::Bh => self.bx,
        };
        match reg {
            Reg8::Ah | Reg8::Ch | Reg8::Dh | Reg8::Bh => (wide >> 8) as u8,
            _ => wide as u8,
        }
    }

    /// Writes an 8-bit register, preserving the other half.
    pub fn set8(&mut self, reg: Reg8, value: u8) {
        let wide = match reg {
            Reg8::Al | Reg8::Ah => &mut self.ax,
            Reg8::Cl | Reg8::Ch => &mut self.cx,
            Reg8::Dl | Reg8::Dh => &mut self.dx,
            Reg8::Bl | Reg8::Bh => &mut self.bx,
        };
        match reg {
            Reg8::Ah | Reg8::Ch | Reg8::Dh | Reg8::Bh => {
                *wide = (*wide & 0x00FF) | (u16::from(value) << 8);
            }
            _ => {
                *wide = (*wide & 0xFF00) | u16::from(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_and_low_halves_are_independent() {
        let mut r = Registers::default();
        r.set8(Reg8::Al, 0x12);
        r.set8(Reg8::Ah, 0x34);
        assert_eq!(r.ax, 0x3412);
        assert_eq!(r.get8(Reg8::Al), 0x12);
        assert_eq!(r.get8(Reg8::Ah), 0x34);
    }

    #[test]
    fn writing_wide_register_updates_both_halves() {
        let mut r = Registers::default();
        r.set(Reg16::Dx, 0xABCD);
        assert_eq!(r.get8(Reg8::Dl), 0xCD);
        assert_eq!(r.get8(Reg8::Dh), 0xAB);
    }

    #[test]
    fn every_selector_round_trips() {
        let mut r = Registers::default();
        for (i, reg) in Reg16::ALL.into_iter().enumerate() {
            let value = 0x1000 + i as u16;
            r.set(reg, value);
            assert_eq!(r.get(reg), value);
        }
    }
}
