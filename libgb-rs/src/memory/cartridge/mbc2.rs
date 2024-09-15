use crate::memory::MemoryWriteError;

use super::{bankedrom::BankedRom, CartridgeMapper, LoadCartridgeError, SaveError, ROM_BANK_SIZE};

pub const MBC2_MEM_SIZE: usize = 512;

pub struct MBC2 {
    rom: BankedRom,
    ram: [u8; MBC2_MEM_SIZE],
    ram_enabled: bool,
    has_battery: bool
}

impl CartridgeMapper for MBC2 {
    fn create(
        rom: Vec<u8>, rom_banks: u8,
        _ram_banks: u8, has_battery:bool
    ) -> Result<MBC2, LoadCartridgeError> where Self:Sized {
        let rom = BankedRom::new(rom, rom_banks as usize, 0, false, false)?;
        let ram = [0; MBC2_MEM_SIZE];

        Ok(
            MBC2 {
                rom,
                ram,
                ram_enabled: false,
                has_battery
            }
        )
    }
    fn read_rom(&self, address: u16) -> Option<u8> {
        self.rom.read_rom(address)
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError> {
        if address > 0x7FFF {
            return Err(MemoryWriteError);
        }
        if address >= (ROM_BANK_SIZE as u16) {
            return Ok(());
        }
        // look at bit 8 to check whether the rom bank should be changed
        // or the ram should be enabled
        if address & 0x0100 == 0 {
           self.ram_enabled = data == 0x0A; 
        } else {
            let mut bank = data & 0x1F;
            bank = if bank != 0 { bank } else { 1 };
            self.rom.set_rom_bank(bank as usize);
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

    fn load_save(&mut self, save_data: Vec<u8>) -> Result<(), SaveError> {
        if !self.has_battery {
            return Err(SaveError::SavesNotSupported);
        }

        if save_data.len() > MBC2_MEM_SIZE {
            return Err(SaveError::SaveFileTooBig);
        }

        // Can't just do a copy because the data needs to be only 4 bits
        for idx in 0..save_data.len() {
            self.ram[idx] = save_data[idx] & 0xF;
        }

        Ok(())
    }

    fn save(&self) -> Vec<u8> {
        self.ram.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cartridge::RomBank;

    use super::*;

    fn init_mapper(rom: Vec<RomBank>, ram: [u8; MBC2_MEM_SIZE]) -> MBC2 {
        // I do this conversion because I changed how the ROM is stored and I don't want to change
        // all of the tests
        let sequential_rom = rom.concat();
        let ram = Vec::from(ram);

        let result = MBC2::create(sequential_rom, rom.len() as u8, 0, true);
        assert!(result.is_ok(), "Should create MBC2 object correctly");
        let mut cartridge = result.unwrap();

        let save_result = cartridge.load_save(ram);
        assert!(save_result.is_ok(), "Should load memory successfully");

        cartridge
    }

    #[test]
    fn test_read_bank_0() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 2];
        rom[0][0x4] = 0x28;
        let ram = [0; MBC2_MEM_SIZE];
        let mbc2 = init_mapper(rom, ram);

        let result = mbc2.read_rom(4);
        
        assert_eq!(result, Some(0x28), "Should be able to read from first half");
    }

    #[test]
    fn test_read_bank_0_after_switch() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 4];
        rom[0][0x4] = 0x28;
        let ram = [0; MBC2_MEM_SIZE];
        let mut mbc2 = init_mapper(rom, ram);

        let write_result = mbc2.write_rom(0x0106, 3);
        let read_result = mbc2.read_rom(0x0004);

        assert!(write_result.is_ok(), "Should be able to change ROM banks successfully");
        assert_eq!(read_result, Some(0x28), "Should still read from bank 0 in first half");
    }

    #[test]
    fn test_bank_switching() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 32];
        rom[1][0x3FFF] = 0xBE;
        rom[28][0x4] = 0x07;
        let ram = [0; MBC2_MEM_SIZE];
        let mut mbc2 = init_mapper(rom, ram);

        let bank0_read_result = mbc2.read_rom(0x7FFF);
        let write_result = mbc2.write_rom(0x0106, 28);
        let bank28_read_result = mbc2.read_rom(0x4004);

        assert_eq!(bank0_read_result, Some(0xBE), "Should be able to read from bank 1");
        assert!(write_result.is_ok(), "Should be able to switch banks");
        assert_eq!(bank28_read_result, Some(0x7), "Should be able to read from bank 28");
    }

    #[test]
    fn test_switch_to_bank_0() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 32];
        rom[1][0x42] = 0x42;
        let ram = [0; MBC2_MEM_SIZE];
        let mut mbc2 = init_mapper(rom, ram);

        let write_result = mbc2.write_rom(0x0106, 0);
        let read_result = mbc2.read_rom(0x4042);

        assert!(write_result.is_ok(), "Should still be able to switch to bank 0");
        assert_eq!(read_result, Some(0x42), "Switching to bank 0 should switch to bank 1 instead");
    }

    #[test]
    fn test_invalid_rom_read() {
        let rom = vec![[0; ROM_BANK_SIZE]; 32];
        let ram = [0; MBC2_MEM_SIZE];
        let mbc2 = init_mapper(rom, ram);

        let result = mbc2.read_rom(0x8000);

        assert!(result.is_none(), "Should return none when ROM read address is out of bounds");
    }

    #[test]
    fn test_invalid_rom_write() {
        let rom = vec![[0; ROM_BANK_SIZE]; 32];
        let ram = [0; MBC2_MEM_SIZE];
        let mut mbc2 = init_mapper(rom, ram);
        
        let result = mbc2.write_rom(0x8000, 0xFE);

        assert!(result.is_err(), "Should return error when ROM write address is out of bounds");
    }
    
    #[test]
    fn test_ram_read() {
        let rom = vec![];
        let mut ram = [0; MBC2_MEM_SIZE];
        ram[0x1FF] = 42;
        let mut mbc2 = init_mapper(rom, ram);

        let enable_result = mbc2.write_rom(0x000A, 0x0A);
        let result = mbc2.read_mem(0x1FF);
        let repeat_result = mbc2.read_mem(0x3FF);

        assert!(enable_result.is_ok(), "Should be able to enable RAM");
        assert_eq!(result, Some(42), "Should be able to read from memory");
        assert_eq!(repeat_result, Some(42), "Should repeat when reading past max address");
    }

    #[test]
    fn test_ram_disabled_read() {
        let rom = vec![];
        let ram = [0; MBC2_MEM_SIZE];
        let mbc2 = init_mapper(rom, ram);

        let result = mbc2.read_mem(0x42);

        assert_eq!(result, Some(0xFF), "Should return '0xFF' when RAM is disabled");
    }

    #[test]
    fn test_ram_write() {
        let rom = vec![];
        let ram = [0; MBC2_MEM_SIZE];
        let mut mbc2 = init_mapper(rom, ram);

        let enable_result = mbc2.write_rom(0x02FA, 0x0A);
        let write_result = mbc2.write_mem(0x42, 0x77);
        let write_repeat_result = mbc2.write_mem(0x442, 0x88);
        let written_value = mbc2.read_mem(0x42);

        assert!(enable_result.is_ok(), "Should be able to enable RAM");
        assert_eq!(write_result, Ok(0), "Should be able to write to RAM");
        assert_eq!(
            write_repeat_result, Ok(0x07), 
            "Should repeat when writing past max address, and first half-byte should be cut off"
        );
        assert_eq!(written_value, Some(0x08), "Previous call should have changed value in RAM");
    }
    
    #[test]
    fn test_ram_disabled_write() {
        let rom = vec![];
        let ram = [0; MBC2_MEM_SIZE];
        let mut mbc2 = init_mapper(rom, ram);
        
        let result = mbc2.write_mem(0xBE, 0xEF);

        assert_eq!(result, Ok(0xFF), "Should ignore writes when memory is disabled");
    }
}
