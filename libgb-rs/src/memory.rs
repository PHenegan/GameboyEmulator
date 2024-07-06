use crate::utils::{Merge, Split};

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
const DMG_RAM_START: u16 = 0xC000;
const DMG_RAM_END: u16 = 0xDFFF;

const DMG_RAM_SIZE: usize = 8192;
const DMG_VRAM_SIZE: usize = 8192;

pub struct DmgMemoryController {
    ram: [u8; DMG_RAM_SIZE],
    vram: [u8; DMG_VRAM_SIZE]
}

impl DmgMemoryController {
    pub fn new() -> DmgMemoryController {
        DmgMemoryController {
            ram: [0; DMG_VRAM_SIZE],
            vram: [0; DMG_VRAM_SIZE]
        }
    }
    fn get_byte(&self, address: u16) -> Option<&u8> {
        match address {
            0 ..= DMG_ROM_END => {
                // handle fetching memory from ROM here
                todo!()
            },
            DMG_VRAM_START ..= DMG_VRAM_END => {
                Some(&self.vram[(address - DMG_VRAM_START) as usize])
            },
            DMG_RAM_START ..= DMG_RAM_END => {
                Some(&self.ram[(address - DMG_RAM_START) as usize])
            },
            _ => None
        }
    }

    fn get_byte_mut<'a>(&'a mut self, address: u16) -> Option<&'a mut u8> {
        match address {
            0 ..= DMG_ROM_END => {
                // handle fetching memory from ROM here
                todo!()
            },
            DMG_VRAM_START ..= DMG_VRAM_END => {
                Some(&mut self.vram[(address - DMG_VRAM_START) as usize])
            },
            DMG_RAM_START ..= DMG_RAM_END => {
                Some(&mut self.ram[(address - DMG_RAM_START) as usize])
            },
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
