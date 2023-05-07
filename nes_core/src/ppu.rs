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
struct StatusRegister {
    ppu_open_bus: u8,
    sprite_overflow: bool,
    sprite_0_hit: bool,
    ///　Vertical blank has started (false: not in vblank; true: in vblank)
    vertical_blank: bool,
}

impl StatusRegister {
    fn new() -> Self {
        StatusRegister::default()
    }

    fn is_vertical_blank(&self) -> bool {
        self.vertical_blank
    }

    fn set_vertical_blank(&mut self) {
        self.vertical_blank = true;
    }

    fn reset_vertical_blank(&mut self) {
        self.vertical_blank = false;
    }
}

#[derive(Default)]
struct PpuRegister {
    ppu_control: ControlRegister,
    ppu_mask: u8,
    ppu_status: StatusRegister,
    oam_address: u8,
    oam_data: u8,
    ppu_scroll: u8,
    ppu_address: AddressRegister,
    ppu_data: u8,
}

pub struct Ppu {
    /// 0x0000 ~ 0x1FFF
    pub charactor_rom: Vec<u8>,
    /// 0x3F00 ~ 0x3FFF
    palette_table: [u8; 0x20],
    /// 0x2000 ~ 0x3F00
    pub vram: [u8; 0x800],
    oam_data: [u8; 0x100],
    mirroring: Mirroring,
    register: PpuRegister,
    internal_data_buf: u8,

    scanline: u16,
    cycles: usize,

    nmi_interrupt: bool,
}

impl Ppu {
    pub fn new(chr_rom_data: &Vec<u8>, mirroring: Mirroring) -> Ppu {
        Ppu {
            charactor_rom: chr_rom_data.clone(),
            palette_table: [0; 0x20],
            vram: [0; 0x800],
            oam_data: [0; 0x100],
            mirroring: mirroring,
            register: PpuRegister::default(),
            internal_data_buf: 0,

            scanline: 0,
            cycles: 0,
            nmi_interrupt: false,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        // 1 scanline あたり、341 PPU clock
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scanline += 1;

            // 241 scanline の時点でNMI割り込みをトリガーする。
            if self.scanline == 241 {
                self.register.ppu_status.set_vertical_blank();
                if self.register.ppu_control.read_generate_nmi() {
                    self.nmi_interrupt = true;
                }
            }

            // PPU は 262 scanline をレンダリングする。
            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = false;
                self.register.ppu_status.reset_vertical_blank();
                return true;
            }
        }
        return false;
    }

    pub fn poll_nmi_status(&self) -> bool {
        self.register.ppu_control.read_generate_nmi()
    }

    pub fn write_to_ppu_address(&mut self, value: u8) {
        self.register.ppu_address.update(value);
    }

    pub fn write_to_control(&mut self, value: u8) {
        let before_nmi_status = self.register.ppu_control.read_generate_nmi();
        self.register.ppu_control.update(value);
        if !before_nmi_status
            && self.register.ppu_control.read_generate_nmi()
            && self.register.ppu_status.is_vertical_blank()
        {
            self.nmi_interrupt = true;
        }
    }

    /// 事前に address レジスタにアドレスを書き込んでおく。
    /// そのアドレスに対応したPPU内部のデータに書き込む。
    /// 呼び出すたびに、アドレスがインクリメントされる。
    pub fn write_to_data(&mut self, value: u8) {
        let address = self.register.ppu_address.get();
        self.increment_vram_address();

        match address {
            0..=0x1fff => {
                self.charactor_rom[address as usize] = value;
            }
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_address(address) as usize] = value;
            }
            0x3000..=0x3eff => panic!(
                "アドレス 0x3000 ~ 0x3eff 使用を想定していません。 要求: {}",
                address
            ),
            0x3f00..=0x3fff => self.palette_table[(address - 0x3f00) as usize] = value,
            _ => panic!(
                "ミラーリング領域への予期せぬライトアクセスです。 要求: {}",
                address
            ),
        }
    }

    fn increment_vram_address(&mut self) {
        self.register
            .ppu_address
            .increment(self.register.ppu_control.vram_address_increment());
    }

    /// 事前に address レジスタにアドレスを書き込んでおく。
    /// そのアドレスに対応したPPU内部のデータを返す。
    /// 呼び出すたびに、アドレスがインクリメントされる。
    pub fn read_data(&mut self) -> u8 {
        let address = self.register.ppu_address.get();
        self.increment_vram_address();

        match address {
            0..=0x1fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.charactor_rom[address as usize];
                result
            }
            0x2000..=0x2fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_address(address) as usize];
                result
            }
            0x3000..=0x3eff => panic!(
                "アドレス 0x3000 ~ 0x3eff 使用を想定していません。 要求: {}",
                address
            ),
            0x3f00..=0x3fff => self.palette_table[(address - 0x3f00) as usize],
            _ => panic!(
                "ミラーリング領域への予期せぬリードアクセスです。 要求: {}",
                address
            ),
        }
    }

    pub fn mirror_vram_address(&self, address: u16) -> u16 {
        // 画面は2面ある。1面あたり0x400で、vramの0x800に収める。
        // ただ、アドレス空間上は4面ある。（0x2000 ~ 0x3fffで、0x3000~は0x2000~にミラーリングする。）
        // 下の例だと、左上、右上、左下、右下の順にマッピングされる。
        // vramにはＡ, Bの順で格納する。a, bはそれぞれA, Bにミラーリングする。
        //
        // Horizontal(水平):
        //   [ A ] [ a ]
        //   [ B ] [ b ]
        //
        // Vertical(垂直):
        //   [ A ] [ B ]
        //   [ a ] [ b ]
        //

        // 0x3000 ~ 0x3eff を 0x2000 ~ 0x2eff にする。（ミラーリング）
        let mirrored_vram = address & 0b0010_1111_1111_1111;
        // vram index（0x2000始まりなので、計算のため減算。）
        let vram_index = mirrored_vram - 0x2000;
        // name table index（4面のうち、そのアドレス領域を指すか分かる。）
        let name_table = vram_index / 0x400;

        // ミラーリング指定に応じてミラーリング
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn read_background_pattern_address(&self) -> u16 {
        if self.register.ppu_control.background_pattern_address {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn poll_nmi_interrupt(&self) -> bool {
        self.nmi_interrupt
    }
}
