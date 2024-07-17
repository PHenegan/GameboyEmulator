use crate::memory::MemoryWriteError;

enum StorageMode {
    ROM = 0,
    RAM = 1
}

impl From<u8> for StorageMode {
    fn from(value: u8) -> Self {
        if value % 2 == 0 { StorageMode::ROM } else { StorageMode::RAM }
    }
}


use super::CartridgeMemoryBankController;

const ROM_BANK_SIZE: usize = 16384;
const RAM_BANK_SIZE: usize = 8192;

pub type RomBank = [u8; ROM_BANK_SIZE];
pub type MemBank = [u8; RAM_BANK_SIZE];

pub struct MBC1 {
    rom: Vec<RomBank>,
    ram: Vec<MemBank>,
    storage_mode: StorageMode,
    rom_bank: u8,
    ram_bank: u8,
    has_battery: bool
}

impl MBC1 {

    // TODO - figure out how I want to take in the fields and convert them into banks 
    pub fn new() -> MBC1 {
        MBC1 {
            rom: vec!(),
            ram: vec!(),
            storage_mode: StorageMode::ROM,
            rom_bank: 1,
            ram_bank: 1,
            has_battery: false,
        }
    }

    /// Set the lower 5 bits of the rom bank value
    fn set_lower_rom_bank(&mut self, data: u8) {
        self.rom_bank = (self.rom_bank & 0xE0) & (data & 0x1F);
        // hardware bug present in MBC1 cartridges
        if self.rom_bank % 10 == 0 {
            self.rom_bank += 1;
        }
   }

    /// Set the upper 3 bits of the rom bank value, or the ram bank value
    /// depending on the storage mode of the cartridge
    fn set_ram_bank(&mut self, data: u8) {
        match self.storage_mode {
            StorageMode::ROM => self.rom_bank = (self.rom_bank & 0x1F) & (data << 5),
            StorageMode::RAM => self.ram_bank = data
        }
    }
}

impl CartridgeMemoryBankController for MBC1 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        todo!()
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(),MemoryWriteError> {
        // TODO - does writing to the ROM change it? I'm assuming no?
        // Ignoring the "enable" call because there are no electronic components to actually
        // enable
        match address {
            0x2000 ..= 0x3FFF => {
                self.set_lower_rom_bank(data);
                Ok(())
            },
            0x4000 ..= 0x5FFF => {
                self.set_ram_bank(data);
                Ok(())
            },
            0x6000 ..= 0x7FFF => {
                self.storage_mode = data.into();
                Ok(())
            }
            _ => Err(MemoryWriteError)
        }
    }

    fn read_mem(&self, address: u16) -> Option<u8> {
        if address as usize > RAM_BANK_SIZE {
            todo!()
        }
        todo!()
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8,MemoryWriteError> {
        todo!()
    }
}
