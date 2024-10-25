use crate::{GameBoySystem, GameBoySystemError};

use super::instructions::{Instruction, Operation};
use super::{CpuRegister, FlagRegister};

impl GameBoySystem {
    pub fn exec_instruction(
        &mut self, instruction: &Instruction
    ) -> Result<(), GameBoySystemError> {
        match instruction.op {
            Operation::NOP => Ok(()), 
            Operation::Load8(reg, val) => self.set_r8(reg, val),
            Operation::Load16(reg, val) => Ok(self.set_r16(reg, val)),
            Operation::Store8(addr, val) => self.store8(addr, val),
            Operation::Store16(addr, val) => self.store16(addr, val),
            Operation::Add8(val, carry) => Ok(self.add8(val, carry)),
            Operation::Add16(val) => Ok(self.add16(val)),
            Operation::Sub8(val, carry) => Ok(self.sub8(val, carry)),
            Operation::And8(val) => Ok(self.and8(val)),
            Operation::Or8(val) => Ok(self.or8(val)),
            Operation::Xor8(val) => Ok(self.xor8(val)),
            Operation::Compare8(val) => Ok(self.compare8(val)),
            Operation::Increment8(reg) => self.inc8(reg),
            Operation::Increment16(reg) => Ok(self.inc16(reg)),
            Operation::Decrement8(reg) => self.dec8(reg),
            Operation::Decrement16(reg) => Ok(self.dec16(reg)),
            _ => todo!()
        }
    }

    fn store8(&mut self, address: u16, value: u8) -> Result<(), GameBoySystemError> {
        self.memory.store_byte(address, value)
            .map_err(|_err| GameBoySystemError::MemoryWriteError(address, value as u16))?;
        Ok(())
    }

    fn store16(&mut self, address: u16, value: u16) -> Result<(), GameBoySystemError> {
        self.memory.store_half_word(address, value)
            .map_err(|_err| GameBoySystemError::MemoryWriteError(address, value))?;
        Ok(())
    }

    fn add8(&mut self, value: u8, use_carry: bool) {
        let current = self.registers.get_register(CpuRegister::A);
        let (mut result, mut carry) = current.overflowing_add(value);

        // do an add with the lower 4 bits to determine if there is a half carry
        let (_, mut half_carry) = (current << 4).overflowing_add(value << 4);

        if use_carry {
            let flags: FlagRegister = self.registers.get_register(CpuRegister::F).into();
            let carry_result = result.overflowing_add(flags.carry as u8);

            half_carry |= (result << 4).overflowing_add(flags.carry as u8).1;
            result = carry_result.0;
            carry |= carry_result.1;
        }

        // TODO - set carry flags here
        let flags = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry,
            carry
        };
        
        self.registers.set_register(CpuRegister::F, flags.into());
        self.registers.set_register(CpuRegister::A, result);
    }

    fn add16(&mut self, value: u16) {
        let current = self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L);

        let (_, half_carry) = (current << 12).overflowing_add(value << 12);
        let (result, carry) = current.overflowing_add(value);

        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let flags_result = FlagRegister {
            subtract: false,
            half_carry,
            carry,
            ..flags_current
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_joined_registers(CpuRegister::H, CpuRegister::L, result);
    }

    fn sub8(&mut self, value: u8, use_carry: bool) {
        let current = self.registers.get_register(CpuRegister::A);

        let mut half_carry = (current & 0xF) < (value & 0xF);
        let (mut result, mut carry) = current.overflowing_sub(value);

        if use_carry {
            let flags: FlagRegister = self.registers.get_register(CpuRegister::F).into();
            let carry_result = result.overflowing_sub(flags.carry as u8);

            // the only way for a half carry to happen here is if the first 4 bits are 0
            half_carry |= (result & 0xF) == 0;
            result = carry_result.0;
            carry |= carry_result.1;
        }

        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: true,
            half_carry,
            carry
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_register(CpuRegister::A, result);
    }

    // Does the same thing as sub8 but does not store the result in A
    fn compare8(&mut self, value: u8) {
        let current = self.registers.get_register(CpuRegister::A);
        let half_carry = (current & 0xF) < (value & 0xF);
        let (result, carry) = current.overflowing_sub(value);
        let flags = FlagRegister {
            zero: result == 0,
            subtract: true,
            half_carry,
            carry
        };

        self.registers.set_register(CpuRegister::F, flags.into());
    }

    fn and8(&mut self, value: u8) {
        let result = self.registers.get_register(CpuRegister::A) & value;
        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry: true,
            carry: false
        };
        
        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_register(CpuRegister::A, result);
    }

    fn or8(&mut self, value: u8) {
        let result = self.registers.get_register(CpuRegister::A) | value;
        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: false
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_register(CpuRegister::A, result);
    }

    fn xor8(&mut self, value: u8) {
        let result = self.registers.get_register(CpuRegister::A) ^ value;
        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: false
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_register(CpuRegister::A, result);
    }

    fn inc8(&mut self, register: u8) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();

        let half_carry = (current & 0x0F) == 0x0F;
        let result = current.overflowing_add(1).0;
        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry,
            ..flags_current
        };
        
        self.set_r8(register, result)?;
        self.registers.set_register(CpuRegister::F, flags_result.into());
        Ok(())
    }

    fn inc16(&mut self, register: u8) {
        let current = self.get_r16(register);
        let result = current.overflowing_add(1).0;
       
        self.set_r16(register, result);
    }

    fn dec8(&mut self, register: u8) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();

        let result = current.overflowing_sub(1).0;
        let half_carry = current & 0x0F == 0;
        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: true,
            half_carry,
            ..flags_current
        };
        
        self.set_r8(register, result);
        self.registers.set_register(CpuRegister::F, flags_result.into());
        Ok(())
    }

    fn dec16(&mut self, register: u8) {
        let current = self.get_r16(register);
        let result = current.overflowing_add(1).0;
        
        self.set_r16(register, result);
    }

    fn rotate_left(&mut self, register: u8, use_carry: bool) -> Result<(), GameBoySystemError> {
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let carry_current = flags_current.carry;
        let current = self.get_r8(register)?;

        let carry_result = (current & 0x80) != 0;
        let mut result = current << 1;

        if use_carry {
            result |= carry_current as u8;
        }
        else {
            result |= carry_result as u8;
        }

        let flags_result = FlagRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: carry_result
        };

        self.set_r8(register, result)?;
        self.registers.set_register(CpuRegister::F, flags_result.into());
        Ok(())
    }

    fn rotate_right(&mut self, register: u8, use_carry: bool) -> Result<(), GameBoySystemError> {
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let carry_current = flags_current.carry;
        let current = self.get_r8(register)?;

        let carry_result = (current & 0x80) != 0;
        let mut result = current << 1;

        if use_carry {
            result |= carry_current as u8;
        }
        else {
            result |= carry_result as u8;
        }

        let flags_result = FlagRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: carry_result
        };

        self.set_r8(register, result)?;
        self.registers.set_register(CpuRegister::F, flags_result.into());
        Ok(())
    }


}
