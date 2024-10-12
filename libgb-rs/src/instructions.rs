/// #Operation
/// Represents a CPU instruction for the Sharp SM83 (CPU used by the Game Boy & Game Boy Color)
#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    NOP,
    Load8(u8, u8), // Load 8-bit register (register, value)
    Load16(u8, u16), // Load 16-bit register (register, value)
    Store8(u16, u8), // Store an 8-bit value in memory (address, value)
    Store16(u16, u16), // Store a 16-bit value in memory (address, value)
    Add8(u8, bool), // value to add to A, and whether the carry flag should be used in op
    Add16(u16), // value to add to HL
    Sub8(u8, bool), // value to subtract from A, and whether the carry flag should be used in op
    And8(u8), // value to do bitwise and with A
    Or8(u8), // value to do bitwise or with A
    Xor8(u8), // value to do bitwise xor with A
    Compare8(u8), // value to compare with A (same as Sub8 but without storing result)
    Increment8(u8), // register to increment
    Increment16(u8), // register to increment 
    Decrement8(u8), // register to decrement
    Decrement16(u8), // register to decrement
    RotateLeft(u8, bool), // Rotate register left 1 bit. Bool is whether to use carry bit in op
    RotateRight(u8, bool), // Rotate register right 1 bit. Bool is whether to use carry bit in op
    ShiftLeftArithmetic(u8), // Shift register left 1 bit
    ShiftRightArithmetic(u8), // Shift register right 1 bit, keeping most significant bit (MSB)
    ShiftRightLogical(u8), // Shift the register right 1 bit, using 0 as the new MSB
    SwapBits(u8), // Swap the upper and lower 4 bits of the given register
    DAA, // ???
    Complement, // A = !A
    SetCarryFlag, // Set c = 1
    ComplementCarryFlag, // Set c = !c
    Jump(u16), // Address to jump to
    Call(u16), // Address to jump to, storing next address on the stack
    Return(bool), // Return to the previous address on the stack, and whether to enable interrupts
    TestBit(u8, u8), // Set C to the value of the target bit in the target register (reg, bit)
    ResetBit(u8, u8), // Set the target bit in the target register to 0 (reg, bit)
    SetBit(u8, u8), // Set the target bit in the target register to 1 (reg, bit)
    PopStack(u8), // Pop the last 2 bytes of the stack into the given 16-bit register
    PushStack(u8), // Push the value in the given 16-bit register onto the stack
    EnableInterrupts,
    DisableInterrupts,
    Stop,
    Halt,
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
