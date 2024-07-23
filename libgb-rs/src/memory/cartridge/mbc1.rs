use crate::memory::MemoryWriteError;

#[derive(Debug, PartialEq, Eq)]
enum StorageMode {
    ROM = 0,
    RAM = 1,
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
    ram_enabled: bool,
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
            ram_bank: 0,
            has_battery: false,
            ram_enabled: false
        }
    }

    /// Set the lower 5 bits of the rom bank value
    fn set_lower_rom_bank(&mut self, data: u8) {
        self.rom_bank = data & 0x1F;
        // hardware bug present in MBC1 cartridges, because the 0-comparison
        // only looks at the first 5 bits
        if self.rom_bank == 0 {
            self.rom_bank += 1;
        }
   }

    /// Set the upper 2 bits of the rom bank value, or the ram bank value
    /// depending on the storage mode of the cartridge
    fn set_ram_bank(&mut self, data: u8) {
        self.ram_bank = data & 3;
    }

    fn get_mem_bank(&self) -> usize {
        if self.ram.len() == 1 || self.storage_mode == StorageMode::ROM {
            return 0;
        }
        self.ram_bank as usize
    }
}

impl CartridgeMemoryBankController for MBC1 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let first_half = address < ROM_BANK_SIZE as u16;
        let mut bank = self.rom_bank as usize;
        if first_half && self.storage_mode == StorageMode::RAM && self.rom.len() > 96 {
            // ignore the first 5 bits of the bank for 0x0000->0x3FFF
            // This is the same bug as with setting the rom bank, see set_lower_rom_bank
            bank = (self.rom_bank & 0x60) as usize;
        }
        else if first_half {
            bank = 0;
        }
        self.rom.get(bank)?
            .get(address as usize)
            .copied()
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(),MemoryWriteError> {
        // TODO - does writing to the ROM change it? I'm assuming no?
        // Ignoring the "enable" call because there are no electronic components to actually
        // enable
        match address {
            0x0 ..= 0x1FFF => {
                self.ram_enabled = (data & 0xF) == 0xA;
                Ok(())
            }
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
        if !self.ram_enabled {
            return Some(0xFF);
        }
        let bank = self.get_mem_bank();
        match self.ram.get(bank) {
            Some(&ram_bank) => ram_bank.get(address as usize).copied(),
            None => Some(0xFF)
        }
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8,MemoryWriteError> {
        if !self.ram_enabled {
            return Ok(0);
        }
        let bank = self.get_mem_bank();
        let byte = self.ram.get_mut(bank)
            .ok_or(MemoryWriteError)?
            .get_mut(address as usize)
            .ok_or(MemoryWriteError)?;
        let original = byte.clone();
        *byte = data;
        Ok(original)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_bank(rom: Vec<[u8; ROM_BANK_SIZE]>, ram: Vec<[u8; RAM_BANK_SIZE]>) -> MBC1 {
        MBC1 {
            rom,
            ram,
            storage_mode: StorageMode::ROM,
            rom_bank: 1,
            ram_bank: 0,
            has_battery: false,
            ram_enabled: false,
        }
    }

    #[test]
    fn test_storage_mode_ram_access() {
        let rom = vec!([0; ROM_BANK_SIZE]; 2);
        let mut ram = vec!([0; RAM_BANK_SIZE]; 2);
        ram[1][0x407] = 61;
        let mut bank = init_bank(rom, ram);

        // test that changing to RAM mode and accessing a different bank works
        let en_ram = bank.write_rom(0x1000, 0xA);
        assert!(en_ram.is_ok(), "check that enabling RAM succeeds");
        let en_ram_mode = bank.write_rom(0x6000, 3);
        assert!(en_ram_mode.is_ok(), "check that banking mode only reads first bit");
        let change_bank = bank.write_rom(0x4000, 5);
        assert!(change_bank.is_ok(), "check that memory bank value only uses first 2 bits");
        let value = bank.read_mem(0x407);
        assert_eq!(value, Some(61), "check that memory read retrieves correct value");
        
        let change_bank = bank.write_rom(0x4000, 0);
        assert!(change_bank.is_ok(), "check that banking mode can be switched back to 0");
        let bank0_value = bank.read_mem(0x0407);
        assert_eq!(
            bank0_value, Some(0),
            "check that memory works correctly when banking mode is 0"
        );
    }

    #[test]
    fn test_ram_access_when_1_bank() {
        let rom = vec!();
        let ram = vec!([0; RAM_BANK_SIZE]);

        let mut bank = init_bank(rom, ram);

        assert!(bank.write_rom(0x1000, 0xA).is_ok());
        assert!(bank.write_rom(0x4000, 0x2).is_ok());

        let write_result = bank.write_mem(0xF0, 40);
        assert!(write_result.is_ok());
        
        assert!(bank.write_rom(0x4000, 0x0).is_ok());
        
        let read_result = bank.read_mem(0xF0);
        assert_eq!(
            read_result, Some(40),
            "Memory should only access one bank if there is only one"
        );

    }

    #[test]
    fn test_ram_access_when_not_enabled() {
        let rom = vec!();
        let ram = vec!();
        let mut bank = init_bank(rom, ram);

        let read_result = bank.read_mem(42);
        let write_result = bank.write_mem(42, 28);
        
        assert_eq!(read_result, Some(0xFF), "Memory read should return 0xFF when RAM is disabled");
        assert_eq!(write_result, Ok(0), "Writes should be ignored when RAM is disabled");
    }

    #[test]
    fn test_read_bank_0() {
        let mut rom = vec!([0; ROM_BANK_SIZE], [0; ROM_BANK_SIZE]);
        rom[0][0x42] = 0x28;
        let ram = vec!([0; RAM_BANK_SIZE]);
        let bank = init_bank(rom, ram);

        let result = bank.read_rom(0x42);

        assert_eq!(result, Some(0x28));
    }
}
