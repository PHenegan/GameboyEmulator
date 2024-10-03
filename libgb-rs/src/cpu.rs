use crate::utils::{Merge, Split};

/// # CpuRegister
/// An enum storing each of the lettered registers in a Game Boy CPU.
#[derive(Debug, Clone, Copy)]
pub enum CpuRegister {
    A = 0,
    B = 1,
    C = 2, 
    D = 3,
    E = 4,
    H = 5,
    L = 6,
    F = 7
}

/// #FlagRegister
/// A convenient struct for holding CPU flags
#[derive(Debug, Clone, Copy)]
pub struct FlagRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool
}

impl From<FlagRegister> for u8 {
    fn from(value: FlagRegister) -> Self {
        ((value.zero as u8) << 7)
            | ((value.subtract as u8) << 6)
            | ((value.half_carry as u8) << 5)
            | ((value.carry as u8) << 4)
    }
}

impl From<u8> for FlagRegister {
    fn from(value: u8) -> Self {
        FlagRegister {
            zero: (value & 0x80) != 0,
            subtract: (value & 0x40) != 0,
            half_carry: (value & 0x20) != 0,
            carry: (value & 0x10) != 0
        }
    }
}

/// # CpuData
/// The CPU Registers of a Gameboy/Gameboy Color system
pub struct CpuData {
    // 7 8-bit registers A-L, followed by the last flag register F 
    registers: Vec<u8>, 
    pub sp: u16,
    pub pc: u16
}

impl CpuData {
    pub fn new() -> Self {
        CpuData {
            registers: vec![0; 8],
            sp: 0,
            pc: 0
        }
    }

    pub fn get_register(&self, idx: CpuRegister) -> u8 {
        // option isn't necessary since the type is being used to guarantee bounds
        self.registers[idx as usize]
    }

    pub fn set_register(&mut self, idx: CpuRegister, value: u8) {
        // result isn't necessary since the type is being used to guarantee bounds
        self.registers[idx as usize] = value;
    }

    /// Get a 16-bit by joining two bytes from the given registers in Little-Endian ordering
    pub fn get_joined_registers(&self, idx1: CpuRegister, idx2: CpuRegister) -> u16 {
        let right = self.get_register(idx1);
        let left = self.get_register(idx2);
        left.merge(right)
    }

    /// Store a 16-bit value by splitting the given data and storing it in Little-Endian ordering
    /// into the given registers
    pub fn set_joined_registers(&mut self, idx1: CpuRegister, idx2: CpuRegister, data: u16) {
        let (left_data, right_data) = data.split();

        // Register 1 gets the 8 most significant bits
        self.set_register(idx1, right_data);
        // Register 2 gets the 8 least significant bits
        self.set_register(idx2, left_data);
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::Merge;

    use super::{CpuData, CpuRegister};

    #[test]
    fn test_endianness() {
        let mut data = CpuData::new();
        data.set_joined_registers(CpuRegister::B, CpuRegister::C, 0xBEEF);

        let right = data.get_register(CpuRegister::B);
        let left = data.get_register(CpuRegister::C);

        let n16 = data.get_joined_registers(CpuRegister::B, CpuRegister::C);

        assert_eq!(n16, left.merge(right), "Data should be assigned in Little Endian order");
    }
}
