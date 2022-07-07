/*
OPCODE definition:
    1st hex letter (4 bits): operation caregory
    2nd and 3rd hex letter: registers
    4th hex letter: the actual operation

    Opcode cateogies:
        0x8__X: means the operation X will be performend on the 2 registers provided
        0x7YZZ: means that the ZZ arg will be added to the Y register
        0x2AAA: means that it will jump in memory to address AAA
        0x00EE: sets position in memory to the previous call opcode
 */

pub struct Cpu {
    pub registers: [u8; 16],
    pub position_in_memory: usize,
    pub memory: [u8; 0x1000], // Mmmory Holds u8 slots to be similar to the hw architecture of CHIP-8
    pub stack: [u16; 16],     // allows 16 nested function calls
    pub stack_pointer: usize,
}

impl Cpu {
    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.position_in_memory += 2;

            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;

            let addr = opcode & 0x0FFF;

            match (c, x, y, d) {
                (0, 0, 0, 0) => {
                    return;
                }
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(addr),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!("Opcode group:{} Operation:{} - not implemented", c, d),
            }
        }
    }

    fn read_opcode(&self) -> u16 {
        let p = self.position_in_memory;
        let op_byte1 = self.memory[p] as u16;
        let op_byte2 = self.memory[p + 1] as u16;

        let opcode = op_byte1 << 8 | op_byte2;
        opcode
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1; // Last register (the 16th) represents the carry
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &self.stack;

        if sp > stack.len() {
            panic!("Stack overflow");
        }

        self.stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = addr as usize;
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        let call_addr = self.stack[self.stack_pointer];
        self.position_in_memory = call_addr as usize;
    }
}

#[cfg(test)]
mod tests {
    use crate::Cpu;

    #[test]
    fn test_add_call_ret() {
        let mut cpu = Cpu {
            registers: [0; 16],
            position_in_memory: 0,
            memory: [0; 4096],
            stack: [0; 16],
            stack_pointer: 0,
        };

        cpu.registers[0] = 5;
        cpu.registers[1] = 10;

        let mem = &mut cpu.memory;

        // 2 Positions in memory represent 1 instruction
        mem[0x000] = 0x21;
        mem[0x001] = 0x00; // Call the function at 0x100
        mem[0x002] = 0x21;
        mem[0x003] = 0x00; // Call the function at 0x100
        mem[0x004] = 0x21;
        mem[0x005] = 0x00; // Call the function at 0x100

        mem[0x100] = 0x80;
        mem[0x101] = 0x14; // Add reg[0] value to reg[1] value
        mem[0x102] = 0x80;
        mem[0x103] = 0x14; // Add reg[0] value to reg[1] value
        mem[0x104] = 0x00;
        mem[0x105] = 0xEE; // Return
        cpu.run();

        assert_eq!(cpu.registers[0], 65);
    }

}