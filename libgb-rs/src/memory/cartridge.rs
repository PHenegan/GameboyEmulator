use mockall::automock;
use crate::memory::MemoryWriteError;

mod basicrom;
mod mbc1;
mod mbc2;
mod mbc3;
mod bankedrom;
mod builder;

pub use basicrom::RomOnlyCartridge;
pub use mbc1::MBC1;
pub use mbc2::MBC2;
pub use mbc3::MBC3;

const ROM_BANK_SIZE: usize = 16384;
const RAM_BANK_SIZE: usize = 8192;

pub type RomBank = [u8; ROM_BANK_SIZE];
pub type MemBank = [u8; RAM_BANK_SIZE];

#[derive(Debug)]
pub enum LoadCartridgeError {
    UnsupportedType,
    InvalidRomFile
}

#[derive(Debug)]
pub enum SaveError {
    SavesNotSupported,
    SaveFileTooBig,
}

/// # CartridgeMapper
/// A Trait representing A Game boy system's cartridge memory mapper. This trait is necessary
/// to accomodate the different types of Game boy cartridges which allow for increased memory
/// and ROM storage in several slightly different ways.
#[automock]
pub trait CartridgeMapper {
    /// Get the 8-bit number at the given address on the cartridge ROM
    ///
    /// Parameters:
    /// - `address`: the ROM address to read from, indexed between 0 and 32,767
    ///
    /// Returns the byte at the given address in the ROM, or None if the address is not valid
    fn read_rom(&self, address: u16) -> Option<u8>;

    /// Sends a write command to an address in the ROM of a cartridge
    ///
    /// NOTE - This should never actually modify the contents of the ROM. Certain Memory Bank
    /// Controllers use writes to a ROM address to switch between memory banks, allowing for more
    /// than 32 KiB of RAM and more than 8 KiB of ROM.
    ///
    /// Parameters:
    /// - `address`: the ROM address to write to, indexed between 0 and 32,767
    /// - `data`: the value to store in RAM
    ///
    /// Returns a MemoryWriteError if the address is not in the valid range
    fn write_rom(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError>;

    /// Get the 8-bit number at the given address on the cartridge RAM
    ///
    /// Parameters:
    /// - `address`: the RAM address to read from, indexed between 0 and 8,191
    ///
    /// Returns the number retrieved from RAM
    fn read_mem(&self, address: u16) -> Option<u8>;

    /// Write the given byte into this cartridge's RAM at the given location,
    /// returning the value that was overwritten
    ///
    /// Parameters:
    /// - `address`: the RAM address to get for writing, between 0 and 8,191
    /// - `data`: the value to store in RAM
    ///
    /// Returns the value of the byte that was previously in the given location in RAM,
    /// or a MemoryWriteError if the address is not in the valid range
    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError>;

    /// Returns whether or not this cartridge supports saving
    fn can_save(&self) -> bool;

    /// Load a save file into the cartridge's memory
    ///
    /// Parameters:
    /// - `save_data`: the memory to load, as a vector of bytes
    ///
    /// Returns:
    ///
    /// () When the function completes successfully, or a SaveError when saving is not supported
    /// or the save being loaded is too large
    fn load_save(&mut self, save_data: Vec<u8>) -> Result<(), SaveError>;

    /// Dump a cartridge's memory as a vector of bytes.
    fn save(&self) -> Vec<u8>;
}
