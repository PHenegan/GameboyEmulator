use crate::memory::MemoryWriteError;

use super::{LoadCartridgeError, SaveError, RAM_BANK_SIZE, ROM_BANK_SIZE};

/// # BankedRom
/// This is an abstraction (not intended to be exposed publicly) for Game Boy cartridges.
/// It is intended to reduce the amount of logic duplication between cartridges, and also to
/// make it easier to implement ROM loading, as well as save loading and dumping.
///
/// It will handle loading and dumping, but without any extra logic for determining rom banks
pub struct BankedRom {
    rom: Vec<u8>,
    rom_bank: usize,
    ram: Vec<u8>,
    ram_bank: usize,
    has_battery: bool, // whether or not the ROM supports saving
    manual_bank_logic: bool // flag for overriding bank 0 behavior
}

impl BankedRom {
    pub fn new(
        rom_bytes: Vec<u8>, rom_banks: usize,
        ram_banks: usize,
        has_battery: bool,
        manual_bank_logic: bool
    ) -> Result<BankedRom, LoadCartridgeError> {
        let ram_size = RAM_BANK_SIZE * ram_banks;
        let rom_size = ROM_BANK_SIZE * rom_banks;

        if rom_bytes.len() > rom_size {
            return Err(LoadCartridgeError);
        }

        // the copy method requires the two sides to have the same length, so a mutable slice
        // is used to copy over the ROM with memcpy
        let mut rom = vec![0; rom_size];
        let rom_to_load = &mut rom[0..rom_bytes.len()];
        rom_to_load.copy_from_slice(&rom_bytes[..]);

        Ok(
            BankedRom {
                rom,
                rom_bank: 1,
                ram: vec![0; ram_size],
                ram_bank: 0,
                has_battery,
                manual_bank_logic
            }
        )
    }

    pub fn set_rom_bank(&mut self, bank: usize) {
        let bank_count = self.rom.len() / ROM_BANK_SIZE;
        self.rom_bank = bank % bank_count;
    }

    pub fn read_rom(&self, address: u16) -> Option<u8> {
        if address >= 0x8000 {
            return None;
        }

        let address = address as usize;
        let offset = address & 0x3FFF; // address inside of the bank (up to 16KB)
        // bank #
        let tag = if !self.manual_bank_logic && offset == address { 0 } else { self.rom_bank };

        let rom_address = (tag << 14) | offset;

        self.rom.get(rom_address)
            .copied()
    }

    pub fn set_mem_bank(&mut self, bank: usize) {
        let bank_count = self.ram.len() / RAM_BANK_SIZE;
        self.ram_bank = bank % bank_count;
    }

    pub fn read_mem(&self, address: u16) -> Option<u8> {
        if address >= 0x2000 {
            return None;
        }

        let offset = address as usize & 0x1FFF; // address inside of the bank (up to 8KB)
        let ram_address = (self.ram_bank << 13) | offset;
        self.ram.get(ram_address)
            .copied()
    }

    pub fn write_mem(&mut self, address: u16, value: u8) -> Result<u8, MemoryWriteError> {
        if address >= 0x2000 {
            return Err(MemoryWriteError);
        }

        let address = address as usize & 0x1FFF; // address inside of the bank (up to 8KB)
        let ram_address = (self.ram_bank << 13) | address;
        let byte = self.ram.get_mut(ram_address)
            .ok_or(MemoryWriteError)?;
        let old_value = byte.clone();
        *byte = value;

        Ok(old_value)
    }

    // TODO - think about how this would interact with RTC functionality
    pub fn load_save(&mut self, save_data: Vec<u8>) -> Result<(), SaveError> {
        if !self.has_battery {
            return Err(SaveError::SavesNotSupported);
        }

        if save_data.len() > self.ram.len() {
            return Err(SaveError::SaveFileTooBig);
        }

        let slice = &mut self.ram[0..save_data.len()];
        slice.copy_from_slice(save_data.as_slice());
        
        Ok(())
    }

    // TODO - think about how this would interact with RTC functionality
    pub fn save(&self) -> Vec<u8> {
        self.ram.clone()
    }
}

