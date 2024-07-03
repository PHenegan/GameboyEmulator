pub struct RegisterIndexError;

/// The CPU of a Gameboy/Gameboy Color system
pub struct CPU {
    // 7 8-bit registers A-L, followed by the last flag register F 
    registers: Vec<u8>, 
    pub sp: u16,
    pub pc: u16
}

impl CPU {

    pub fn new() -> CPU {
        return CPU {
            registers: vec![0, 0, 0, 0, 0, 0, 0, 0],
            sp: 0,
            pc: 0
        };
    }

    pub fn get_register<'a>(&'a self, idx: usize) -> Option<&u8> {
        self.registers.get(idx)
    }

    pub fn get_register_mut<'a>(&'a mut self, idx: usize) -> Option<&mut u8> {
       self.registers.get_mut(idx) 
    }

    pub fn get_joined_registers(&self, idx1: usize, idx2: usize) -> Option<u16> {
        // convert the two Option enums into a single option with a tuple,
        // join the two integers using bitshifting
        self.registers.get(idx1)
            .zip(self.registers.get(idx2))
            .map(|(val1, val2)| {
                ((*val1 as u16) << 8) + *val2 as u16
            })
    }

    pub fn set_joined_registers(
        &mut self, idx1: usize, idx2: usize, data: u16
    ) -> Result<(), RegisterIndexError> {

        // Register 1 gets the 8 most significant bits (done by shifting, then casting)
        let reg1: &mut u8 = self.registers.get_mut(idx1)
            .ok_or(RegisterIndexError)?;
        *reg1 = (data >> 8) as u8;
        
        // Register 2 gets the 8 least significant bits (done by simply casting)
        let reg2: &mut u8 = self.registers.get_mut(idx2)
            .ok_or(RegisterIndexError)?;
        *reg2 = data as u8;
        Ok(())
    }
}

