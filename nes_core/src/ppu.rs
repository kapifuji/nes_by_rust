use crate::rom::Mirroring;

const BIT0: u8 = 0b0000_0001;
const BIT1: u8 = 0b0000_0010;
const BIT2: u8 = 0b0000_0100;
const BIT3: u8 = 0b0000_1000;
const BIT4: u8 = 0b0001_0000;
const BIT5: u8 = 0b0010_0000;
const BIT6: u8 = 0b0100_0000;
const BIT7: u8 = 0b1000_0000;

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
struct ControlRegister {
    /// bit0 - 1
    ///
    /// Base nametable address
    /// (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    nametable: u8,
    /// bit2
    ///
    /// VRAM address increment per CPU read/write of PPUDATA
    ///  (0: add 1, going across; 1: add 32, going down)
    vram_add_increment: bool,
    /// bit3
    ///
    /// Sprite pattern table address for 8x8 sprites
    /// (0: $0000; 1: $1000; ignored in 8x16 mode)
    sprite_pattern_address: bool,
    /// bit4
    ///
    /// Background pattern table address (0: $0000; 1: $1000)
    background_pattern_address: bool,
    /// bit5
    ///
    /// Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    sprite_size: bool,
    /// bit6
    ///
    /// PPU master/slave select
    /// (0: read backdrop from EXT pins; 1: output color on EXT pins)
    master_lave_select: bool,
    /// bit7
    ///
    /// Generate an NMI at the start of the
    /// vertical blanking interval (0: off; 1: on)
    generate_nmi: bool,
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::default()
    }

    pub fn vram_address_increment(&self) -> u8 {
        if !self.vram_add_increment {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        self.nametable = data & (BIT0 | BIT1);
        self.vram_add_increment = if data & BIT2 == BIT2 { true } else { false };
        self.sprite_pattern_address = if data & BIT3 == BIT3 { true } else { false };
        self.background_pattern_address = if data & BIT4 == BIT4 { true } else { false };
        self.sprite_size = if data & BIT5 == BIT5 { true } else { false };
        self.master_lave_select = if data & BIT6 == BIT6 { true } else { false };
        self.generate_nmi = if data & BIT7 == BIT7 { true } else { false };
    }

    pub fn read(&self) -> u8 {
        let mut result = match self.nametable {
            1 => BIT0,
            2 => BIT1,
            3 => BIT0 | BIT1,
            _ => 0,
        };
        result |= if self.vram_add_increment { BIT2 } else { 0 };
        result |= if self.sprite_pattern_address { BIT3 } else { 0 };
        result |= if self.background_pattern_address {
            BIT4
        } else {
            0
        };
        result |= if self.sprite_size { BIT5 } else { 0 };
        result |= if self.master_lave_select { BIT6 } else { 0 };
        result |= if self.generate_nmi { BIT7 } else { 0 };

        result
    }

    pub fn read_generate_nmi(&self) -> bool {
        self.generate_nmi
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
