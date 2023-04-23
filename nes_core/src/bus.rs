use crate::{ppu::Ppu, rom::Rom};

pub struct Bus {
    /// 0x0000 ~ 0x07FF (0x0100 ~ 0x01ff is stack)
    wram: [u8; 0x800],
    /// 0x2000 ~ 0x2007
    ppu: Ppu,
    /// 0x8000 ~ 0xFFFF
    program: Vec<u8>,

    cycles: usize,
}
impl Bus {
    pub fn new(rom: &Rom) -> Self {
        let ppu = Ppu::new(&rom.charactor, rom.header.read_mirroring());

        Self {
            wram: [0; 0x800],
            ppu,
            program: rom.program.clone(),
            cycles: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        self.ppu.tick(cycles * 3); // PPUのクロックはCPUの3倍
    }

    fn read_program_rom_byte(&self, address: u16) -> u8 {
        let mut address = address - 0x8000;
        // Program ROM 大きさは 0x4000 または 0x8000
        // 0x4000の場合、0x4000以降へのアクセスは 0x0000 ~ 0x3FFF にミラーリングされる。
        if self.program.len() == 0x4000 && address >= 0x4000 {
            address = address % 0x4000;
        }
        self.program[address as usize]
    }

    pub fn read_memory_byte(&mut self, address: u16) -> u8 {
        match address {
            0..=0x1fff => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                self.wram[address as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("{} は書き込み専用のPPUアドレスです。", address)
            }
            0x2007 => self.ppu.read_data(),
            0x2008..=0x3fff => {
                // 0x2000 ~ 0x3fff は 0x2000 ~ 0x2008 のみ有効。（以降はミラーリング）
                // よって、bit0 ~ bit2, bit13 を mask
                let address = address & (0b0010_0000_0000_0111);
                self.read_memory_byte(address)
            }
            0x8000..=0xffff => self.read_program_rom_byte(address),
            _ => {
                println!("{} へのbyteリードアクセスは無視されます。", address);
                0
            }
        }
    }

    pub fn read_memory_word(&mut self, address: u16) -> u16 {
        match address {
            0..=0x1ffe => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                let lo = self.read_memory_byte(address);
                let hi = self.read_memory_byte(address + 1);
                (lo as u16) + ((hi as u16) << 8)
            }
            0x8000..=0xfffe => {
                let lo = self.read_program_rom_byte(address);
                let hi = self.read_program_rom_byte(address + 1);
                (lo as u16) + ((hi as u16) << 8)
            }
            _ => {
                println!("{} へのwordリードアクセスは無視されます。", address);
                0
            }
        }
    }

    pub fn write_memory_byte(&mut self, address: u16, value: u8) {
        match address {
            0..=0x1fff => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                self.wram[address as usize] = value;
            }
            0x2000 => self.ppu.write_to_control(value),
            0x2006 => self.ppu.write_to_ppu_address(value),
            0x2007 => self.ppu.write_to_data(value),
            0x2008..=0x3fff => {
                // 0x2000 ~ 0x3fff は 0x2000 ~ 0x2008 のみ有効。（以降はミラーリンク）
                // よって、bit0 ~ bit2, bit13 を mask
                let address = address & (0b0010_0000_0000_0111);
                self.write_memory_byte(address, value);
            }
            0x8000..=0xffff => panic!("ROM is readonly"),
            _ => {
                println!("{} へのbyteライトアクセスは無視されます。", address);
            }
        };
    }

    pub fn write_memory_word(&mut self, address: u16, value: u16) {
        let lo = (value & 0x00ff) as u8;
        let hi = ((value & 0xff00) >> 8) as u8;

        match address {
            0..=0x1ffe => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                self.write_memory_byte(address, lo);
                self.write_memory_byte(address + 1, hi)
            }
            0x8000..=0xfffe => {
                panic!("ROM is readonly")
            }
            _ => {
                println!("{} へのwordライトアクセスは無視されます。", address);
            }
        }
    }

    pub fn poll_nmi_status(&self) -> bool {
        self.ppu.poll_nmi_status()
    }
}
