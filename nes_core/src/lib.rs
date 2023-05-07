mod bus;
pub mod cpu;
mod opcode;
pub mod ppu;
pub mod rom;

pub struct Nes<'a> {
    //rom_header: rom::Header,
    pub cpu: cpu::Cpu<'a>,
    //ppu: ppu::Ppu,
}

impl<'a> Nes<'a> {
    pub fn new<'b, F>(rom_data: &Vec<u8>, gameloop_callback: F) -> Nes<'b>
    where
        F: FnMut(&ppu::Ppu) + 'b,
    {
        let rom = rom::Rom::new(rom_data);
        let bus = bus::Bus::new(&rom, gameloop_callback);
        Nes {
            cpu: cpu::Cpu::new(bus),
        }
    }

    pub fn run_with_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut cpu::Cpu),
    {
        self.cpu.run_with_callback(callback);
    }
}
