use crate::{cpu::{CpuData, CpuRegister}, instructions::{Instruction, Operation}, memory::{MemoryController, MemoryWriteError}};

pub struct LoadInstructionError;

pub struct GameBoySystem {
    registers: CpuData,
    memory: Box<dyn MemoryController>,
    // PPU will also need to go here eventually
}

impl GameBoySystem {
    fn fetch_byte(&mut self) -> Result<u8, LoadInstructionError> {
        let byte = self.memory.load_byte(self.registers.pc);
        self.registers.pc += 1;

        byte.ok_or(LoadInstructionError)
    }

    pub fn load_instruction(&mut self) -> Result<Instruction, LoadInstructionError>{
        let instruction = self.fetch_byte()?;
        let block = (instruction & 0xC0) >> 6;

        match block {
            0 => self.load_block_0(instruction),
            1 => todo!("code for block 1"),
            2 => todo!("code for block 2"),
            3 => todo!("code for block 3"),
            _ => return Err(LoadInstructionError)
        }
    }

    fn load_block_0(&mut self, instruction: u8) -> Result<Instruction, LoadInstructionError> {
        let opcode = instruction & 0x07;
        if instruction == 0 {
            return Ok(Instruction {
                op: Operation::NOP,
                cycles: 1
            });
        } else if instruction == 0x10 {
            self.registers.sp += 1;
            return Ok(Instruction {
                op: Operation::Stop,
                cycles: 1
            })
        }
        
        if opcode < 4 {
            return self.load_block_0_16bit(instruction);        
        } else if opcode == 7 {
            return self.load_block_0_alu(instruction);
        }

        // convert 
        let reg = (instruction >> 4) & 0x03;

        let result = Instruction {
            cycles: if opcode == 6 { 2 } else { 1 },
            op: match opcode {
                4 => Operation::Increment8(reg),
                5 => Operation::Decrement8(reg),
                6 => Operation::Load8(reg, self.fetch_byte()?),
                _ => return Err(LoadInstructionError)
            }
        };

        Ok(result)
    }

    

    fn load_block_0_16bit(&mut self, instruction: u8) -> Result<Instruction, LoadInstructionError> {
        // use a 4-bit opcode for these instructions
        let opcode = instruction & 0x0F;

        todo!();
    }

    fn load_block_0_alu(&mut self, instruction: u8) -> Result<Instruction, LoadInstructionError> {
        let result = Instruction {
            cycles: 1,
            op: match instruction {
                // TODO - I smell a pattern here
                0x07 => Operation::RotateLeftA(true),
                0x0F => Operation::RotateRightA(true),
                0x17 => Operation::RotateLeftA(false),
                0x1F => Operation::RotateRightA(false),
                0x27 => Operation::DAA,
                0x2F => Operation::Complement,
                0x37 => Operation::SetCarryFlag,
                0x3F => Operation::ComplementCarryFlag,
                _ => return Err(LoadInstructionError)
            }
        };
        Ok(result)
    }

    fn get_r8(&self, reg: u8) -> Result<u8, LoadInstructionError> {
        if reg == 6 {
            return self.memory.load_byte(
                self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L)
            ).ok_or(LoadInstructionError);
        }

        Ok(self.registers.get_register(reg.into()))
    }

    fn set_r8(&mut self, reg: u8, value: u8) -> Result<(), MemoryWriteError> {
        if reg == 6 {
            self.memory.store_byte(
                self.registers.get_joined_registers(CpuRegister::H, CpuRegister::L),
                value
            )?;
            return Ok(());
        }

        todo!();
    }
}

