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
            // carry args: register, whether to rotate, carry, (right only - arithmetic op flag)
            Operation::RotateLeft(reg, carry, zero) => self.rotate_left(reg, carry, zero),
            Operation::RotateRight(reg, carry, zero) => self.rotate_right(reg, carry, zero),
            Operation::ShiftLeftArithmetic(reg) => self.shift_left(reg),
            Operation::ShiftRightArithmetic(reg) => self.shift_right(reg, true),
            Operation::ShiftRightLogical(reg) => self.shift_right(reg, false),
            Operation::SwapBits(reg) => self.swap_bits(reg),
            Operation::DAA => self.daa(),
            Operation::Complement => Ok(self.complement()),
            Operation::SetCarryFlag => Ok(self.set_carry()),
            Operation::ComplementCarryFlag => Ok(self.complement_carry()),
            Operation::Jump(addr) => Ok(self.jump(addr)),
            Operation::Call(addr) => self.call(addr),
            Operation::Return(enable_interrupts) => self.ret(enable_interrupts),
            Operation::TestBit(reg, bit) => self.test_bit(reg, bit),
            Operation::SetBit(reg, bit) => self.set_bit(reg, bit),
            Operation::ResetBit(reg, bit) => self.reset_bit(reg, bit),
            Operation::PopStack(reg) => self.pop_stack(reg),
            Operation::PushStack(reg) => self.push_stack(reg),
            Operation::AddStackPointer(val) => Ok(self.add_stack_pointer(val)),
            Operation::SetInterrupts(enabled) => Ok(self.set_interrupts(enabled)),
            Operation::Stop => todo!(),
            Operation::Halt => todo!(),
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
        
        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.set_r8(register, result)
    }

    fn dec16(&mut self, register: u8) {
        let current = self.get_r16(register);
        let result = current.overflowing_add(1).0;
        
        self.set_r16(register, result);
    }

    fn shift_left(&mut self, register: u8) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;

        let carry_result = (current & 0x80) != 0;
        let result = current << 1;

        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: carry_result
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.set_r8(register, result)
    }

    fn shift_right(
        &mut self, register: u8, arithmetic: bool
    ) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;

        // get the last bit for arithmetic shifts, otherwise fill with 0
        let extend_bit = current & 0x80 & ((arithmetic as u8) << 7);
        let carry_result = (current & 0x01) != 0;
        let result = current >> 1 | extend_bit;

        let flags_result = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: carry_result
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.set_r8(register, result)
    }

    fn rotate_left(
        &mut self, register: u8, use_carry_result: bool, check_zero: bool
    ) -> Result<(), GameBoySystemError> {
        // TODO - check if the carry logic is correct
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let current = self.get_r8(register)?;

        let carry_result = (current & 0x80) != 0;
        // if use cary is true, rotate the carry flag in, otherwise use the leftmost bit
        let new_bit =
            (use_carry_result && carry_result) || (!use_carry_result && flags_current.carry);
        let result = (current << 1) | (new_bit as u8);
        let flags_result = FlagRegister {
            zero: check_zero && result == 0,
            subtract: false,
            half_carry: false,
            carry: carry_result
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.set_r8(register, result)
    }

    fn rotate_right(
        &mut self, register: u8, use_carry_result: bool, check_zero: bool
    ) -> Result<(), GameBoySystemError> {
        // TODO - check if the carry logic is correct
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let current = self.get_r8(register)?;

        let carry_result = (current & 0x01) != 0;
        // see rotate_left for explanation of this
        let new_bit =
            (use_carry_result && flags_current.carry) || (!use_carry_result && carry_result);
        let result = (current >> 1) | ((new_bit as u8) << 7);
        let flags_result = FlagRegister {
            zero: check_zero && result == 0,
            subtract: false,
            half_carry: false,
            carry: carry_result
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.set_r8(register, result)
   }

    fn swap_bits(&mut self, register: u8) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;
        let result = (current << 4) | (current >> 4);

        let flags = FlagRegister {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: false
        };

        self.registers.set_register(CpuRegister::F, flags.into());
        self.set_r8(register, result)
    }

    fn daa(&mut self) -> Result<(), GameBoySystemError> {
        todo!()
    }

    fn complement(&mut self) {
        let result = !self.registers.get_register(CpuRegister::A);
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let flags_result = FlagRegister {
            subtract: true,
            half_carry: true,
            ..flags_current.into()
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_register(CpuRegister::A, result);
    }

    fn set_carry(&mut self) {
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let flags_result = FlagRegister {
            subtract: false,
            half_carry: false,
            carry: true,
            ..flags_current
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
    }

    fn complement_carry(&mut self) {
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let flags_result = FlagRegister {
            subtract: false,
            half_carry: false,
            carry: !flags_current.carry,
            ..flags_current
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
    }

    fn jump(&mut self, address: u16) {
        self.registers.pc = address;
    }

    fn call(&mut self, address: u16) -> Result<(), GameBoySystemError> {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.memory.store_half_word(self.registers.sp, self.registers.pc)
            .map_err(|_err|
                GameBoySystemError::MemoryWriteError(self.registers.sp, self.registers.pc)
            )?;
        self.registers.pc = address;
        Ok(())
    }

    fn ret(&mut self, enable_interrupts: bool) -> Result<(), GameBoySystemError> {
        let address: u16 = self.memory.load_half_word(self.registers.sp)
            .ok_or(GameBoySystemError::MemoryReadError(self.registers.sp))?;
        self.registers.pc = address;

        if enable_interrupts {
            self.set_interrupts(true);
        }

        Ok(())
    }

    fn test_bit(&mut self, register: u8, bit_idx: u8) -> Result<(), GameBoySystemError> {
        let value = self.get_r8(register)?;
        let bit = ((value >> bit_idx) & 0x01) == 0;
        let flags_current: FlagRegister = self.registers.get_register(CpuRegister::F).into();
        let flags_result = FlagRegister {
            zero: bit,
            subtract: false,
            half_carry: true,
            ..flags_current
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        Ok(())
    }

    fn set_bit(&mut self, register: u8, bit_idx: u8) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;
        let mask = 0x1 << bit_idx;
        let result = current | mask;
        
        self.set_r8(register, result)
    }

    fn reset_bit(&mut self, register: u8, bit_idx: u8) -> Result<(), GameBoySystemError> {
        let current = self.get_r8(register)?;
        let mask = 0x1 << bit_idx;
        let result = current | !mask;
        
        self.set_r8(register, result)
    }

    fn pop_stack(&mut self, register: u8) -> Result<(), GameBoySystemError> {
        let head_value = self.memory.load_half_word(self.registers.sp)
            .ok_or(GameBoySystemError::MemoryReadError(self.registers.sp))?;
        self.registers.sp = self.registers.sp.wrapping_add(2);

        self.set_r16_stk(register, head_value);
        Ok(())
    }

    fn push_stack(&mut self, register: u8) -> Result<(), GameBoySystemError> {
        let push_value = self.get_r16_stk(register);
        self.registers.sp = self.registers.sp.wrapping_sub(2);

        self.memory.store_half_word(self.registers.sp, push_value)
            .map_err(|_err| GameBoySystemError::MemoryWriteError(self.registers.sp, push_value))?;
        Ok(())
    }

    fn add_stack_pointer(&mut self, value: i8) {
        let value_result = self.registers.sp.wrapping_add(value as u16);

        // check carries by looking at results when limiting operations to specific bits
        // I don't know how this works with negative numbers? Would it always carry?
        let half_carry = (self.registers.sp & 0xF).wrapping_add((value as u16) & 0xF) > 0xF;
        let carry = (self.registers.sp & 0xFF).wrapping_add((value as u16) & 0xFF) > 0xFF;

        let flags_result = FlagRegister {
            zero: false,
            subtract: false,
            half_carry,
            carry
        };

        self.registers.set_register(CpuRegister::F, flags_result.into());
        self.registers.set_joined_registers(CpuRegister::H, CpuRegister::L, value_result);
    }

    fn set_interrupts(&mut self, enable_interrupts: bool) {
        self.internal_interrupts = enable_interrupts;
    }
}
