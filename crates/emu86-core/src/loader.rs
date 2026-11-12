use crate::{Cpu, Memory, Registers, COM_LOAD_ADDRESS};

const COM_PSP_SEGMENT: u16 = 0x0100;
const COM_STACK_POINTER: u16 = 0xFFFE;
const MAX_COM_IMAGE_SIZE: usize = (COM_STACK_POINTER - COM_LOAD_ADDRESS) as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError {
    EmptyImage,
    ImageTooLarge(usize),
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyImage => f.write_str("COM image is empty"),
            Self::ImageTooLarge(size) => {
                write!(
                    f,
                    "COM image has {size} bytes; maximum is {MAX_COM_IMAGE_SIZE} bytes"
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}

impl Cpu {
    pub fn from_com(image: &[u8]) -> Result<Self, LoadError> {
        if image.is_empty() {
            return Err(LoadError::EmptyImage);
        }
        if image.len() > MAX_COM_IMAGE_SIZE {
            return Err(LoadError::ImageTooLarge(image.len()));
        }

        let mut cpu = Self {
            regs: Registers {
                cs: COM_PSP_SEGMENT,
                ds: COM_PSP_SEGMENT,
                es: COM_PSP_SEGMENT,
                ss: COM_PSP_SEGMENT,
                ip: COM_LOAD_ADDRESS,
                sp: COM_STACK_POINTER,
                ..Registers::default()
            },
            ..Self::new()
        };
        cpu.mem
            .load_linear(Memory::physical(COM_PSP_SEGMENT, COM_LOAD_ADDRESS), image);
        cpu.mem.write16(COM_PSP_SEGMENT, 0, 0x20CD);
        cpu.mem
            .write16(COM_PSP_SEGMENT, 2, COM_PSP_SEGMENT + 0x1000);
        cpu.mem.write8(COM_PSP_SEGMENT, 0x80, 0);
        cpu.mem.write8(COM_PSP_SEGMENT, 0x81, 0x0D);
        cpu.mem.write16(COM_PSP_SEGMENT, COM_STACK_POINTER, 0);
        Ok(cpu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Flags, Registers};

    #[test]
    fn com_starts_at_the_entry_point_with_shared_segments_and_a_stack() {
        let cpu = Cpu::from_com(&[0xB8, 0x34, 0x12, 0xF4]).unwrap();
        assert_eq!(
            cpu.regs,
            Registers {
                cs: 0x0100,
                ds: 0x0100,
                es: 0x0100,
                ss: 0x0100,
                ip: 0x0100,
                sp: 0xFFFE,
                flags: Flags::power_on(),
                ..Registers::default()
            }
        );
        assert!(!cpu.halted);
        assert_eq!(
            &cpu.mem.as_slice()[0x1100..0x1104],
            &[0xB8, 0x34, 0x12, 0xF4]
        );
        assert!(cpu.mem.as_slice()[..0x1000].iter().all(|&byte| byte == 0));
        assert!(cpu.mem.as_slice()[0x1104..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn com_initializes_a_minimal_psp_and_zero_return_word() {
        let cpu = Cpu::from_com(&[0xF4]).unwrap();
        assert_eq!(cpu.mem.read16(0x0100, 0), 0x20CD);
        assert_eq!(cpu.mem.read16(0x0100, 2), 0x1100);
        assert_eq!(cpu.mem.read8(0x0100, 0x80), 0);
        assert_eq!(cpu.mem.read8(0x0100, 0x81), 0x0D);
        assert_eq!(cpu.mem.read16(0x0100, 0xFFFE), 0);
    }

    #[test]
    fn com_rejects_empty_and_oversized_images() {
        assert_eq!(Cpu::from_com(&[]).unwrap_err(), LoadError::EmptyImage);
        for size in [0xFEFF, 0xFF00, 0x10000, 0x100001] {
            assert_eq!(
                Cpu::from_com(&vec![0x90; size]).unwrap_err(),
                LoadError::ImageTooLarge(size)
            );
        }
    }

    #[test]
    fn largest_com_preserves_its_last_byte_and_the_return_word() {
        let cpu = Cpu::from_com(&vec![0xA5; 0xFEFE]).unwrap();
        assert!(cpu.mem.as_slice()[0x1100..0x10FFE]
            .iter()
            .all(|&byte| byte == 0xA5));
        assert_eq!(cpu.mem.read8(0x0100, 0xFFFD), 0xA5);
        assert_eq!(cpu.mem.read16(0x0100, 0xFFFE), 0);
        assert_eq!(cpu.mem.read8(0x1100, 0), 0);
    }

    #[test]
    fn loaded_com_executes_arithmetic_memory_and_stack_operations() {
        let mut cpu = Cpu::from_com(&[
            0xB8, 0xFF, 0x7F, 0x05, 0x01, 0x00, 0xA3, 0x00, 0x02, 0x50, 0x5B, 0xF4,
        ])
        .unwrap();
        for _ in 0..6 {
            cpu.step().unwrap();
        }
        assert!(cpu.halted);
        assert_eq!((cpu.regs.ax, cpu.regs.bx), (0x8000, 0x8000));
        assert_eq!(cpu.mem.read16(0x0100, 0x0200), 0x8000);
        assert_eq!(cpu.mem.read16(0x0100, 0xFFFC), 0x8000);
        assert_eq!(cpu.regs.sp, 0xFFFE);
        assert_eq!(cpu.regs.ip, 0x010B);
        assert!(cpu.regs.flags.contains(Flags::OF));
        assert!(!cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn loaded_com_terminates_through_dos_or_the_psp_return_address() {
        for (image, steps, ip, sp) in [
            (&[0xCD, 0x20][..], 1, 0x0102, 0xFFFE),
            (&[0xB8, 0x07, 0x4C, 0xCD, 0x21][..], 2, 0x0105, 0xFFFE),
            (&[0xC3][..], 2, 0x0002, 0x0000),
        ] {
            let mut cpu = Cpu::from_com(image).unwrap();
            for _ in 0..steps {
                assert!(!cpu.halted);
                cpu.step().unwrap();
            }
            assert!(cpu.halted);
            assert_eq!(cpu.regs.cs, 0x0100);
            assert_eq!((cpu.regs.ip, cpu.regs.sp), (ip, sp));
        }
    }
}
