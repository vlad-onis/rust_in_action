use crate::cpu::Cpu;

mod cpu;

fn main() {
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
