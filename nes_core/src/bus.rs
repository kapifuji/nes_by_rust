use crate::{cpu::PpuRegister, rom::Rom};

pub struct Bus {
    /// 0x0000 ~ 0x07FF (0x0100 ~ 0x01ff is stack)
    wram: [u8; 0x800],
    /// 0x2000 ~ 0x2007
    ppu_register: PpuRegister,
    /// 0x8000 ~ 0xFFFF
    rom: Rom,
}
impl Bus {
    pub fn new(rom: &Rom) -> Self {
        let wram: [u8; 0x800] = [0; 0x800];
        let ppu_register = PpuRegister::default();

        Self {
            wram,
            ppu_register,
            rom: rom.clone(),
        }
    }

    fn read_program_rom_byte(&self, address: u16) -> u8 {
        let mut address = address - 0x8000;
        // Program ROM 大きさは 0x4000 または 0x8000
        // 0x4000の場合、0x4000以降へのアクセスは 0x0000 ~ 0x3FFF にミラーリングされる。
        if self.rom.program.len() == 0x4000 && address >= 0x4000 {
            address = address % 0x4000;
        }
        self.rom.program[address as usize]
    }

    pub fn read_memory_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x1fff => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                self.wram[address as usize]
            }
            0x2000..=0x3fff => {
                // 0x2000 ~ 0x3fff は 0x2000 ~ 0x2008 のみ有効。（以降はミラーリンク）
                // よって、bit0 ~ bit2, bit13 を mask
                let address = address & (0b0010_0000_0000_0111);
                todo!()
            }
            0x8000..=0xffff => self.read_program_rom_byte(address),
            _ => panic!("unexpected address: {}", address),
        }
    }

    pub fn read_memory_word(&self, address: u16) -> u16 {
        match address {
            0..=0x1ffe => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                let lo = self.read_memory_byte(address);
                let hi = self.read_memory_byte(address + 1);
                (lo as u16) + ((hi as u16) << 8)
            }
            0x2000..=0x3ffe => {
                // 0x2000 ~ 0x3fff は 0x2000 ~ 0x2008 のみ有効。（以降はミラーリンク）
                // よって、bit0 ~ bit2, bit13 を mask
                let address = address & (0b0010_0000_0000_0111);
                todo!()
            }
            0x8000..=0xfffe => {
                let lo = self.read_program_rom_byte(address);
                let hi = self.read_program_rom_byte(address + 1);
                (lo as u16) + ((hi as u16) << 8)
            }
            _ => panic!("unexpected address: {}", address),
        }
    }

    pub fn write_memory_byte(&mut self, address: u16, value: u8) {
        let write_ref = match address {
            0..=0x1fff => {
                // 0x0000 ~ 0x1fff は 0x0000 ~ 0x07ff のみ有効。（以降はミラーリング）
                // よって、bit0 ~ 10 を mask
                let address = address & (0b0000_0111_1111_1111);
                &mut self.wram[address as usize]
            }
            0x2000..=0x3fff => {
                // 0x2000 ~ 0x3fff は 0x2000 ~ 0x2008 のみ有効。（以降はミラーリンク）
                // よって、bit0 ~ bit2, bit13 を mask
                let address = address & (0b0010_0000_0000_0111);
                todo!()
            }
            0x8000..=0xffff => panic!("ROM is readonly"),
            _ => panic!("unexpected address: {}", address),
        };
        *write_ref = value;
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
            0x2000..=0x3ffe => {
                // 0x2000 ~ 0x3fff は 0x2000 ~ 0x2008 のみ有効。（以降はミラーリンク）
                // よって、bit0 ~ bit2, bit13 を mask
                let address = address & (0b0010_0000_0000_0111);
                todo!()
            }
            0x8000..=0xfffe => {
                panic!("ROM is readonly")
            }
            _ => panic!("unexpected address: {}", address),
        }
    }
}
