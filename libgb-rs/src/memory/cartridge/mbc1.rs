
use std::cell::RefCell;
use crate::memory::MemoryWriteError;
use super::{bankedrom::BankedRom, CartridgeMapper, LoadCartridgeError, SaveError, ROM_BANK_SIZE};

/// # StorageMode
/// An Enum representing the banking mode of an MBC1 Cartridge. 
/// - In "ROM" Mode the the cartridge can read from up to 2 MiB of RAM (128 banks) and 8 KiB
///   of RAM (just 1 bank). However, there is a glitch preventing access of banks 0x20, 0x40, and
///   0x60 in this mode because of logic that locks the first half of the address space to bank 0.
///   Changing the cartridge's RAM bank will change the upper 2 bits of the 7-bit bank number
///
/// - In "RAM" Mode the cartridge can read from 512 KiB of ROM and 32 KiB of RAM. In cartridges
///   with more than 512 KiB, there can only be 8 KiB of RAM, but using this mode will allow
///   the first half of the address space to be switched, allowing banks 0x20, 0x40, and 0x60
///   to be accessed where bank 0 would have been.
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

/// # MBC1
/// A struct which recreates the MBC1 (Memory Bank Controller 1) cartridge functionality
/// for a DMG system.
pub struct MBC1 {
    // uses RefCell to allow mutability (for changing memory/rom banks on read calls)
    // that is required by MBC1 but not other cartridges
    rom: RefCell<BankedRom>,
    storage_mode: StorageMode,    
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    extra_storage: bool
}

impl MBC1 {
    /// Constructor for building a basic ROM cartridge
    ///
    /// Parameters:
    /// - `rom`: An array containing all of the ROM data in a single array.
    /// - `rom_banks`: the number of banks which should be created to hold the ROM
    /// - `ram_banks`: the number of banks which should be created to hold cartridge memory
    /// - `has_battery`: whether or not the cartridge supports saving data
    ///
    /// Returns:
    ///
    /// A new cartridge object, or an error if the ROM is larger than what can bet stored in
    pub fn new(
        rom: Vec<u8>, rom_banks: u8,
        ram_banks: u8, has_battery: bool
    ) -> Result<Self, LoadCartridgeError> where Self : Sized {
        let rom = BankedRom::new(rom, rom_banks as usize, ram_banks as usize, has_battery, true)?;

        Ok(
            MBC1 {
                rom: RefCell::new(rom),
                storage_mode: StorageMode::ROM,
                ram_bank: 0,
                rom_bank: 1,
                ram_enabled: false,
                extra_storage: rom_banks > 32
            }
        )
    }

   /// the given number of rom banks
 
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
        if self.storage_mode == StorageMode::ROM {
            return 0;
        }
        self.ram_bank as usize
    }
}

// TODO - worth noting that the logic for accessing ROM might still be off, I don't know if there
// is a reliable knowing how the hardware on an individual cartridge is wired up for using the
// extra 2 bit register for RAM vs. ROM
impl CartridgeMapper for MBC1 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let mut bank = self.rom_bank as usize;
        let first_half = address < (ROM_BANK_SIZE as u16);

        // The first half is mapped to 0x00, 0x20, 0x40, or 0x60 when there are enough banks
        // and the advanced banking mode is 0
        if first_half && self.storage_mode == StorageMode::RAM && self.extra_storage {
            bank = (self.ram_bank << 5) as usize;
        }
        // the first half is always bank 0 when the advanced banking mode is disabled
        else if first_half {
            bank = 0;
        }
        else if self.extra_storage {
            // account for the offset in the internal index
            bank = (self.ram_bank << 5) as usize | (bank & 0x1F);
        }

        // TODO - should I be handling the case where a bank is out of bounds or is returning
        // "None" here fine?
        let mut rom = self.rom.borrow_mut();
        rom.set_rom_bank(bank);
        rom.read_rom(address)
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(),MemoryWriteError> {
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
        let mut rom = self.rom.borrow_mut();

        rom.set_mem_bank(bank);
        rom.read_mem(address)
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8,MemoryWriteError> {
        if !self.ram_enabled {
            return Ok(0);
        }

        let bank = self.get_mem_bank();
        let mut rom = self.rom.borrow_mut();

        rom.set_mem_bank(bank);
        rom.write_mem(address, data)
    }

    fn load_save(&mut self, save_data: Vec<u8>) -> Result<(), SaveError> {
        self.rom.borrow_mut()
            .load_save(save_data)
    }

    fn save(&self) -> Vec<u8> {
        self.rom.borrow()
            .save()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cartridge::{MemBank, RomBank, RAM_BANK_SIZE};

    use super::*;

    fn init_bank(rom: Vec<RomBank>, ram: Vec<MemBank>) -> MBC1 {
        let sequential_rom = rom.concat();
        
        let result = MBC1::new(sequential_rom, rom.len() as u8, ram.len() as u8, true);
        assert!(result.is_ok(), "Should create ROM successfully");
        let mut cartridge = result.unwrap();

        let save_result = cartridge.load_save(ram.concat());
        assert!(save_result.is_ok(), "Should load ROM memory");

        cartridge
    }

    #[test]
    fn test_storage_mode_ram_access() {
        let rom = vec!([0; ROM_BANK_SIZE]; 2);
        let mut ram = vec!([0; RAM_BANK_SIZE]; 2);
        ram[1][0x407] = 61;
        let mut bank = init_bank(rom, ram);

        // test that changing to RAM mode and accessing a different bank works
        let en_ram = bank.write_rom(0x1000, 0xA);
        let en_ram_mode = bank.write_rom(0x6000, 3);
        let first_change_bank = bank.write_rom(0x4000, 5);
        let value = bank.read_mem(0x407);
        let second_change_bank = bank.write_rom(0x4000, 0);
        let bank0_value = bank.read_mem(0x0407);

        assert!(en_ram.is_ok(), "check that enabling RAM succeeds");
        assert!(en_ram_mode.is_ok(), "check that banking mode only reads first bit");
        assert!(first_change_bank.is_ok(), "check that memory bank value only uses first 2 bits");
        assert_eq!(value, Some(61), "check that memory read retrieves correct value");
        assert!(second_change_bank.is_ok(), "check that banking mode can be switched back to 0");
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

    #[test]
    fn test_read_switching_banks() {
        let mut rom = vec!([0; ROM_BANK_SIZE]; 4);
        rom[1][0x28] = 0x03;
        rom[3][0x15] = 0x62;
        let ram = vec!([0; RAM_BANK_SIZE]);
        let mut bank = init_bank(rom, ram);

        let bank_1_result = bank.read_rom(0x4028);
        assert!(bank.write_rom(0x2000, 0x3).is_ok(), "Change to ROM bank 3");
        let bank_3_result = bank.read_rom(0x4015);

        assert_eq!(bank_1_result, Some(0x03), "Test initial read");
        assert_eq!(bank_3_result, Some(0x62), "Test read after switching ROM banks");
        
    }

    #[test]
    fn test_64_rom_banks_basic_storage_mode() {
        let mut rom = vec!([0; ROM_BANK_SIZE]; 64);
        rom[0][0x95] = 0x42;
        rom[0x1][0x4] = 0x28;
        rom[0x21][0x7] = 0x63;
        let ram = vec!();
        let mut bank = init_bank(rom, ram);

        assert!(bank.write_rom(0x2000, 0).is_ok(), "set bank to 0");
        let bank_0_result = bank.read_rom(0x4004);

        assert!(bank.write_rom(0x2000, 1).is_ok(), "Change bank to 1");
        let bank_1_result = bank.read_rom(0x4004);

        assert!(bank.write_rom(0x4000, 0x1).is_ok(), "Set RAM bank to 1");
        let bank_21_result = bank.read_rom(0x4007);
        
        let first_half_result = bank.read_rom(0x95);

        assert_eq!(bank_0_result, Some(0x28), "Checking value after setting bank to 0");
        assert_eq!(bank_1_result, Some(0x28), "Checking that bank 1 value matches bank 0");
        assert_eq!(bank_21_result, Some(0x63), "Check that second half maps correctly in bank 21");
        assert_eq!(first_half_result, Some(0x42), "Check that first half still maps to bank 0");
    }

    #[test]
    fn test_64_rom_banks_advanced_storage_mode() {
        let mut rom = vec!([0; ROM_BANK_SIZE]; 64);
        rom[0x20][0x20] = 0x19;
        let ram = vec!();
        let mut bank = init_bank(rom, ram);

        assert!(bank.write_rom(0x2000, 1).is_ok());
        assert!(bank.write_rom(0x4000, 1).is_ok());
        assert!(
            bank.write_rom(0x6000, 1).is_ok(),
            "Checking that storage mode is changed successfully"
        );
        let result = bank.read_rom(0x20);

        assert_eq!(result, Some(0x19), "Check that bank 0 switches in advanced storage mode");
    }

    #[test]
    fn test_4_rom_banks_advanced_storage_mode() {
        let mut rom = vec!([0; ROM_BANK_SIZE]; 4);
        rom[0][0x4] = 0x28;
        rom[3][0x7] = 0x63;
        let ram = vec!();
        let mut bank = init_bank(rom, ram);

        assert!(bank.write_rom(0x6000, 0x1).is_ok(), "Change into advanced banking mode");
        assert!(bank.write_rom(0x2000, 0x3).is_ok(), "Change ROM banks");
        assert!(
            bank.write_rom(0x4000, 0x11).is_ok(),
            "Change bank even though there are not enough ROM or RAM banks"
        );
        
        let first_half_result = bank.read_rom(0x4);
        let second_half_reuslt = bank.read_rom(0x4007);

        assert_eq!(first_half_result, Some(0x28), "Check read result from first half of addresses");
        assert_eq!(
            second_half_reuslt, Some(0x63),
            "Check read result from second half of addresses"
        );
    }
}
