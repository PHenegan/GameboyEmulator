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
        // only use the first 9 bits since there are only 512 entries in memory
        let address = (address & 0x1FF) as usize;
        if self.ram_enabled {
            self.ram.get(address).copied()
        } else {
            Some(0xFF)
        }
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        if !self.ram_enabled {
            return Ok(0xFF)
        }
        // only use the first 9 bits since there are only 512 entries in memory
        let address = (address & 0x1FF) as usize;
        let half_byte = self.ram.get_mut(address)
            .ok_or(MemoryWriteError)?;
        let old_value = *half_byte;

        // only use the lower 4 bits of the address, leaving the rest as 0
        // (technically the behavior is undefined for actual MBC2 cartridges)
        *half_byte = data & 0xF;

        Ok(old_value)
    }
}
