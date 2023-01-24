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
    /// false: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)\
    /// true: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
    mirroring: bool,
    /// true: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
    is_persistent_memory: bool,
    /// true: 512-byte trainer at $7000-$71FF (stored before PRG data)
    is_trainer: bool,
    /// true: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
    ignore_mirroring: bool,
    /// Lower nybble of mapper number (size is 4 bit)
    lower_mapper: u8,
}
impl Flags6 {
    pub fn new(data: u8) -> Flags6 {
        let mirroring = (data & 0x01) == 0x01;
        let is_persistent_memory = (data & 0x02) == 0x02;
        let is_trainer = (data & 0x04) == 0x04;
        let ignore_mirroring = (data & 0x08) == 0x08;
        let lower_mapper = data >> 4;

        Flags6 {
            mirroring,
            is_persistent_memory,
            is_trainer,
            ignore_mirroring,
            lower_mapper,
        }
    }
}

struct Flags7 {
    /// true: VS Unisystem
    vs_unisystem: bool,
    /// true: PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
    play_choice_10: bool,
    /// true: flags 8-15 are in NES 2.0 format
    is_nes_2_0: bool,
    /// Upper nybble of mapper number (size is 4 bit)
    upper_mapper: u8,
}
impl Flags7 {
    pub fn new(data: u8) -> Flags7 {
        let vs_unisystem = (data & 0x01) == 0x01;
        let play_choice_10 = (data & 0x02) == 0x02;
        let is_nes_2_0 = (data & 0x08) == 0x08;
        let upper_mapper = data >> 4;

        Flags7 {
            vs_unisystem,
            play_choice_10,
            is_nes_2_0,
            upper_mapper,
        }
    }
}

struct Flags8 {
    prg_ram_size: u8,
}
impl Flags8 {
    pub fn new(data: u8) -> Flags8 {
        Flags8 { prg_ram_size: data }
    }
}

struct Flags9 {
    /// TV system (false: NTSC, true: PAL)
    tv_system: bool,
}
impl Flags9 {
    pub fn new(data: u8) -> Flags9 {
        let tv_system = (data & 0x01) == 0x01;

        Flags9 { tv_system }
    }
}

struct Flags10 {
    /// TV system (0: NTSC, 2: PAL, 1 or 3: dual compatible)
    tv_system: u8,
    /// PRG RAM ($6000-$7FFF) (false: present; true: not present)
    is_not_prg_ram: bool,
    /// false: Board has no bus conflicts, true: Board has bus conflicts
    is_bus_conflicts: bool,
}
impl Flags10 {
    pub fn new(data: u8) -> Flags10 {
        let tv_system = data & 0x02;
        let is_not_prg_ram = (data & 0x10) == 0x10;
        let is_bus_conflicts = (data & 0x20) == 0x20;

        Flags10 {
            tv_system,
            is_not_prg_ram,
            is_bus_conflicts,
        }
    }
}

impl Header {
    pub fn new(header_data: &[u8; 16]) -> Header {
        let mut constant = [0; 4];
        constant.copy_from_slice(&header_data[0..4]);
        let prg_rom_size = header_data[4];
        let chr_rom_size = header_data[5];
        let flags6 = Flags6::new(header_data[6]);
        let flags7 = Flags7::new(header_data[7]);
        let flags8 = Flags8::new(header_data[8]);
        let flags9 = Flags9::new(header_data[9]);
        let flags10 = Flags10::new(header_data[10]);
        let mut unused = [0; 5];
        unused.copy_from_slice(&header_data[11..]);

        Header {
            constant,
            prg_rom_size,
            chr_rom_size,
            flags6,
            flags7,
            flags8,
            flags9,
            flags10,
            unused,
        }
    }
}

pub struct Rom {
    header: Header,
    program: Vec<u8>,
    charactor: Vec<u8>,
}

impl Rom {
    pub fn new(rom_data: &Vec<u8>) -> Rom {
        let mut header = [0; 16];
        header.copy_from_slice(&rom_data[0..16]);
        let header = Header::new(&header);

        let program_start = 16;
        let program_end = program_start + 0x4000 * header.prg_rom_size as usize;
        let program = &rom_data[program_start..program_end];

        let character_start = program_end;
        let character_end = character_start + 0x2000 * header.chr_rom_size as usize;
        let character = &rom_data[character_start..character_end];

        Rom {
            header,
            program: program.to_vec(),
            charactor: character.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::io::Result;

    fn read_rom(path: &str) -> Result<Vec<u8>> {
        use std::fs::File;

        let f = File::open(path)?;

        let mut rom_bytes: Vec<u8> = Vec::new();
        for byte in f.bytes() {
            rom_bytes.push(byte.unwrap());
        }
        Ok(rom_bytes)
    }

    #[test]
    fn test_rom_new() {
        let rom = read_rom("../rom/sample1.nes").unwrap();
        let rom = Rom::new(&rom);
        assert_eq!([0x4e, 0x45, 0x53, 0x1a], rom.header.constant);
        assert_eq!(0x02, rom.header.prg_rom_size);
        assert_eq!(0x01, rom.header.chr_rom_size);
        assert_eq!([0x78, 0xa2], rom.program[0..2]);
        assert_eq!([0x00, 0x00], rom.charactor[0..2]);
    }
}
