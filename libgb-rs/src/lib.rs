pub mod cpu;
pub mod memory;

use cpu::{CpuData, CpuRegister};
use memory::MemoryController;

mod utils;

#[derive(Debug)]
pub enum GameBoySystemError {
    MemoryReadError(u16), // the address at which a read was attempted
    MemoryWriteError(u16, u16), // The address at which a write was attempted, and the write value
    InvalidInstructionError(u8) // The invalid binary instruction
}

pub struct GameBoySystem {
    registers: CpuData,
    memory: Box<dyn MemoryController>,
    // PPU will also need to go here eventually
}

impl GameBoySystem {
    pub fn new(memory: Box<dyn MemoryController>) -> Self {
        Self {
            registers: CpuData::new(),
            memory
        }
    }

    fn fetch_byte(&mut self) -> Result<u8, GameBoySystemError> {
        let byte = self.memory.load_byte(self.registers.pc)
            .ok_or(GameBoySystemError::MemoryReadError(self.registers.pc))?;
        self.registers.pc += 1;

        Ok(byte)
    }

    fn fetch_imm16(&mut self) -> Result<u16, GameBoySystemError> {
        let half_word = self.memory.load_half_word(self.registers.pc)
            .ok_or(GameBoySystemError::MemoryReadError(self.registers.pc))?;
        self.registers.pc += 2;
        Ok(half_word)
    }

    fn get_r8(&self, reg: u8) -> Result<u8, GameBoySystemError> {
        if reg == 6 {
            let addr = self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L);
            return self.memory.load_byte(addr)
                .ok_or(GameBoySystemError::MemoryReadError(addr));
        }

        Ok(self.registers.get_register(reg.into()))
    }

    fn set_r8(&mut self, reg: u8, value: u8) -> Result<(), GameBoySystemError> {
        if reg == 6 {
            let address = self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L);
            self.memory.store_byte(address, value)
                .map_err(|_err| GameBoySystemError::MemoryWriteError(address, value as u16))?;
            return Ok(());
        }

        Ok(self.registers.set_register(reg.into(), value))
    }

    fn get_r16(&mut self, register: u8) -> u16 {
        match register {
            0 => self.registers.get_joined_registers(CpuRegister::B, CpuRegister::C),
            1 => self.registers.get_joined_registers(CpuRegister::D, CpuRegister::E),
            2 => self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L),
            3 => self.registers.sp,
            _ => panic!("Invalid r16 address - value {register} greater than 4 passed to get_r16")
        }
    }

    fn get_r16_mem(&mut self, register: u8) -> u16 {
        match register {
            0 => self.registers.get_joined_registers(CpuRegister::B, CpuRegister::C),
            1 => self.registers.get_joined_registers(CpuRegister::D, CpuRegister::E),
            2 => {
                let value = self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L);
                self.registers.set_joined_registers(
                    CpuRegister::H, CpuRegister::L, value.overflowing_add(1).0
                );
                value
            },
            3 => {
                let value = self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L);
                self.registers.set_joined_registers(
                    CpuRegister::H, CpuRegister::L, value.overflowing_sub(1).0
                );
                value
            },
            _ => panic!("Invalid r16mem address - value greater than 4 passed in")
        }
    }
}
