#[derive(Clone, Copy)]
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

/// The CPU of a Gameboy/Gameboy Color system
pub struct CpuData {
    // 7 8-bit registers A-L, followed by the last flag register F 
    registers: Vec<u8>, 
    pub sp: u16,
    pub pc: u16
}

impl CpuData {
    pub fn new() -> CpuData {
        return CpuData {
            registers: vec![0, 0, 0, 0, 0, 0, 0, 0],
            sp: 0,
            pc: 0
        };
    }

    pub fn get_register<'a>(&'a self, idx: &CpuRegister) -> &'a u8 {
        self.registers.get(*idx as usize)
            .unwrap()
    }

    pub fn get_register_mut<'a>(&'a mut self, idx: &CpuRegister) -> &'a mut u8 {
       self.registers.get_mut(*idx as usize)
           .unwrap()
    }

    pub fn get_joined_registers(&self, idx1: &CpuRegister, idx2: &CpuRegister) -> u16 {
        let reg1 = self.get_register(idx1);
        let reg2 = self.get_register(idx2);
        
        // join the two integers using bitshifting
        ((*reg1 as u16) << 8) + *reg2 as u16
    }

    pub fn set_joined_registers(&mut self, idx1: &CpuRegister, idx2: &CpuRegister, data: u16) {
        // Register 1 gets the 8 most significant bits
        let reg1: &mut u8 = self.get_register_mut(idx1);
        *reg1 = (data >> 8) as u8;
        
        // Register 2 gets the 8 least significant bits
        let reg2: &mut u8 = self.get_register_mut(idx2);
        *reg2 = data as u8;
    }
}

