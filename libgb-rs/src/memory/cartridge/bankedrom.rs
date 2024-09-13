use crate::memory::MemoryWriteError;

use super::{RAM_BANK_SIZE, ROM_BANK_SIZE};

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
    manual_bank_logic: bool // flag for overriding bank 0 behavior
}

impl BankedRom {
    pub fn new(rom: Vec<u8>, ram_banks: usize, manual_bank_logic: bool) -> BankedRom {
        let ram_size = RAM_BANK_SIZE * ram_banks;
        // TODO - 0 padding needed for the rom
        BankedRom {
            rom,
            rom_bank: 1,
            ram: vec![0; ram_size],
            ram_bank: 0,
            manual_bank_logic
        }
    }

    pub fn set_rom_bank(&mut self, bank: usize) {
        self.rom_bank = bank;
    }

    pub fn read_rom(&self, address: u16) -> Option<u8> {
        let address = address as usize;
        let offset = address & 0x3FFF; // address inside of the bank (up to 16KB)
        // bank #
        let tag = if !self.manual_bank_logic && offset == address { 0 } else { self.rom_bank };

        let rom_address = (tag << 14) | offset;

        self.rom.get(rom_address)
            .copied()
    }

    pub fn set_ram_bank(&mut self, bank: usize) {
        self.ram_bank = bank;
    }

    pub fn read_mem(&self, address: u16) -> Option<u8> {
        let offset = address as usize & 0x1FFF; // address inside of the bank (up to 8KB)
        let ram_address = (self.ram_bank << 13) | offset;
        self.ram.get(ram_address)
            .copied()
    }

    pub fn write_mem(&mut self, address: u16, value: u8) -> Result<u8, MemoryWriteError> {
        let address = address as usize & 0x1FFF; // address inside of the bank (up to 8KB)
        let ram_address = (self.ram_bank << 13) | address;
        let byte = self.ram.get_mut(ram_address)
            .ok_or(MemoryWriteError)?;
        let old_value = byte.clone();
        *byte = value;

        Ok(old_value)
    }

    // TODO - think about how this would interact with RTC functionality
    pub fn load_save(&mut self, save_bytes: Vec<u8>) {
        // TODO - some extra logic needed here
        self.ram.copy_from_slice(save_bytes.as_slice());
        todo!();
    }

    // TODO - think about how this would interact with RTC functionality
    pub fn save(&self) -> Vec<u8> {
        todo!();
    }
}

