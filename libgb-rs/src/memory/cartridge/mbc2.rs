use crate::memory::MemoryWriteError;

use super::{CartridgeMapper, RomBank, ROM_BANK_SIZE};

pub const MBC2_MEM_SIZE: usize = 512;

pub struct MBC2 {
    rom: Vec<RomBank>,
    ram: [u8; MBC2_MEM_SIZE],
    bank: u8,
    ram_enabled: bool,
    has_battery: bool
}

impl CartridgeMapper for MBC2 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let address = address as usize;
        let mut bank = self.bank as usize;
        if address < ROM_BANK_SIZE {
            bank = 0;
        }
        
        self.rom.get(bank)?
            .get(address)
            .copied()
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError> {
        if address >= (ROM_BANK_SIZE as u16) {
            return Ok(());
        }
        // look at bit 8 to check whether the rom bank should be changed
        // or the ram should be enabled
        if address & 0x0100 == 0 {
           self.ram_enabled = data == 0x0A; 
        } else {
            let bank = data & 0xF;
            self.bank = if bank != 0 { bank } else { 1 }
        }
        Ok(())
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
