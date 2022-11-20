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

struct PpuMemory {
    /// 0x0000 ~ 0x0FFF
    pattern_table_0: [u8; 0x1000],
    /// 0x1000 ~ 0x1FFF
    pattern_table_1: [u8; 0x1000],
    /// 0x2000 ~ 0x23BF
    name_table_0: [u8; 0x03C0],
    /// 0x23C0 ~ 0x23FF
    attribute_table_0: [u8; 0x0040],
    /// 0x2400 ~ 0x27BF
    name_table_1: [u8; 0x03C0],
    /// 0x27C0 ~ 0x27FF
    attribute_table_1: [u8; 0x0040],
    /// 0x2800 ~ 0x2BBF
    name_table_2: [u8; 0x03C0],
    /// 0x2BC0 ~ 0x2BFF
    attribute_table_2: [u8; 0x0040],
    /// 0x2C00 ~ 0x2FBF
    name_table_3: [u8; 0x03C0],
    /// 0x2FC0 ~ 0x2FFF
    attribute_table_3: [u8; 0x0040],
    /// 0x3000 ~ 0x3EFF
    mirror_0x2000_0x2EFF: [u8; 0x0F00],
    /// 0x3F00 ~ 0x3F0F
    background_palette: [u8; 0x0010],
    /// 0x3F10 ~ 0x3F1F
    sprite_palette: [u8; 0x0010],
    /// 0x3F20 ~ 0x3FFF
    mirror_0x3F00_0x3F1F: [u8; 0x0020],
}
pub struct Ppu {
    memory_map: PpuMemory,
}

impl Ppu {
    pub fn new(chr_rom_data: &Vec<u8>) -> Ppu {
        unimplemented!()
    }
    pub fn tick(register: &PpuRegister) {
        unimplemented!()
    }
}
