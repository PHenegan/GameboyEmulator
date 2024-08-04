use crate::memory::MemoryWriteError;

use super::{CartridgeMapper, RomBank};

pub const MBC2_MEM_SIZE = 512;

pub struct MBC2 {
    rom: Vec<RomBank>,
    ram: [u8; MBC2_MEM_SIZE],
    bank: u8,
    ram_enabled: bool,
    has_battery: bool
}

impl CartridgeMapper for MBC2 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        todo!()
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError> {
        todo!()
    }

    fn read_mem(&self, address: u16) -> Option<u8> {
        let address = address % 0x200;
        if self.ram_enabled {
            self.ram.get(address as usize).copied()
        } else {
            Some(0xFF)
        }
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        todo!()
    }
}
