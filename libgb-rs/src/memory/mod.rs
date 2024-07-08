use cartridge::CartridgeMemoryBankController;

use crate::utils::{Merge, Split};

mod cartridge;

pub struct MemoryWriteError;

pub trait MemoryController {
    fn load_byte(&self, address: u16) -> Option<u8>;
    fn load_half_word(&self, address: u16) -> Option<u16>;
    fn store_byte(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError>;  
    fn store_half_word(&mut self, address: u16, data: u16) -> Result<(), MemoryWriteError>;
}

// Some memory map constants
const DMG_ROM_END: u16 = 0x7FFF;
const DMG_VRAM_START: u16 = 0x8000;
const DMG_VRAM_END: u16 = 0x9FFF;
const DMG_EXT_START: u16 = 0xA000;
const DMG_EXT_END: u16 = 0xBFFF;
const DMG_RAM_START: u16 = 0xC000;
const DMG_RAM_END: u16 = 0xDFFF;
const DMG_RES_START: u16 = 0xFE00;
const DMG_RES_END: u16 = 0xFFFF;

const DMG_RAM_SIZE: usize = 8192;
const DMG_VRAM_SIZE: usize = 8192;
const DMG_RES_SIZE: usize = (DMG_RES_END - DMG_RES_START + 1) as usize;

pub struct DmgMemoryController {
    cartridge: Box<dyn CartridgeMemoryBankController>,
    ram: [u8; DMG_RAM_SIZE],
    vram: [u8; DMG_VRAM_SIZE],
    system: [u8; DMG_RES_SIZE]
}

impl DmgMemoryController {
    pub fn new(cartridge: Box<dyn CartridgeMemoryBankController>) -> DmgMemoryController {
        DmgMemoryController {
            cartridge,
            ram: [0; DMG_VRAM_SIZE],
            vram: [0; DMG_VRAM_SIZE],
            system: [0; DMG_RES_SIZE]
        }
    }
    fn get_byte(&self, address: u16) -> Option<&u8> {
        match address {
            0 ..= DMG_ROM_END => {
                self.cartridge.get_rom_byte(address)
            },
            DMG_EXT_START ..= DMG_EXT_END => {
                self.cartridge.get_mem_byte(address)
            },
            DMG_VRAM_START ..= DMG_VRAM_END => {
                Some(&self.vram[(address - DMG_VRAM_START) as usize])
            },
            DMG_RAM_START ..= DMG_RAM_END => {
                Some(&self.ram[(address - DMG_RAM_START) as usize])
            },
            DMG_RES_START ..= DMG_RES_END => {
                Some(&self.system[(address - DMG_RES_START) as usize])
            }
            _ => None
        }
    }

    fn get_byte_mut<'a>(&'a mut self, address: u16) -> Option<&'a mut u8> {
        match address {
            0 ..= DMG_ROM_END => {
                None
            },
            DMG_VRAM_START ..= DMG_VRAM_END => {
                Some(&mut self.vram[(address - DMG_VRAM_START) as usize])
            },
            DMG_EXT_START ..= DMG_EXT_END => {
                self.cartridge.get_mem_byte_mut(address)
            },
            DMG_RAM_START ..= DMG_RAM_END => {
                Some(&mut self.ram[(address - DMG_RAM_START) as usize])
            },
            DMG_RES_START ..= DMG_RES_END => {
                Some(&mut self.system[(address - DMG_RES_START) as usize])
            }
            _ => None
        }
    }
}

impl MemoryController for DmgMemoryController {
    fn load_byte(&self, address: u16) -> Option<u8> {
        self.get_byte(address).copied()
    }

    fn load_half_word(&self, address: u16) -> Option<u16> {
        let left = self.get_byte(address)?;
        let right = self.get_byte(address + 1)?;

        Some(left.merge(*right))
    }

    fn store_byte(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError> {
        let byte = self.get_byte_mut(address).ok_or(MemoryWriteError)?;
        *byte = data;

        Ok(())
    }

    fn store_half_word(&mut self, address: u16, data: u16) -> Result<(), MemoryWriteError> {
        let (left_data, right_data) = data.split();
        
        // Avoid using store_byte so that the function will not load the first half when the second 
        // half of the data cannot be loaded
        let left = self.get_byte_mut(address)
            .ok_or(MemoryWriteError)?;
        *left = left_data;
        let right = self.get_byte_mut(address)
            .ok_or(MemoryWriteError)?;
        *right = right_data;

        Ok(())
    }
}
