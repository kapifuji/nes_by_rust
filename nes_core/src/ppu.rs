use crate::rom::Mirroring;


struct AddressRegister {
    /// 上位アドレス
    value_high: u8,
    /// 下位アドレス
    value_low: u8,
    /// 次回書き込み先 true: 上位, false: 下位
    hi_ptr: bool,
}

impl Default for AddressRegister {
    fn default() -> Self {
        AddressRegister {
            value_high: 0,
            value_low: 0,
            hi_ptr: true,
        }
    }
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            value_high: 0,
            value_low: 0,
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value_high = (data >> 8) as u8;
        self.value_low = (data & 0x00ff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value_high = data;
        } else {
            self.value_low = data;
        }

        if self.get() > 0x3fff {
            // 3fffを超えた分はミラーリング（0x0000に戻る。）
            self.set(self.get() & 0b0011_1111_1111_1111)
        }
        // 次回書き込み先を入れ替える。（書き込みの度に上位/下位が入れ替わる。）
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value_low;
        self.value_low = self.value_low.wrapping_add(inc);
        if lo > self.value_low {
            // 桁の繰り上がり
            self.value_high = self.value_high.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            // 3fffを超えた分はミラーリング（0x0000に戻る。）
            self.set(self.get() & 0b0011_1111_1111_1111);
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value_high as u16) << 8) | (self.value_low as u16)
    }
}

#[derive(Default)]
struct PpuRegister {
    ppu_control: u8,
    ppu_mask: u8,
    ppu_status: u8,
    oam_address: u8,
    oam_data: u8,
    ppu_scroll: u8,
    ppu_address: u8,
    ppu_data: u8,
}

pub struct Ppu {
    charactor_rom: Vec<u8>,
    palette_table: [u8; 0x20],
    vram: [u8; 0x800],
    oam_data: [u8; 0x100],
    mirroring: Mirroring,
}

impl Ppu {
    pub fn new(chr_rom_data: &Vec<u8>, mirroring: Mirroring) -> Ppu {
        Ppu {
            charactor_rom: chr_rom_data.clone(),
            palette_table: [0; 0x20],
            vram: [0; 0x800],
            oam_data: [0; 0x100],
            mirroring: mirroring,
        }
    }
}
