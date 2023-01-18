#[derive(Default)]
pub struct PpuRegister {
    ppu_control: u8,
    ppu_mask: u8,
    ppu_status: u8,
    oam_address: u8,
    oam_data: u8,
    ppu_scroll: u8,
    ppu_address: u8,
    ppu_data: u8,
}

struct CpuRegister {
    a: u8,
    x: u8,
    y: u8,
    p: StatusRegister,
    sp: u16,
    pc: u16,
}
impl CpuRegister {
    pub fn new() -> Self {
        Self {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            p: StatusRegister::new(),
            sp: 0x01fd,
            pc: 0x00,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.p.reset();
        self.sp = 0x01fd;
        self.pc = 0;
    }
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
impl StatusRegister {
    pub fn new() -> Self {
        Self {
            n: false,
            v: false,
            r: true,
            b: true,
            d: false,
            i: true,
            z: false,
            c: false,
        }
    }

    pub fn reset(&mut self) {
        self.n = false;
        self.v = false;
        self.r = true;
        self.b = true;
        self.d = false;
        self.i = true;
        self.z = false;
        self.c = false;
    }
}

struct CpuMemory {
    /// 0x0000 ~ 0x07FF
    wram: [u8; 0x800],
    /// 0x2000 ~ 0x2007
    ppu_register: PpuRegister,
    /// 0x8000 ~ 0xFFFF
    prg_rom: [u8; 0x8000],
}
impl CpuMemory {
    pub fn new(prg_rom_data: &Vec<u8>) -> Self {
        let wram: [u8; 0x800] = [0; 0x800];
        let ppu_register = PpuRegister::default();
        let mut prg_rom: [u8; 0x8000] = [0; 0x8000];

        for i in 0..prg_rom.len() {
            if i >= prg_rom_data.len() {
                break;
            }
            prg_rom[i] = prg_rom_data[i];
        }

        Self {
            wram,
            ppu_register,
            prg_rom,
        }
    }

    pub fn read_memory_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x7fe => self.wram[address as usize],
            0x2000..=0x2006 => todo!(),
            0x8000..=0xfffe => self.prg_rom[(address - 0x8000) as usize],
            _ => panic!("unexpected address: {}", address),
        }
    }

    pub fn read_memory_word(&self, address: u16) -> u16 {
        match address {
            0..=0x7fe => {
                let ret_under = self.wram[address as usize];
                let ret_upper = self.wram[(address as usize) + 1];
                (ret_under as u16) + ((ret_upper as u16) << 8)
            }
            0x2000..=0x2007 => todo!(),
            0x8000..=0xfffe => {
                let index = address - 0x8000;
                let ret_under = self.prg_rom[index as usize];
                let ret_upper = self.prg_rom[(index as usize) + 1];
                (ret_under as u16) + ((ret_upper as u16) << 8)
            }
            _ => panic!("unexpected address: {}", address),
        }
    }
}

pub struct Cpu {
    register: CpuRegister,
    memory_map: CpuMemory,
}

impl Cpu {
    pub fn new(prg_rom_data: &Vec<u8>) -> Cpu {
        let register = CpuRegister::new();
        let memory_map = CpuMemory::new(&prg_rom_data);

        Self {
            register,
            memory_map,
        }
    }

    pub fn tick() {
        todo!()
    }

    pub fn get_ppu_register() -> PpuRegister {
        todo!()
    }

    pub fn reset(&mut self) {
        self.register.reset();
        self.register.pc = self.memory_map.read_memory_word(0xfffc);
    }

    fn lda(&mut self, value: u8) {
        self.register.a = value;

        self.update_zero_and_negative_flags(self.register.a);
    }

    fn tax(&mut self) {
        self.register.x = self.register.a;

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn inx(&mut self) {
        self.register.x = if self.register.x == 0xff {
            0
        } else {
            self.register.x + 1
        };

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.register.p.z = if result == 0 { true } else { false };

        self.register.p.n = if (result & 0b1000_0000) != 0 {
            true
        } else {
            false
        };
    }

    /// 命令テスト用関数
    pub fn interpret(&mut self) {
        self.register.pc = 0x8000;

        loop {
            let opscode = self.memory_map.read_memory_byte(self.register.pc);
            self.register.pc += 1;

            match opscode {
                0x00 => {
                    return;
                }
                0xA9 => {
                    let param = self.memory_map.read_memory_byte(self.register.pc);
                    self.register.pc += 1;

                    self.lda(param);
                }
                0xAA => {
                    self.tax();
                }
                0xE8 => {
                    self.inx();
                }
                _ => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_new() {
        let mut prg_rom: Vec<u8> = Vec::new();
        for i in 0..5 {
            prg_rom.push(i);
        }
        let cpu = Cpu::new(&prg_rom);

        assert_eq!(cpu.register.sp, 0x01fd);
        assert_eq!(cpu.memory_map.prg_rom[0..5], [0x00, 0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn cpu_reset() {
        let mut prg_rom: Vec<u8> = Vec::new();
        for i in 0u16..0x8000 {
            prg_rom.push((i % 0x100) as u8);
        }
        let mut cpu = Cpu::new(&prg_rom);

        cpu.reset();

        assert_eq!(cpu.register.pc, 0xfdfc);
    }

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let program = vec![0xa9, 0x05, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x05);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let program = vec![0xa9, 0x00, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.p.z, true);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let program = vec![0xaa, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 10;
        cpu.interpret();

        assert_eq!(cpu.register.x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let program = vec![0xe8, 0xe8, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.x = 0xff;
        cpu.interpret();

        assert_eq!(cpu.register.x, 1)
    }
}
