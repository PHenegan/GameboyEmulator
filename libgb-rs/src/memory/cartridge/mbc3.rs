use crate::memory::cartridge::{CartridgeMapper, MemBank, ROM_BANK_SIZE, RomBank};
use crate::memory::rtc::RealTimeClock;
use crate::memory::MemoryWriteError;

pub struct MBC3 {
    rom: Vec<RomBank>,
    ram: Vec<MemBank>,
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    rtc: Option<RealTimeClock>,
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
                    self.rtc.as_mut()
                        .ok_or(MemoryWriteError)?
                        .latch();
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
            8 => Some(self.rtc.as_ref()?.get_seconds()),
            9 => Some(self.rtc.as_ref()?.get_minutes()),
            0xA => Some(self.rtc.as_ref()?.get_hours()),
            0xB => Some(self.rtc.as_ref()?.get_days_lower()),
            0xC => Some(self.rtc.as_ref()?.get_days_upper()),
            _ => None
        }
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        if !self.ram_enabled {
            return Ok(0xFF);
        }
        match self.ram_bank {
            0..=3 => self.write_ram(address, data),
            8 => Ok(self.rtc.as_mut().ok_or(MemoryWriteError)?.set_seconds(data)),
            9 => Ok(self.rtc.as_mut().ok_or(MemoryWriteError)?.set_minutes(data)),
            0xA => Ok(self.rtc.as_mut().ok_or(MemoryWriteError)?.set_hours(data)),
            0xB => Ok(self.rtc.as_mut().ok_or(MemoryWriteError)?.set_days_lower(data)),
            0xC => Ok(self.rtc.as_mut().ok_or(MemoryWriteError)?.set_days_upper(data)),
            _ => Err(MemoryWriteError)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cartridge::RAM_BANK_SIZE;

    use super::*;

    fn init_mapper(rom: Vec<RomBank>, ram: Vec<MemBank>, rtc: Option<RealTimeClock>) -> MBC3 {
        MBC3 {
            rom,
            ram,
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            rtc,
            latching: false,
            has_battery: false
        }
    }

    #[test]
    fn test_read_rom_bank_0() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 16];
        rom[0][0x42] = 28;
        let mapper = init_mapper(rom, Vec::new(), None);

        let result = mapper.read_rom(0x42);

        assert_eq!(result, Some(28), "Should read correctly from bank 0");
    }

    #[test]
    fn test_read_rom_bank_0_after_switch() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 64];
        rom[0][0x42] = 28;
        let mut mapper = init_mapper(rom, Vec::new(), None);

        let switch_result = mapper.write_rom(0x3000, 0x20);
        let read_result = mapper.read_rom(0x42);
        
        assert!(switch_result.is_ok(), "Should successfully switch banks");
        assert_eq!(read_result, Some(28), "Should read correctly from bank 0");
    }

    #[test]
    fn test_read_rom_after_bank_switch() {
        let mut rom = vec![[0; ROM_BANK_SIZE]; 64];
        rom[5][0x280] = 28;
        let mut mapper = init_mapper(rom, Vec::new(), None);

        let switch_result = mapper.write_rom(0x3000, 5);
        let read_result = mapper.read_rom(0x4280);

        assert!(switch_result.is_ok(), "Should successfully switch banks");
        assert_eq!(read_result, Some(28), "Should read correctly from switched bank");
    }

    #[test]
    fn test_read_rom_invalid_address() {
        let rom = vec![[0; ROM_BANK_SIZE]; 16];
        let mapper = init_mapper(rom, Vec::new(), None);

        let result = mapper.read_rom(0x8000);

        assert!(result.is_none(), "Should not read invalid address");
    }

    #[test]
    fn test_rom_write_invalid_address() {
        let rom = vec![[0; ROM_BANK_SIZE]; 16];
        let mut mapper = init_mapper(rom, Vec::new(), None);

        let result = mapper.write_rom(0x8000, 0xFF);

        assert!(result.is_err(), "Should not write to invalid address");
    }

    #[test]
    fn test_read_ram_bank_0() {
        let rom = vec![[0; ROM_BANK_SIZE]; 16];
        let mut ram = vec![[0; RAM_BANK_SIZE]; 4];
        ram[0][0x0315] = 62;
        let mut mapper = init_mapper(rom, ram, None);

        let enable_result = mapper.write_rom(0x1000, 0xA0);
        let read_result = mapper.read_mem(0x0315);

        assert!(enable_result.is_ok(), "Should enable RAM successfully");
        assert_eq!(read_result, Some(62), "Should read from RAM bank 0 successfully");
    }

    #[test]
    fn test_read_ram_banks() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let mut ram = vec![[0; RAM_BANK_SIZE]; 4];
        let changed_values: Vec<(u16, u8)> = vec![(0x789, 42), (0x456, 43), (0x123, 44)];
        ram[1][0x789] = 42;
        ram[2][0x456] = 43;
        ram[3][0x123] = 44;
        let mut mapper = init_mapper(rom, ram, None);

        let _ = mapper.write_rom(0x1000, 0xA0);

        for i in 1..4 {
            let switch_result = mapper.write_rom(0x5000, i);
            let read_result = mapper.read_mem(changed_values[(i - 1) as usize].0);

            assert!(switch_result.is_ok(), "Should successfully switch to bank {i}");
            assert_eq!(
                read_result, Some(changed_values[(i - 1) as usize].1), 
                "Should successfully switch to bank {i}"
            );
        }
    }

    #[test]
    fn test_read_ram_rtc() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 4];
        let rtc = RealTimeClock::new(Some(1), Some(2), Some(3), Some(4), Some(5));
        let mut mapper = init_mapper(rom, ram, Some(rtc));

        assert!(mapper.write_rom(0x1000, 0xA0).is_ok());

        assert!(mapper.write_rom(0x5000, 8).is_ok());
        assert_eq!(mapper.read_mem(0x0), Some(1), "Check seconds register");
        assert!(mapper.write_rom(0x5000, 9).is_ok());
        assert_eq!(mapper.read_mem(0x0), Some(2), "Check minutes register");
        assert!(mapper.write_rom(0x5000, 0xA).is_ok());
        assert_eq!(mapper.read_mem(0x0), Some(3), "Check hours register");
        assert!(mapper.write_rom(0x5000, 0xB).is_ok());
        assert_eq!(mapper.read_mem(0x0), Some(4), "Check lower days register");
        assert!(mapper.write_rom(0x5000, 0xC).is_ok());
        assert_eq!(mapper.read_mem(0x0), Some(1), "Check upper days register");
    }

    #[test]
    fn test_read_ram_invalid_address() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 1];
        let mut mapper = init_mapper(rom, ram, None);

        let enable_result = mapper.write_rom(0x1000, 0xA0);
        let result = mapper.read_mem(0x2000);

        assert!(enable_result.is_ok(), "Should be able to enable RAM");
        assert!(result.is_none(), "Should not read invalid address");
    }

    #[test]
    fn test_read_ram_disabled() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 1];
        let mapper = init_mapper(rom, ram, None);

        let result = mapper.read_mem(0x1000);

        assert_eq!(result, Some(0xFF));
    }

    #[test]
    fn test_write_ram_bank_0() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let mut ram = vec![[0; RAM_BANK_SIZE]; 1];
        ram[0][0x123] = 6;
        let mut mapper = init_mapper(rom, ram, None);
        
        let enable_result = mapper.write_rom(0x1234, 0xA0);
        let write_result = mapper.write_mem(0x0123, 5);
        let value_written = mapper.read_mem(0x123);

        assert!(enable_result.is_ok());
        assert_eq!(write_result, Ok(6));
        assert_eq!(value_written, Some(5));
    }

    #[test]
    fn test_write_ram_banks() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 4];
        let mut mapper = init_mapper(rom, ram, None);
        
        assert!(mapper.write_rom(0x0, 0xA0).is_ok());

        for i in 1..4 {
            assert!(mapper.write_rom(0x4040, i).is_ok(), "Should switch to bank {i}");
            let write_result = mapper.write_mem(0x42, 0x50);
            let value_written = mapper.read_mem(0x42);

            assert_eq!(
                write_result, Ok(0),
                "Should always overwrite 0 (indicates switching to a new bank)"
            );
            assert_eq!(value_written, Some(0x50));
        }
    }

    #[test]
    fn test_write_ram_rtc_banks() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 4];
        let rtc = RealTimeClock::new(None, None, None, None, Some(0x40));
        let mut mapper = init_mapper(rom, ram, Some(rtc));
        
        assert!(mapper.write_rom(0x0500, 0xA0).is_ok());

        assert!(mapper.write_rom(0x5FFF, 8).is_ok());
        assert_eq!(mapper.write_mem(0, 5), Ok(0), "Write to seconds register");
        assert_eq!(mapper.read_mem(0), Some(5), "Check seconds value");
        assert!(mapper.write_rom(0x50FF, 9).is_ok());
        assert_eq!(mapper.write_mem(0, 5), Ok(0), "Write to minutes register");
        assert_eq!(mapper.read_mem(0), Some(5), "Check minutes value");
        assert!(mapper.write_rom(0x5F0F, 0xA).is_ok());
        assert_eq!(mapper.write_mem(0, 5), Ok(0), "Write to hours register");
        assert_eq!(mapper.read_mem(0), Some(5), "Check hours value");
        assert!(mapper.write_rom(0x5FF0, 0xB).is_ok());
        assert_eq!(mapper.write_mem(0, 5), Ok(0), "Write to lower day register");
        assert_eq!(mapper.read_mem(0), Some(5), "Check lower day value");
        assert!(mapper.write_rom(0x5FF0, 0xC).is_ok());
        assert_eq!(mapper.write_mem(0, 0x41), Ok(0x40), "Write to upper day register");
        assert_eq!(mapper.read_mem(0), Some(0x41), "Check upper day value");
    }

    #[test]
    fn test_write_ram_disabled() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 1];
        let mut mapper = init_mapper(rom, ram, None);

        let result = mapper.write_mem(0x420, 42);
        assert!(mapper.write_rom(0, 0xA0).is_ok());
        let check_result = mapper.read_mem(0x420);

        assert_eq!(result, Ok(0xFF), "Writing when disabled should do nothing");
        assert_eq!(check_result, Some(0), "Nothing should be present in write address");
    }

    #[test]
    fn test_write_ram_invalid_address() {
        let rom = vec![[0; ROM_BANK_SIZE]; 2];
        let ram = vec![[0; RAM_BANK_SIZE]; 1];
        let mut mapper = init_mapper(rom, ram, None);

        assert!(mapper.write_rom(0x0001, 0xA0).is_ok());
        let result = mapper.write_mem(0x2000, 42);

        assert!(result.is_err(), "Should not be able to write to an invalid address");
    }
}
