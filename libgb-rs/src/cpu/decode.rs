use crate::{GameBoySystem, GameBoySystemError};
use crate::cpu::instructions::{Instruction, Operation};

use super::{CpuRegister, FlagRegister};

const REG_A: u8 = 0;
const REG_MEM_READ: u8 = 6;

impl GameBoySystem {
    // -- DEV DESIGN NOTE --
    // This implementation uses a lot of panics and asserts. This is because I want to make sure 
    // on a library level that the code where a panic occurs can never be reached.
    // It should be impossible for a library user to reach these panics. In a situation caused
    // by invalid parameters or an invalid GameBoy state, an error will be returned instead of
    // panicking.

    /// Highly recommend looking at the following page: 
    ///
    /// - https://gbdev.io/pandocs/CPU_Instruction_Set.html
    ///
    /// It has tables showing the bit-layout of the instructions which is the basis of most of the
    /// bitwise logic/bitshifting going on here
    pub fn load_instruction(&mut self) -> Result<Instruction, GameBoySystemError>{
        let instruction = self.fetch_byte()?;
        let block = (instruction & 0xC0) >> 6;

        if instruction == 0 {
            return Ok(Instruction {
                op: Operation::NOP,
                cycles: 1
            });
        }
        else if instruction == 0x10 {
            self.registers.pc += 1;
            return Ok(Instruction {
                op: Operation::Stop,
                cycles: 1
            })
        }
        
        match block {
            0 => self.load_block_0(instruction),
            1 => self.load_block_1(instruction),
            2 => self.load_block_2(instruction),
            3 => self.load_block_3(instruction),
            _ => panic!("logic error while extracting block from instruction {instruction:#X}")
        }
    }

    fn load_block_0(&mut self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        assert!(instruction & 0xC0 == 0, "Should only call when first 2 bits are 0");
        let fn3 = instruction & 0x07;
        if fn3 == 0 && (instruction & 0xF0) != 0 {
            return self.load_jump_relative(instruction);
        }
        if fn3 < 4 {
            return self.load_block_0_16bit(instruction);        
        } else if fn3 == 7 {
            return Ok(self.load_block_0_alu(instruction));
        }

        let reg = (instruction >> 4) & 0x03;
        let mut cycles = 1;
        if reg == REG_MEM_READ {
            // doing anything on [HL] takes more cycles
            cycles = 3;
        } else if fn3 == 6 {
            // loading from memory takes 2 cycles
            cycles = 2;
        }
       
        let result = Instruction {
            cycles,
            op: match fn3 {
                4 => Operation::Increment8(reg),
                5 => Operation::Decrement8(reg),
                6 => Operation::Load8(reg, self.fetch_byte()?),
                _ => panic!("Invalid block 0 fn3 code for instruction {instruction:#X}")
            }
        };

        Ok(result)
    }

    fn load_jump_relative(&mut self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        let jump_type = instruction & 0x20; // the only distinguishing bit between jr and jr [cond]

        // the double cast is done to sign extend into a 16-bit integer. This allows for 16-bit
        // overflow addition of negative numbers (which is effectively subtraction)
        let offset = (self.fetch_byte()? as i8) as u16;
        let address = self.registers.pc.overflowing_add(offset).0;
        let result = Instruction { cycles: 3, op: Operation::Jump(address) };

        if jump_type == 0 {
            return Ok(result);
        }

        let flag_code = (instruction & 0x18) >> 3;
        if self.get_cond_flag(flag_code) {
            Ok(result)
        } else {
            Ok(Instruction { cycles: 2, op: Operation::NOP })
        }
    }

    fn get_cond_flag(&self, flag_code: u8) -> bool {
        let flag_register: FlagRegister = self.registers.get_register(CpuRegister::F)
            .into();
        match flag_code {
            0 => !flag_register.zero,
            1 => flag_register.zero,
            2 => !flag_register.carry,
            3 => flag_register.carry,
            _ => panic!("Impossible flag code found")
        }
    }

    fn load_block_0_16bit(&mut self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        // use a 4-bit opcode for these instructions
        let fn4 = instruction & 0x0F;
        let register = (instruction & 0x18) >> 3;
        // NOTE - ordering is grouped based on the instruction, not a numeric ordering of fn4 codes
        let (op, cycles) = match fn4 {
            1 => (Operation::Load16(register, self.fetch_imm16()?), 3),
            2 => (
                Operation::Store8(
                    self.get_r16_mem(register),
                    self.registers.get_register(CpuRegister::A)
                ), 2
            ),
            0xA => {
                let address = self.get_r16_mem(register);
                (
                    Operation::Load8(
                        7, /* Register A */
                        self.memory.load_byte(address)
                            .ok_or(GameBoySystemError::MemoryReadError(address))?
                    ), 2
                )
            },
            8 => (Operation::Store16(self.fetch_imm16()?, self.registers.sp), 5),
            3 => (Operation::Increment16(register), 2),
            0xB => (Operation::Decrement16(register), 2),
            9 => (Operation::Add16(self.get_r16(register)), 2),
            _ => panic!("Invalid block 0 function 4 in instruction {instruction}")
        };

        Ok(Instruction { op, cycles })
    }

    fn load_block_0_alu(&self, instruction: u8) -> Instruction {
        Instruction {
            cycles: 1,
            op: match instruction {
                // TODO - I smell a pattern here
                0x07 => Operation::RotateLeft(0, true),
                0x0F => Operation::RotateRight(0, true),
                0x17 => Operation::RotateLeft(0, false),
                0x1F => Operation::RotateRight(0, false),
                0x27 => Operation::DAA,
                0x2F => Operation::Complement,
                0x37 => Operation::SetCarryFlag,
                0x3F => Operation::ComplementCarryFlag,
                _ => panic!("Invalid Block 0 ALU instruction {instruction:#X}")
            }
        }
    }

    fn load_block_1(&mut self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        assert!(instruction & 0xC0 == 0x40, "Should not be able to call when block is not 1");

        let src_reg = instruction & 7;
        let dest_reg = (instruction >> 3) & 7;

        // If both registers are [HL] then it should be a halt
        if src_reg == dest_reg && src_reg == REG_MEM_READ {
            return Ok(Instruction { op: Operation::Halt, cycles: 1 });
        } 
        Ok(Instruction {
            op: Operation::Load8(dest_reg, self.get_r8(src_reg)?),
            cycles: 2
        })
    }

    fn load_block_2(&self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        assert!(instruction & 0xC0 == 0x80, "Should not be able to call when block is not 2");
        // 8-bit logic arithmetic
        let register = instruction & 7;
        let value = self.get_r8(register)?;
        let opcode = instruction >> 3;

        let cycles = if register == REG_MEM_READ { 2 } else { 1 };
        // TODO - this is the exact same as the block 3 code
        let operation = match opcode {
            0x10 => Operation::Add8(value, false),
            0x11 => Operation::Add8(value, true),
            0x12 => Operation::Sub8(value, false),
            0x13 => Operation::Sub8(value, true),
            0x14 => Operation::And8(value),
            0x15 => Operation::Xor8(value),
            0x16 => Operation::Or8(value),
            0x17 => Operation::Compare8(value),
            _ => panic!("Should not be able to get to invalid block 2 opcode")
        };

        Ok(Instruction { op: operation, cycles })
    }

    fn load_block_3(&mut self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        assert!(instruction & 0xC0 == 0xC0, "Should not be able to call when block is not 3");

        let fn3 = instruction & 7;
        let tgt = instruction & 0x38;

        if instruction == 0xCB {
            return self.load_prefixed();
        } else if fn3 == 6 {
            return self.load_block_3_alu(instruction);
        } else if fn3 == 7 && (instruction & 0x2) != 0 {
            return Ok(Instruction { op: Operation::Call(tgt as u16), cycles: 4});
        }

        let fn4 = instruction & 0xF;
        if fn4 == 1 || fn4 == 5 {
            return Ok(self.load_block_3_stack(instruction));
        }

        if instruction & 1 == 0 {
            return self.load_block_3_cond(instruction)
        }

        // I kind of hate this but it's fine :upside_down:
        match instruction {
            0xC9 => Ok(Instruction { op: Operation::Return(false), cycles: 4 }),
            0xD9 => Ok(Instruction { op: Operation::Return(true), cycles: 4 }),
            0xC3 => Ok(Instruction { op: Operation::Jump(self.fetch_imm16()?), cycles: 4 }),
            0xE9 => Ok(
                Instruction { 
                    op: Operation::Jump(
                            self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L)
                    ),
                    cycles: 1
                }
            ),
            0xCD => Ok(Instruction { op: Operation::Call(self.fetch_imm16()?), cycles: 6 }),
            0xE0 => Ok(Instruction {
                op: Operation::Store8(
                    0xFF00 + (self.fetch_byte()? as u16),
                    self.registers.get_register(CpuRegister::A)
                ),
                cycles: 3
            }),
            0xE2 => Ok(Instruction { 
                op: Operation::Store8(
                    0xFF00 + (self.registers.get_register(CpuRegister::C) as u16),
                    self.registers.get_register(CpuRegister::A)
                ),
                cycles: 2
            }),
            0xEA => Ok(Instruction {
                op: Operation::Store8(
                    self.fetch_imm16()?,
                    self.registers.get_register(CpuRegister::A)
                ),
                cycles: 4
            }),
            0xF0 => {
                let byte = self.fetch_byte()?;
                let addr = 0xFF00 + (byte as u16);
                let mem_value = self.memory.load_byte(addr)
                    .ok_or(GameBoySystemError::MemoryReadError(addr))?;
                Ok(Instruction { op: Operation::Load8(REG_A, mem_value), cycles: 3 })
            },
            0xF2 => {
                let byte = self.registers.get_register(CpuRegister::C);
                let addr = 0xFF00 + (byte as u16);
                let mem_value = self.memory.load_byte(addr)
                    .ok_or(GameBoySystemError::MemoryReadError(addr))?;
                Ok(Instruction { op: Operation::Load8(REG_A, mem_value), cycles: 3 })
            }
            0xFA => {
                let addr = self.fetch_imm16()?;
                let mem_val = self.memory.load_byte(addr)
                    .ok_or(GameBoySystemError::MemoryReadError(addr))?;
                Ok(Instruction { op: Operation::Load8(REG_A, mem_val), cycles: 4 }) }
            0xE8 => Ok(Instruction { 
                op: Operation::AddStackPointer(self.fetch_byte()? as i8),
                cycles: 4
            }),
            0xF8 => {
                let imm8 = self.fetch_byte()? as i8;
                let new_val = self.registers.sp.overflowing_add(imm8 as u16).0;
                Ok(Instruction { 
                    op: Operation::Load16(2, new_val),
                    cycles: 3
                })
            },
            0xF9 => Ok(Instruction { 
                op: Operation::SetStackPointer(
                    self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L)
                ),
                cycles: 2
            }),
            0xF3 => Ok(Instruction { op: Operation::DisableInterrupts, cycles: 1 }),
            0xFB => Ok(Instruction { op: Operation::EnableInterrupts, cycles: 1 }),
            _ => Err(GameBoySystemError::InvalidInstructionError(instruction))
        }
    }

    fn load_block_3_alu(
        &mut self, instruction: u8
    ) -> Result<Instruction, GameBoySystemError> {
        let imm8 = self.fetch_byte()?;
        let fn3 = (instruction >> 3) & 7;
        let op = match fn3 {
            0 => Operation::Add8(imm8, false),
            1 => Operation::Add8(imm8, true),
            2 => Operation::Sub8(imm8, false),
            3 => Operation::Sub8(imm8, true),
            4 => Operation::And8(imm8),
            5 => Operation::Xor8(imm8),
            6 => Operation::Or8(imm8),
            7 => Operation::Compare8(imm8),
            x => panic!("Found invalid function 3 code {x} in instruction {instruction:#X}")
        };

        Ok(Instruction { op, cycles: 2 })
    }

    fn load_block_3_stack(&mut self, instruction: u8) -> Instruction {
        let r16stk = (instruction >> 4) & 3;
        match instruction & 0xF {
            1 => Instruction { op: Operation::PopStack(r16stk), cycles: 3 },
            5 => Instruction { op: Operation::PushStack(r16stk), cycles: 3 },
            _ => panic!("Invalid instruction {instruction:#X} passed to load stack")
        }
    }

    fn load_block_3_cond(&mut self, instruction: u8) -> Result<Instruction, GameBoySystemError> {
        let fn3 = instruction & 7;
        let cond_flag = self.get_cond_flag((instruction >> 3) & 3);
        // Don't do anything if the condition is not met
        if !cond_flag {
            return Ok(Instruction {
                op: Operation::NOP,
                cycles: match fn3 {
                    0 => 2,
                    2 => 3,
                    4 => 3,
                    _ => panic!("Invalid instruction {instruction:#X} passed to block 3 cond")
                }
            });
        }
        
        match fn3 {
            0 => Ok(Instruction { op: Operation::Return(false), cycles: 5 }),
            2 => Ok(Instruction { op: Operation::Jump(self.fetch_imm16()?), cycles: 4 }),
            4 => Ok(Instruction { op: Operation::Call(self.fetch_imm16()?), cycles: 6 }),
            _ => panic!("Invalid instruction {instruction:#X} passed to block 3 cond")
        }
    }

    fn load_prefixed(&mut self) -> Result<Instruction, GameBoySystemError> {
        let instruction = self.fetch_byte()?;
        let fn2 = instruction >> 6;
        let index = (instruction >> 3) & 7;
        let register = instruction & 7;
        let mut cycles = 2;
        if fn2 == 1 && register == REG_MEM_READ  {
            cycles = 3;
        }
        else if register == REG_MEM_READ {
            cycles = 4;
        }
        match fn2 {
            0 => Ok(Instruction { op: self.load_prefixed_alu(index, register), cycles }),
            1 => Ok(Instruction { op: Operation::TestBit(register, index), cycles }),
            2 => Ok(Instruction { op: Operation::ResetBit(register, index), cycles }),
            3 => Ok(Instruction { op: Operation::SetBit(register, index), cycles }),
            x => panic!("Found invalid prefixed fn2 code {x} in instruction {instruction:#X}")
        }
    }

    fn load_prefixed_alu(&mut self, fn3: u8, register: u8) -> Operation {
        assert!(register < 8, "invalid register should never be provided");
        match fn3 {
            0 => Operation::RotateLeft(register, true),
            1 => Operation::RotateRight(register, true),
            2 => Operation::RotateLeft(register, false),
            3 => Operation::RotateRight(register, false),
            4 => Operation::ShiftLeftArithmetic(register),
            5 => Operation::ShiftRightArithmetic(register),
            6 => Operation::SwapBits(register),
            7 => Operation::ShiftRightLogical(register),
            x => panic!("Invalid prefixed alu function 3 code {x}")
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::random;

    use crate::GameBoySystem;
    use crate::memory::MockMemoryController;

    #[test]
    fn fuzz_test_instructions() {
        let mut mem = MockMemoryController::new();
        mem.expect_load_half_word()
            .return_const(0xFFFF);
        mem.expect_load_byte()
            .returning(|_| {
                // According to Pan Docs, these should be the only invalid instructions
                let invalid_instructions: Vec<u8> = vec![
                    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD
                ];

                let mut rand: u8 = random();
                while invalid_instructions.contains(&rand) { rand = random(); }
                Some(rand)
            });

        let mut dmg: GameBoySystem = GameBoySystem::new(Box::new(mem));
        
        // technically this is not guaranteed to test everything but realistically it should
        for _ in 0..10_000 {
            let result = dmg.load_instruction();
            assert!(result.is_ok(), "");
        }
    }
}

