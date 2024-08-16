use crate::memory::cartridge::{CartridgeMapper, MemBank, ROM_BANK_SIZE, RomBank};
use crate::memory::MemoryWriteError;

pub struct MBC3 {
    rom: Vec<RomBank>,
    ram: Vec<MemBank>,
    ram_enabled: bool,
    ram_bank: u8,
    rom_bank: u8,
    has_battery: bool
    // TODO - Add fields for RTC
}

impl CartridgeMapper for MBC3 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let mut bank = 0;
        let mut address = address as usize;
        if (address >= ROM_BANK_SIZE) {
            bank = self.rom_bank;
            address -= ROM_BANK_SIZE;
        }

        self.rom.get(self.rom_bank as usize)?
            .get(address)
            .copied()
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError> {
        todo!()
    }

    fn read_mem(&self, address: u16) -> Option<u8> {
        todo!()
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        todo!()
    }
}