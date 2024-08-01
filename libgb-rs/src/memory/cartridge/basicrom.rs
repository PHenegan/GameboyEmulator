use crate::memory::cartridge::CartridgeMapper;
use crate::memory::MemoryWriteError;

const ROM_SIZE: usize = 32768;
const RAM_SIZE: usize = 8192;

pub struct RomOnlyCartridge {
    // Question - Does having a battery mean that the RAM is persistent?
    // (i.e. the battery is what allows for a save file?)
    rom: [u8; ROM_SIZE],
    ram: Option<[u8; RAM_SIZE]>,
}

impl RomOnlyCartridge {
    pub fn new(
        rom: [u8; ROM_SIZE], ram: Option<[u8; RAM_SIZE]>, _has_battery: bool
    ) -> RomOnlyCartridge {
        RomOnlyCartridge { rom, ram }
    }
}

impl CartridgeMapper for RomOnlyCartridge {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let address = address as usize;
        self.rom.get(address).copied()
    }

    fn write_rom(&mut self, _address: u16, _data: u8) -> Result<(), MemoryWriteError> {
        Err(MemoryWriteError)
    }

    fn read_mem(&self, address: u16) -> Option<u8> {
        let address = address as usize;
        self.ram.as_ref()?.get(address).copied()
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        match self.ram.as_mut() {
            Some(ram) => {
                let address = address as usize;
                let prev = ram.get(address)
                    .ok_or(MemoryWriteError)?.clone();
                let byte = ram.get_mut(address)
                    .ok_or(MemoryWriteError)?;
                *byte = data;
                Ok(prev)
            },
            None => Err(MemoryWriteError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_rom_valid_address() {
        let mut rom = [0; ROM_SIZE];
        rom[2450] = 128;
        let controller = RomOnlyCartridge::new(rom, None, false);

        let result = controller.read_rom(2450);

        assert_eq!(result, Some(128), "Test reading from ROM");
    }

    #[test]
    fn test_read_rom_invalid_address() {
        let rom = [0; ROM_SIZE];
        let controller = RomOnlyCartridge::new(rom, None, false);

        let result = controller.read_rom(0x8000);

        assert_eq!(result, None, "Test that invalid addresses returns 'None'");
    }

    #[test]
    fn test_write_rom_address() {
        let rom = [0; ROM_SIZE];
        let mut controller = RomOnlyCartridge::new(rom, None, false);

        let result = controller.write_rom(0, 12);

        assert_eq!(result, Err(MemoryWriteError), "Writing to ROM is not supported");
    }

    #[test]
    fn test_read_mem_valid_address() {
        let rom = [0; ROM_SIZE];
        let mut ram = [0; RAM_SIZE];
        ram[4096] = 200;
        let controller = RomOnlyCartridge::new(rom, Some(ram), false);

        let result = controller.read_mem(4096);

        assert_eq!(result, Some(200), "Test reading from RAM");
    }

    #[test]
    fn test_read_mem_no_ram() {
        let rom = [0; ROM_SIZE];
        let controller = RomOnlyCartridge::new(rom, None, false);

        let result = controller.read_mem(4096);

        assert_eq!(result, None, "Test reading from RAM when there is no RAM");
    }

    #[test]
    fn test_read_mem_invalid_address() {
        let rom = [0; ROM_SIZE];
        let ram = [0; RAM_SIZE];
        let controller = RomOnlyCartridge::new(rom, Some(ram), false);

        let result = controller.read_mem(8192);

        assert_eq!(result, None, "Test reading from invalid RAM address");
    }

    #[test]
    fn test_write_mem_valid_address() {
        let rom = [0; ROM_SIZE];
        let mut ram = [0; RAM_SIZE];
        ram[4096] = 30;
        let mut controller = RomOnlyCartridge::new(rom, Some(ram), false);

        let result = controller.write_mem(4096, 200);

        assert_eq!(result, Ok(30), "Test writing to a valid RAM address");
        assert_eq!(controller.read_mem(4096), Some(200), "Test Writing to RAM");
    }

    #[test]
    fn test_write_mem_no_ram() {
        let rom = [0; ROM_SIZE];
        let mut controller = RomOnlyCartridge::new(rom, None, false);

        let result = controller.write_mem(1024, 200);

        assert!(result.is_err(), "Test writing when there is no RAM");
    }

    #[test]
    fn test_write_mem_invalid_address() {
        let rom = [0; ROM_SIZE];
        let ram = [0; RAM_SIZE];
        let mut controller = RomOnlyCartridge::new(rom, Some(ram), false);

        let result = controller.write_mem(0x2000, 200);

        assert!(result.is_err(), "Test writing to invalid address")
    }
}
