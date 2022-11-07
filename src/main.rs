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

fn main() -> Result<()> {
    use nes_core::Nes;

    let rom = read_rom("./rom/sample1.nes")?;
    let nes = Nes::new(rom);
    Ok(())
}
