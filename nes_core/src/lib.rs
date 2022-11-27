mod cpu;
mod ppu;
mod rom;

pub struct Nes {
    rom_header: rom::Header,
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,
}

impl Nes {
    pub fn new(rom_data: Vec<u8>) -> Nes {
        todo!()
    }

    pub fn tick_cpu() {
        todo!()
    }

    pub fn tick_ppu() {
        todo!()
    }
}
