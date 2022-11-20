/// 16 byte header
pub struct Header {
    /// 0-3: Constant $4E $45 $53 $1A ("NES" followed by MS-DOS end-of-file)
    constant: [u8; 4],
    /// 4: size of PRG ROM in 16 KB units
    prg_rom_size: u8,
    /// 5: Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM)
    chr_rom_size: u8,
    /// 6: Mapper, mirroring, battery, trainer
    flags6: Flags6,
    /// 7: Mapper, VS/Playchoice, NES 2.0
    flags7: Flags7,
    /// 8: PRG-RAM size (rarely used extension)
    flags8: Flags8,
    /// 9: TV system (rarely used extension)
    flags9: Flags9,
    /// 10: TV system, PRG-RAM presence (unofficial, rarely used extension)
    flags10: Flags10,
    /// 11-15: Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15)
    unused: [u8; 5],
}
struct Flags6 {
    /// Mirroring:\
    /// 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)\
    /// 1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
    mirroring: bool,
    /// 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
    is_persistent_memory: bool,
    /// 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
    is_trainer: bool,
    /// 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
    ignore_mirroring: bool,
    /// Lower nybble of mapper number (size is 4 bit)
    lower_mapper: u8,
}
struct Flags7 {
    /// 1: VS Unisystem
    vs_unisystem: bool,
    /// 1: PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
    play_choice_10: bool,
    /// 1: flags 8-15 are in NES 2.0 format
    is_nes_2_0: bool,
    /// Upper nybble of mapper number (size is 4 bit)
    upper_mapper: u8,
}
struct Flags8 {
    prg_ram_size: u8,
}
struct Flags9 {
    /// TV system (0: NTSC, 1: PAL)
    tv_system: bool,
}
struct Flags10 {
    /// TV system (0: NTSC, 2: PAL, 1 or 3: dual compatible)
    tv_system: u8,
    /// PRG RAM ($6000-$7FFF) (0: present; 1: not present)
    is_prg_ram: bool,
    /// 0: Board has no bus conflicts, 1: Board has bus conflicts
    is_bus_conflicts: bool,
}

impl Header {
    pub fn new(header_data: &[u8; 16]) -> Header {
        unimplemented!()
    }
}

pub struct Rom {
    header: Header,
    program: Vec<u8>,
    charactor: Vec<u8>,
}

impl Rom {
    pub fn new(rom_data: &Vec<u8>) -> Rom {
        unimplemented!()
    }
}
