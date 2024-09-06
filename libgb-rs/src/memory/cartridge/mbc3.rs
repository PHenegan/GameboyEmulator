use crate::memory::cartridge::{CartridgeMapper, MemBank, ROM_BANK_SIZE, RomBank};
use crate::memory::rtc::RealTimeClock;
use crate::memory::MemoryWriteError;

pub struct MBC3 {
    rom: Vec<RomBank>,
    ram: Vec<MemBank>,
    ram_enabled: bool,
    ram_bank: u8,
    rom_bank: u8,
    rtc: RealTimeClock,
    latching: bool,
    has_battery: bool
}

impl MBC3 {
    fn write_ram(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        let byte = self.ram.get_mut(self.ram_bank as usize)
            .ok_or(MemoryWriteError)?
            .get_mut(address as usize)
            .ok_or(MemoryWriteError)?;

        let old_value = byte.clone();
        *byte = data;

        Ok(old_value)
    }
}

impl CartridgeMapper for MBC3 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let mut bank = 0;
        let mut address = address as usize;
        if address >= ROM_BANK_SIZE {
            bank = self.rom_bank as usize;
            address -= ROM_BANK_SIZE;
        }

        self.rom.get(bank)?
            .get(address)
            .copied()
    }

    fn write_rom(&mut self, address: u16, data: u8) -> Result<(), MemoryWriteError> {
        let address = address as usize;
        match address {
            0..=0x1FFF => {
                self.ram_enabled = data == 0xA0;
                Ok(())
            }
            0x2000..=0x3FFF => {
                self.rom_bank = data & 0x7F;
                Ok(())
            }
            0x4000..=0x5FFF => {
                self.ram_bank = data & 0x0F;
                Ok(())
            }
            0x6000..=0x7FFF => {
                if data == 0 {
                    self.latching = true;
                } else if data == 1 && self.latching {
                    self.rtc.latch();
                    self.latching = false;
                } else {
                    self.latching = false;
                }
                Ok(())
            }
            _ => Err(MemoryWriteError)
        }
    }

    fn read_mem(&self, address: u16) -> Option<u8> {
        if !self.ram_enabled {
            return Some(0xFF);
        }

        match self.ram_bank {
            0..=3 => self.ram.get(self.ram_bank as usize)?
                .get(address as usize)
                .copied(),
            8 => Some(self.rtc.get_seconds()),
            9 => Some(self.rtc.get_minutes()),
            0xA => Some(self.rtc.get_hours()),
            0xB => Some(self.rtc.get_days_lower()),
            0xC => Some(self.rtc.get_days_upper()),
            _ => None
        }
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        if !self.ram_enabled {
            return Ok(0xFF);
        }
        match self.ram_bank {
            0..=3 => self.write_ram(address, data),
            8 => Ok(self.rtc.set_minutes(data)),
            9 => Ok(self.rtc.set_minutes(data)),
            0xA => Ok(self.rtc.set_hours(data)),
            0xB => Ok(self.rtc.set_days_lower(data)),
            0xC => Ok(self.rtc.set_days_upper(data)),
            _ => Err(MemoryWriteError)
        }
    }
}
