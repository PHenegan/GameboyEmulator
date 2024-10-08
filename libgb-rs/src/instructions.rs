/// #Operation
/// Represents a CPU instruction for the Sharp SM83 (CPU used by the Game Boy & Game Boy Color)
pub enum Operation {
    NOP,
    Load8(u8, u8), // Load 8-bit register (register, value)
    Load16(u8, u16), // Load 16-bit register (register, value)
    Store8(u8, u16), // Store a 16-bit register in memory
    Add8(u8), // value to add to A
    Add16(u16), // value to add to HL
    Increment8(u8), // register to increment
    Increment16(u8), // register to increment 
    Decrement8(u8), // register to decrement
    Decrement16(u8), // register to decrement
    RotateLeftA(bool), // Rotate A left 1 bit. Bool is whether to include carry bit in rotate
    RotateRightA(bool), // Rotate A right 1 bit. Bool is whether to include carry bit in rotate 
    DAA, // ???
    Complement, // A = !A
    SetCarryFlag, // Set c = 1
    ComplementCarryFlag, // Set c = !c
    Jump(u16), // Address to jump to
    Stop
}

pub struct Instruction {
    pub cycles: u8,
    pub op: Operation
}

// Some extra opcode notes about block 0
// - If last 3 bits in range 4 <= x < 7, then it's a 3-bit opcode (with 8-bit registers)
//   otherwise, it's a 4-bit opcode
// - If the last 3 bits are 7 then it's an ALU operation on A
// - If the last 3 bits are 0 it's either jump, jump with cond, or stop
