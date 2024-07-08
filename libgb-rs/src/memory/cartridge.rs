use std::usize;


pub trait CartridgeMemoryBankController {
    // TODO - think about timer, SRAM, etc. support
    fn get_rom_byte<'a>(&'a self, address: u16) -> Option<&'a u8>;
    fn get_mem_byte<'a>(&'a self, address: u16) -> Option<&'a u8>;
    fn get_mem_byte_mut<'a>(&'a mut self, address: u16) -> Option<&'a mut u8>;
}

const ROM_SIZE: usize = 32768;
const RAM_SIZE: usize = 8192;
pub struct RomOnlyCartridge {
    // Question - Does having a battery mean that the RAM is persistant?
    // (i.e. the battery is what allows for a save file?)
    rom: [u8; ROM_SIZE],
    ram: [u8; RAM_SIZE],
    ram_enabled: bool
}

impl RomOnlyCartridge {
    pub fn new(
        rom: [u8; ROM_SIZE], ram: Option<[u8; RAM_SIZE]>, _has_battery: bool
        ) -> RomOnlyCartridge {
        let ram_enabled = ram.is_some();
        RomOnlyCartridge { rom, ram: ram.unwrap_or([0; RAM_SIZE]), ram_enabled }
    }
}

impl CartridgeMemoryBankController for RomOnlyCartridge {
    fn get_rom_byte<'a>(&'a self, address: u16) -> Option<&'a u8> {
        let address = address as usize;
        self.rom.get(address)
    }

    fn get_mem_byte<'a>(&'a self, address: u16) -> Option<&'a u8> {
        if !self.ram_enabled {
            return None
        }
        self.ram.get(address as usize).to_owned()
    }

    fn get_mem_byte_mut<'a>(&'a mut self, address: u16) -> Option<&'a mut u8> {
        if !self.ram_enabled {
            return None
        }
        let address = address as usize;
        self.ram.get_mut(address)
    }
}
