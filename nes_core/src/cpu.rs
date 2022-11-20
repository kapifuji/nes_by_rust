use crate::ppu::PpuRegister;

struct CpuRegister {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: StatusRegister,
    pc: u8,
}

struct StatusRegister {
    /// bit7 negative
    /// 演算結果が1のときセット
    n: bool,
    /// bit6 overflow
    v: bool,
    /// bit5 reserved
    r: bool,
    /// bit4 break mode
    b: bool,
    /// bit3 decimal mode
    d: bool,
    /// bit2 not allowed IRQ
    i: bool,
    /// bit1 zero
    z: bool,
    /// bit0 carry
    c: bool,
}

struct CpuMemory {
    /// 0x0000 ~ 0x07FF
    wram: [u8; 0x800],
    /// 0x2000 ~ 0x2007
    ppu_register: PpuRegister,
    /// 0x8000 ~ 0xBFFF
    prg_rom: [u8; 0x8000],
}

pub struct Cpu {
    register: CpuRegister,
    memory_map: CpuMemory,
}

impl Cpu {
    pub fn new(prg_rom_data: &Vec<u8>) -> Cpu {
        unimplemented!()
    }
    pub fn tick() {
        unimplemented!()
    }
}
