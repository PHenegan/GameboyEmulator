use std::cell::RefCell;
use std::ops::AddAssign;
use std::time::{Duration, Instant};
use crate::memory::cartridge::{CartridgeMapper, MemBank, ROM_BANK_SIZE, RomBank};
use crate::memory::MemoryWriteError;

const SET_ROM_BANK_START: usize = 0x2000;
const SET_RAM_BANK_START: usize = 0x4000;
const LATCH_CLOCK_START: usize = 0x6000;
const ROM_END: usize = 0x8000;

pub struct MBC3 {
    rom: Vec<RomBank>,
    ram: Vec<MemBank>,
    ram_enabled: bool,
    ram_bank: u8,
    rom_bank: u8,
    last_access: RefCell<Instant>,
    time_register: RefCell<Duration>,
    latching: bool,
    halted: bool,
    has_battery: bool
}
impl MBC3 {
    fn update_time(&self) {
        if self.halted {
            return;
        }
        let elapsed = self.last_access.borrow()
            .elapsed();
        self.time_register.borrow_mut()
            .add_assign(elapsed);
        self.last_access.replace(Instant::now());
    }

    fn seconds(&self) -> u8 {
        self.update_time();
        let seconds = (self.time_register.borrow().as_secs()) % 60;

        seconds as u8
    }

    fn minutes(&self) -> u8 {
        self.update_time();
        let minutes = (self.time_register.borrow().as_secs() / 60) % 60;

        minutes as u8
    }

    fn hours(&self) -> u8 {
        self.update_time();
        let hours = (self.time_register.borrow().as_secs() / 3600) % 24;

        hours as u8
    }

    fn days(&self) -> u16 {
        self.update_time();
        let days = self.time_register.borrow().as_secs() / 86400;

        (days & 0x1FF) as u16
    }
}
impl CartridgeMapper for MBC3 {
    fn read_rom(&self, address: u16) -> Option<u8> {
        let mut bank = 0;
        let mut address = address as usize;
        if address >= ROM_BANK_SIZE {
            bank = self.rom_bank;
            address -= ROM_BANK_SIZE;
        }

        self.rom.get(self.rom_bank as usize)?
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
                    self.last_access.replace(Instant::now());
                    self.time_register.replace(Duration::new(0, 0));
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
        match self.ram_bank {
            0..=3 => self.ram.get(self.ram_bank as usize)?
                .get(address as usize)
                .copied(),
            8 => Some(self.seconds()),
            9 => Some(self.minutes()),
            0xA => Some(self.hours()),
            0xB => Some(self.days() as u8),
            0xC => {
                todo!()
            },
            _ => None
        }
    }

    fn write_mem(&mut self, address: u16, data: u8) -> Result<u8, MemoryWriteError> {
        todo!()
    }
}