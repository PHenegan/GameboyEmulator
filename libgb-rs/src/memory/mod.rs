use cartridge::CartridgeMemoryBankController;
use mockall::automock;

use crate::utils::{Merge, Split};

pub mod cartridge;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct MemoryWriteError;

/// A Trait representing the functionality needed for interacting with a Game Boy system's
/// memory
#[automock]
pub trait MemoryController {
    /// Retrieve a byte from the given address in memory
    ///
    /// `address`: the location in memory to retrieve a byte from.
    ///
    /// Returns the byte of memory, or `None` if the address does not exist
    fn load_byte(&self, address: u16) -> Option<u8>;

    /// Load a 16-bit number from the given address in memory
    ///
    /// `address`: the location in memory to retrieve two successive bytes from.
    ///
    /// Returns the 16-bit number retrieved from memory, or `None` if either byte of the number
    /// is located at an invalid address.
    fn load_half_word(&self, address: u16) -> Option<u16>;

    /// Save a byte into the given location in memory
    ///
    /// `address`: the location in memory to save to
    /// `data`: the 8-bit number being saved into memory
    ///
    /// Returns the byte which was previously in that location of memory, or a MemoryWriteError
    /// if the address is invalid.
    fn store_byte(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError>;

    /// Save a 16-bit number into the given location in memory
    ///
    /// `address`: the location in memory to save to
    /// `data`: the 16-bit number being saved into memory
    ///
    /// If either byte in the 16-bit number occurs at an invalid location in memory,
    /// a MemoryWriteError with be returned.
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

/// A Struct Storing the memory of an original Game Boy (DMG) system
pub struct DmgMemoryController {
    cartridge: Box<dyn CartridgeMemoryBankController>,
    ram: [u8; DMG_RAM_SIZE],
    vram: [u8; DMG_VRAM_SIZE],
    system: [u8; DMG_RES_SIZE],
}

impl DmgMemoryController {
    pub fn new(cartridge: Box<dyn CartridgeMemoryBankController>) -> DmgMemoryController {
        DmgMemoryController {
            cartridge,
            ram: [0; DMG_VRAM_SIZE],
            vram: [0; DMG_VRAM_SIZE],
            system: [0; DMG_RES_SIZE],
        }
    }
}

impl MemoryController for DmgMemoryController {
    fn load_byte(&self, address: u16) -> Option<u8> {
        match address {
            0..=DMG_ROM_END => {
                self.cartridge.read_rom(address)
            }
            DMG_EXT_START..=DMG_EXT_END => {
                self.cartridge.read_mem(address - DMG_EXT_START)
            }
            DMG_VRAM_START..=DMG_VRAM_END => {
                Some(self.vram[(address - DMG_VRAM_START) as usize])
            }
            DMG_RAM_START..=DMG_RAM_END => {
                Some(self.ram[(address - DMG_RAM_START) as usize])
            }
            DMG_RES_START..=DMG_RES_END => {
                Some(self.system[(address - DMG_RES_START) as usize])
            }
            _ => None
        }
    }

    fn load_half_word(&self, address: u16) -> Option<u16> {
        let left = self.load_byte(address)?;
        let right = self.load_byte(address + 1)?;

        Some(left.merge(right))
    }

    fn store_byte(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        match address {
            0..=DMG_ROM_END => {
                self.cartridge.write_rom(address, data)
                    .map(|_| data)
            }
            DMG_VRAM_START..=DMG_VRAM_END => {
                let address = (address - DMG_VRAM_START) as usize;
                let prev = self.vram[address];
                self.vram[address] = data;
                Ok(prev)
            }
            DMG_EXT_START..=DMG_EXT_END => {
                self.cartridge.write_mem(address - DMG_EXT_START, data)
            }
            DMG_RAM_START..=DMG_RAM_END => {
                let address = (address - DMG_RAM_START) as usize;
                let prev = self.vram[address];
                self.ram[address] = data;
                Ok(prev)
            }
            DMG_RES_START..=DMG_RES_END => {
                let address = (address - DMG_RES_START) as usize;
                let prev = self.vram[address];
                self.system[address] = data;
                Ok(prev)
            }
            _ => Err(MemoryWriteError)
        }
    }

    fn store_half_word(&mut self, address: u16, data: u16) -> Result<(), MemoryWriteError> {
        let (left_data, right_data) = data.split();

        let prev_left = self.store_byte(address, left_data)?;
        let right = self.store_byte(address + 1, right_data);
        if right.is_err() {
            self.store_byte(address, prev_left).unwrap();
            return Err(MemoryWriteError);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use mockall::predicate::eq;
    use crate::memory::cartridge::MockCartridgeMemoryBankController;
    use super::*;

    #[test]
    fn test_rom_write_fails() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_write_rom()
            .return_const(Err(MemoryWriteError));
        let mut controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.store_byte(42, 42);

        assert!(result.is_err(), "Test that writing to a rom does not work");
    }

    #[test]
    fn test_rom_read() {
        // NOTE - I couldn't figure out how to put in a mock Option<&u8>
        // into this without doing some jank static lifetime stuff so I'm having it
        // return None and checking that the address gets passed correctly
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_read_rom()
            .times(1)
            .with(eq(42))
            .return_const(Some(210));
        let controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.load_byte(42);

        assert_eq!(result, Some(210), "Test reading from a ROM address");
    }

    #[test]
    fn test_vram_io() {
        let mock = MockCartridgeMemoryBankController::new();
        let mut controller = DmgMemoryController::new(Box::new(mock));

        assert_eq!(controller.load_byte(0x8000), Some(0));

        let result = controller.store_byte(0x8000, 80);

        assert_eq!(result, Ok(0), "Test writing to VRAM");
        assert_eq!(controller.load_byte(0x8000), Some(80), "Test changed RAM value");
    }

    #[test]
    fn test_cart_ram_read_success() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_read_mem()
            .with(eq(42))
            .return_const(Some(0x22));
        let controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.load_byte(DMG_EXT_START + 42);

        assert_eq!(result, Some(0x22), "Test reading from cartridge RAM");
    }

    #[test]
    fn test_cart_ram_read_fail() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_read_mem()
            .with(eq(42))
            .return_const(None);
        let controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.load_byte(DMG_EXT_START + 42);

        assert!(result.is_none(), "Test reading cartridge RAM when it doesn't exist");
    }

    #[test]
    fn test_cart_ram_write_fail() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_write_mem()
            .with(eq(42), eq(42))
            .return_const(Err(MemoryWriteError));
        let mut controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.store_byte(DMG_EXT_START + 42, 42);

        assert!(result.is_err(), "Test that accessing non-existent cartridge RAM errors");
    }

    #[test]
    fn test_ram_io() {
        let mock = MockCartridgeMemoryBankController::new();
        let mut controller = DmgMemoryController::new(Box::new(mock));

        assert_eq!(controller.load_byte(0xC042), Some(0));

        let result = controller.store_byte(0xC042, 28);

        assert_eq!(result, Ok(0), "Test writing to system RAM");
        assert_eq!(controller.load_byte(0xC042), Some(28), "Test changed RAM value");
    }

    #[test]
    fn test_reserved_io() {
        let mock = MockCartridgeMemoryBankController::new();
        let mut controller = DmgMemoryController::new(Box::new(mock));

        assert_eq!(controller.load_byte(0xFE42), Some(0));

        let result = controller.store_byte(0xFE42, 7);

        assert_eq!(result, Ok(0), "Test writing to reserved addresses");
        assert_eq!(controller.load_byte(0xFE42), Some(7), "Test changed RAM value");
    }

    #[test]
    fn test_load_half_word_valid_address() {
        let mock = MockCartridgeMemoryBankController::new();
        let mut controller = DmgMemoryController::new(Box::new(mock));
        controller.store_byte(DMG_RAM_START, 0x04).unwrap();
        controller.store_byte(DMG_RAM_START + 1, 0x28).unwrap();

        let result = controller.load_half_word(DMG_RAM_START);

        assert_eq!(result, Some(0x0428), "Test valid 16-bit load");
    }

    #[test]
    fn test_load_half_word_invalid_first_byte() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_read_mem()
            .with(eq(0x1FFF))
            .return_const(None);
        let controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.load_half_word(0xBFFF);

        assert!(result.is_none(), "Test loading address where 1st byte is an invalid address")
    }

    #[test]
    fn test_load_half_word_invalid_second_byte() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_read_mem()
            .with(eq(0))
            .return_const(None);
        let controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.load_half_word(0x9FFF);

        assert!(result.is_none(), "Test loading address where 2nd byte is an invalid address");
    }

    #[test]
    fn test_store_half_word_valid_address() {
        let mock = MockCartridgeMemoryBankController::new();
        let mut controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.store_half_word(DMG_RAM_START, 0x0428);

        assert_eq!(result, Ok(()), "Test storing 2 bytes into a valid address");
        assert_eq!(controller.load_byte(DMG_RAM_START), Some(0x04), "Test first loaded byte");
        assert_eq!(controller.load_byte(DMG_RAM_START + 1), Some(0x28), "Test second loaded byte");
    }

    #[test]
    fn test_store_half_byte_invalid_first_byte() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_write_rom()
            .with(eq(DMG_ROM_END), eq(0x08))
            .return_const(Err(MemoryWriteError));
        let mut controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.store_half_word(DMG_ROM_END, 0x0812);

        assert!(result.is_err(), "Test that the invalid write failed");
        assert_eq!(
            controller.load_byte(DMG_VRAM_START), Some(0),
            "Test that the valid address is unchanged"
        );
    }

    #[test]
    fn test_store_half_byte_invalid_second_byte() {
        let mut mock = MockCartridgeMemoryBankController::new();
        mock.expect_write_mem()
            .with(eq(0), eq(0x06))
            .return_const(Err(MemoryWriteError));
        let mut controller = DmgMemoryController::new(Box::new(mock));

        let result = controller.store_half_word(DMG_VRAM_END, 0x0106);

        assert_eq!(result, Err(MemoryWriteError), "Test that the invalid write failed");
        assert_eq!(
            controller.load_byte(DMG_VRAM_END), Some(0),
            "Test that the valid address is unchanged"
        );
    }
}
