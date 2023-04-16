mod bus;
mod opcode;
mod ppu;
mod rom;

pub struct Nes {
    rom_header: rom::Header,
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,
}

impl Nes {
    pub fn new(rom_data: &Vec<u8>) -> Nes {
        let rom = rom::Rom::new(rom_data);
        Nes {
            cpu: cpu::Cpu::new(&rom),
        }
    }

    pub fn tick_cpu() {
        todo!()
    }

    pub fn tick_ppu() {
        todo!()
    }
}
