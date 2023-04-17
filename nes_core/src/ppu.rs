use crate::rom::Mirroring;

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
