use crate::memory::{cartridge::{CartridgeMapper, LoadCartridgeError, RomOnlyCartridge, MBC1, MBC2, MBC3}, rtc::RealTimeClock};

impl TryFrom<Vec<u8>> for Box<dyn CartridgeMapper> {
    type Error = LoadCartridgeError;

    fn try_from(rom: Vec<u8>) -> Result<Self, Self::Error> {
        let cartridge_type = rom.get(0x147)
            .ok_or(LoadCartridgeError::InvalidRomFile)?;
        let rom_size = rom.get(0x148)
            .ok_or(LoadCartridgeError::InvalidRomFile)?;
        let ram_size = rom.get(0x148)
            .ok_or(LoadCartridgeError::InvalidRomFile)?;
        let rom_banks = 2 << rom_size;
        let mem_banks = match ram_size {
            0 => 0,
            1 ..= 2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => return Err(LoadCartridgeError::InvalidRomFile)
        };
        match cartridge_type {
            0x00 => Ok(Box::new(RomOnlyCartridge::new(rom, false, false)?)),
            0x08 => Ok(Box::new(RomOnlyCartridge::new(rom, true, false)?)),
            0x09 => Ok(Box::new(RomOnlyCartridge::new(rom, true, true)?)),
            0x01 => Ok(Box::new(MBC1::new(rom, rom_banks, 0, false)?)),
            0x02 => Ok(Box::new(MBC1::new(rom, rom_banks, mem_banks, false)?)),
            0x03 => Ok(Box::new(MBC1::new(rom, rom_banks, mem_banks, true)?)),
            0x05 => Ok(Box::new(MBC2::new(rom, rom_banks, false)?)),
            0x06 => Ok(Box::new(MBC2::new(rom, rom_banks, true)?)),
            0x0F => Ok(
                Box::new(MBC3::new(rom, rom_banks, 0, true, Some(RealTimeClock::default()))?)
            ),
            0x10 => Ok(
                Box::new(
                    MBC3::new(rom, rom_banks, mem_banks, true, Some(RealTimeClock::default()))?
                )
            ),
            0x11 => Ok(Box::new(MBC3::new(rom, rom_banks, 0, false, None)?)),
            0x12 => Ok(Box::new(MBC3::new(rom, rom_banks, mem_banks, false, None)?)),
            0x13 => Ok(Box::new(MBC3::new(rom, rom_banks, mem_banks, true, None)?)),

            _ => Err(LoadCartridgeError::UnsupportedType)
        }
    }
}
