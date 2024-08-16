use crate::memory::cartridge::{CartridgeMapper, MemBank, ROM_BANK_SIZE, RomBank};
use crate::memory::MemoryWriteError;

const SET_ROM_BANK_START: usize = 0x2000;
const SET_RAM_BANK_START: usize = 0x4000;
const LATCH_CLOCK_START: usize = 0x6000;
const ROM_END: usize = 0x8000;

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
        let address = address as usize;
        match address {
            0..SET_ROM_BANK_START => {
                self.ram_enabled = data == 0xA0;
                Ok(())
            }
            SET_ROM_BANK_START..SET_RAM_BANK_START => {
                self.rom_bank = data & 0x7F;
                Ok(())
            }
            SET_RAM_BANK_START..LATCH_CLOCK_START {
                self.ram_bank = data & 0x0F;
                Ok(())
            }
            LATCH_CLOCK_START..ROM_END => {
                todo!()
            }
            _ => Err(MemoryWriteError)
        }
    }

    fn read_mem(&self, address: u16) -> Option<u8> {
        todo!()
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        todo!()
    }
}