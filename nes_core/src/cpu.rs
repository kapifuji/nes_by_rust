use crate::opcode::{create_opcodes_map, Instruction, Opcode};
use std::collections::HashMap;

#[derive(Default)]
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

struct CpuRegister {
    /// Accumulator
    a: u8,
    /// Index Register X
    x: u8,
    /// Index Register Y
    y: u8,
    /// Processor status
    p: StatusRegister,
    /// Stack Pointer
    sp: u8,
    /// Program Counter
    pc: u16,
}
impl Default for CpuRegister {
    fn default() -> Self {
        Self {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            p: StatusRegister::new(),
            sp: 0xfd,
            pc: 0x00,
        }
    }
}
impl CpuRegister {
    pub fn new() -> Self {
        Default::default()
    }
}

struct StatusRegister {
    /// bit7 negative
    /// 演算結果が1のときセット
    n: bool,
    /// bit6 overflow
    /// 演算結果がオーバーフローを起こした時にセット
    v: bool,
    /// bit5 reserved
    /// 常にセット
    r: bool,
    /// bit4 break mode
    /// BRK発生時にセット、IRQ発生時にクリア
    b: bool,
    /// bit3 decimal mode
    /// 0: デフォルト、1: BCDモード
    d: bool,
    /// bit2 not allowed IRQ
    /// false: IRQ許可、true: IRQ禁止
    i: bool,
    /// bit1 zero
    /// 演算結果が0の時にセット
    z: bool,
    /// bit0 carry
    /// キャリー発生時にセット
    c: bool,
}
impl Default for StatusRegister {
    fn default() -> Self {
        Self {
            n: false,
            v: false,
            r: true,
            b: true,
            d: false,
            i: true,
            z: false,
            c: false,
        }
    }
}
impl StatusRegister {
    pub fn new() -> Self {
        Default::default()
    }
}

struct CpuMemory {
    /// 0x0000 ~ 0x07FF (0x0100 ~ 0x01ff is stack)
    wram: [u8; 0x800],
    /// 0x2000 ~ 0x2007
    ppu_register: PpuRegister,
    /// 0x8000 ~ 0xFFFF
    prg_rom: [u8; 0x8000],
}
impl CpuMemory {
    pub fn new(prg_rom_data: &Vec<u8>) -> Self {
        let wram: [u8; 0x800] = [0; 0x800];
        let ppu_register = PpuRegister::default();
        let mut prg_rom: [u8; 0x8000] = [0; 0x8000];

        for i in 0..prg_rom.len() {
            if i >= prg_rom_data.len() {
                break;
            }
            prg_rom[i] = prg_rom_data[i];
        }

        Self {
            wram,
            ppu_register,
            prg_rom,
        }
    }

    pub fn read_memory_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x7fe => self.wram[address as usize],
            0x2000..=0x2006 => todo!(),
            0x8000..=0xfffe => self.prg_rom[(address - 0x8000) as usize],
            _ => panic!("unexpected address: {}", address),
        }
    }

    pub fn read_memory_word(&self, address: u16) -> u16 {
        match address {
            0..=0x7fe => {
                let lo = self.wram[address as usize];
                let hi = self.wram[(address as usize) + 1];
                (lo as u16) + ((hi as u16) << 8)
            }
            0x2000..=0x2007 => todo!(),
            0x8000..=0xfffe => {
                let index = address - 0x8000;
                let lo = self.prg_rom[index as usize];
                let hi = self.prg_rom[(index as usize) + 1];
                (lo as u16) + ((hi as u16) << 8)
            }
            _ => panic!("unexpected address: {}", address),
        }
    }

    pub fn write_memory_byte(&mut self, address: u16, value: u8) {
        let write_ref = match address {
            0..=0x7fe => &mut self.wram[address as usize],
            0x2000..=0x2006 => todo!(),
            0x8000..=0xfffe => &mut self.prg_rom[(address - 0x8000) as usize],
            _ => panic!("unexpected address: {}", address),
        };
        *write_ref = value;
    }

    pub fn write_memory_word(&mut self, address: u16, value: u16) {
        let lo = (value & 0x00ff) as u8;
        let hi = ((value & 0xff00) >> 8) as u8;

        match address {
            0..=0x7fe => {
                self.wram[address as usize] = lo;
                self.wram[(address as usize) + 1] = hi;
            }
            0x2000..=0x2007 => todo!(),
            0x8000..=0xfffe => {
                let index = address - 0x8000;
                self.prg_rom[index as usize] = lo;
                self.prg_rom[(index as usize) + 1] = hi;
            }
            _ => panic!("unexpected address: {}", address),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

pub struct Cpu {
    register: CpuRegister,
    memory_map: CpuMemory,
    opcodes: HashMap<u8, Opcode>,
}

impl Cpu {
    pub fn new(prg_rom_data: &Vec<u8>) -> Cpu {
        let register = CpuRegister::new();
        let memory_map = CpuMemory::new(&prg_rom_data);
        let opcodes = create_opcodes_map();

        Self {
            register,
            memory_map,
            opcodes,
        }
    }

    pub fn tick() {
        todo!()
    }

    pub fn get_ppu_register() -> PpuRegister {
        todo!()
    }

    pub fn reset(&mut self) {
        self.register = CpuRegister::new();
        self.register.pc = self.memory_map.read_memory_word(0xfffc);
    }

    /// アドレッシングモードに応じたアドレスを返します。
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.register.pc,
            AddressingMode::ZeroPage => self.memory_map.read_memory_byte(self.register.pc) as u16,
            AddressingMode::ZeroPageX => {
                let base = self.memory_map.read_memory_byte(self.register.pc);
                base.wrapping_add(self.register.x) as u16
            }
            AddressingMode::ZeroPageY => {
                let base = self.memory_map.read_memory_byte(self.register.pc);
                base.wrapping_add(self.register.y) as u16
            }
            AddressingMode::Relative => self.register.pc,
            AddressingMode::Absolute => self.memory_map.read_memory_word(self.register.pc),
            AddressingMode::AbsoluteX => {
                let base = self.memory_map.read_memory_word(self.register.pc);
                let lo = ((base & 0x00ff) as u8).wrapping_add(self.register.x);
                (base & 0xff00) + lo as u16
            }
            AddressingMode::AbsoluteY => {
                let base = self.memory_map.read_memory_word(self.register.pc);
                let lo = ((base & 0x00ff) as u8).wrapping_add(self.register.y);
                (base & 0xff00) + lo as u16
            }
            AddressingMode::Indirect => todo!("not impl"),
            AddressingMode::IndirectX => {
                let base = self.memory_map.read_memory_byte(self.register.pc);
                let address = base.wrapping_add(self.register.x);
                self.memory_map.read_memory_word(address as u16)
            }
            AddressingMode::IndirectY => {
                let base = self.memory_map.read_memory_byte(self.register.pc);
                let address = self.memory_map.read_memory_word(base as u16);

                let lo = ((address & 0x00ff) as u8).wrapping_add(self.register.y);
                (address & 0xff00) + lo as u16
            }
            AddressingMode::NoneAddressing | AddressingMode::Accumulator => {
                panic!("{:?} is not supported", mode);
            }
        }
    }

    /// is_adc @ true: adc, false: sbc
    fn adc_sbc_sub(&mut self, mode: &AddressingMode, is_adc: bool) {
        let address = self.get_operand_address(mode);
        let value = if is_adc == true {
            self.memory_map.read_memory_byte(address)
        } else {
            255 - self.memory_map.read_memory_byte(address)
        };

        let old_a = self.register.a;

        let ret = self
            .register
            .a
            .overflowing_add(value + if self.register.p.c == true { 1 } else { 0 });

        // 演算結果
        self.register.a = ret.0;

        // update z, n
        self.update_zero_and_negative_flags(self.register.a);

        // update c
        if is_adc == true {
            if ret.1 == true {
                self.register.p.c = true;
            }
        } else {
            if ret.1 == false {
                self.register.p.c = false;
            }
        }

        // update v (ref. http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html)
        if ((old_a ^ ret.0) & (value ^ ret.0) & 0x80) != 0 {
            self.register.p.v = true;
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        self.adc_sbc_sub(mode, true);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.memory_map.read_memory_byte(address);

        self.register.a &= value;

        self.update_zero_and_negative_flags(self.register.a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let value = if *mode == AddressingMode::Accumulator {
            self.register.a
        } else {
            let address = self.get_operand_address(mode);
            self.memory_map.read_memory_byte(address)
        };

        self.register.a = value << 1;

        self.register.p.c = if (value & 0x80) == 0 { false } else { true };
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let offset = self.memory_map.read_memory_byte(address);

        if self.register.p.c == false {
            if offset >= 0x80 {
                self.register.pc += (offset - 0x80) as u16;
            } else {
                self.register.pc -= offset as u16;
            }
        }
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let offset = self.memory_map.read_memory_byte(address);

        if self.register.p.c == true {
            if offset >= 0x80 {
                self.register.pc += (offset - 0x80) as u16;
            } else {
                self.register.pc -= offset as u16;
            }
        }
    }

    fn beq(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let offset = self.memory_map.read_memory_byte(address);

        if self.register.p.z == true {
            if offset >= 0x80 {
                self.register.pc += (offset - 0x80) as u16;
            } else {
                self.register.pc -= offset as u16;
            }
        }
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let offset = self.memory_map.read_memory_byte(address);

        if self.register.p.n == true {
            if offset >= 0x80 {
                self.register.pc += (offset - 0x80) as u16;
            } else {
                self.register.pc -= offset as u16;
            }
        }
    }

    fn bne(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let offset = self.memory_map.read_memory_byte(address);

        if self.register.p.z == false {
            if offset >= 0x80 {
                self.register.pc += (offset - 0x80) as u16;
            } else {
                self.register.pc -= offset as u16;
            }
        }
    }

    fn inx(&mut self) {
        self.register.x = self.register.x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.memory_map.read_memory_byte(address);

        self.register.a = value;
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        self.adc_sbc_sub(mode, false);
    }

    fn sec(&mut self) {
        self.register.p.c = true;
    }

    fn sed(&mut self) {
        self.register.p.d = true;
    }

    fn sei(&mut self) {
        self.register.p.i = true;
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory_map.write_memory_byte(address, self.register.a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory_map.write_memory_byte(address, self.register.x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory_map.write_memory_byte(address, self.register.y);
    }

    fn tax(&mut self) {
        self.register.x = self.register.a;

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn tay(&mut self) {
        self.register.y = self.register.a;

        self.update_zero_and_negative_flags(self.register.y);
    }

    fn tsx(&mut self) {
        self.register.x = self.register.sp;

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn txa(&mut self) {
        self.register.a = self.register.x;

        self.update_zero_and_negative_flags(self.register.a);
    }

    fn txs(&mut self) {
        self.register.sp = self.register.x;
    }

    fn tya(&mut self) {
        self.register.a = self.register.y;

        self.update_zero_and_negative_flags(self.register.a);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.register.p.z = if result == 0 { true } else { false };

        self.register.p.n = if (result & 0b1000_0000) != 0 {
            true
        } else {
            false
        };
    }

    /// 命令テスト用関数
    pub fn interpret(&mut self) {
        self.register.pc = 0x8000;

        loop {
            let code = self.memory_map.read_memory_byte(self.register.pc);
            self.register.pc += 1;

            let opcode = self
                .opcodes
                .get(&code)
                .expect(&format!("{} is not recognized", code))
                .clone();

            match opcode.instruction {
                Instruction::ADC => self.adc(&opcode.addressing_mode),
                Instruction::AND => self.and(&opcode.addressing_mode),
                Instruction::ASL => self.asl(&opcode.addressing_mode),
                Instruction::BCC => self.bcc(&opcode.addressing_mode),
                Instruction::BCS => self.bcs(&opcode.addressing_mode),
                Instruction::BEQ => self.beq(&opcode.addressing_mode),
                Instruction::BMI => self.bmi(&opcode.addressing_mode),
                Instruction::BNE => self.bne(&opcode.addressing_mode),
                Instruction::BRK => return,
                Instruction::INX => self.inx(),
                Instruction::LDA => self.lda(&opcode.addressing_mode),
                Instruction::SBC => self.sbc(&opcode.addressing_mode),
                Instruction::SEC => self.sec(),
                Instruction::SED => self.sed(),
                Instruction::SEI => self.sei(),
                Instruction::STA => self.sta(&opcode.addressing_mode),
                Instruction::STX => self.stx(&opcode.addressing_mode),
                Instruction::STY => self.sty(&opcode.addressing_mode),
                Instruction::TAX => self.tax(),
                Instruction::TAY => self.tay(),
                Instruction::TSX => self.tsx(),
                Instruction::TXA => self.txa(),
                Instruction::TXS => self.txs(),
                Instruction::TYA => self.tya(),
                _ => todo!("not impl"),
            }

            self.register.pc += opcode.bytes as u16 - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_new() {
        let mut prg_rom: Vec<u8> = Vec::new();
        for i in 0..5 {
            prg_rom.push(i);
        }
        let cpu = Cpu::new(&prg_rom);

        assert_eq!(cpu.register.sp, 0xfd);
        assert_eq!(cpu.memory_map.prg_rom[0..5], [0x00, 0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_cpu_reset() {
        let mut prg_rom: Vec<u8> = Vec::new();
        for i in 0u16..0x8000 {
            prg_rom.push((i % 0x100) as u8);
        }
        let mut cpu = Cpu::new(&prg_rom);

        cpu.reset();

        assert_eq!(cpu.register.pc, 0xfdfc);
    }

    #[test]
    fn test_0x69_adc_addtion() {
        let program = vec![0x69, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0x50;
        cpu.register.p.c = true;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x61);
        assert_eq!(cpu.register.p.c, true);
        assert_eq!(cpu.register.p.v, false);
    }

    #[test]
    fn test_0x69_adc_overflow1() {
        let program = vec![0x69, 0x50, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0x50;
        cpu.register.p.c = true;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xa1);
        assert_eq!(cpu.register.p.c, true);
        assert_eq!(cpu.register.p.v, true);
    }

    #[test]
    fn test_0x69_adc_overflow2() {
        let program = vec![0x69, 0x90, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xd0;
        cpu.register.p.c = false;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x60);
        assert_eq!(cpu.register.p.c, true);
        assert_eq!(cpu.register.p.v, true);
    }

    #[test]
    fn test_0x29_and() {
        let program = vec![0x29, 0xf0, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x25_and() {
        let program = vec![0x25, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.memory_map.write_memory_byte(0x0010, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x35_and() {
        let program = vec![0x35, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.register.x = 0xff;
        cpu.memory_map.write_memory_byte(0x000f, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x2d_and() {
        let program = vec![0x2d, 0x12, 0x05, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.memory_map.write_memory_byte(0x0512, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x3d_and() {
        let program = vec![0x3d, 0x12, 0x05, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.register.x = 0xff;
        cpu.memory_map.write_memory_byte(0x0511, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x39_and() {
        let program = vec![0x39, 0x12, 0x05, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.register.y = 0xff;
        cpu.memory_map.write_memory_byte(0x0511, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x21_and() {
        let program = vec![0x21, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.register.x = 0xff;
        cpu.memory_map.write_memory_word(0x000f, 0x0512);
        cpu.memory_map.write_memory_byte(0x0512, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x31_and() {
        let program = vec![0x31, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xff;
        cpu.register.y = 0xff;
        cpu.memory_map.write_memory_word(0x0010, 0x0512);
        cpu.memory_map.write_memory_byte(0x0511, 0xf0);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x0a_asl() {
        let program = vec![0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0b1100_1111;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b1001_1110);
        assert_eq!(cpu.register.p.c, true);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0x06_asl() {
        let program = vec![0x06, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.memory_map.write_memory_byte(0x0010, 0b0000_1111);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0001_1110);
        assert_eq!(cpu.register.p.c, false);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, false);
    }

    #[test]
    fn test_bcc_branch() {
        let program = vec![0x90, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.c = false;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bcc_not_branch() {
        let program = vec![0x90, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.c = true;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bcs_branch() {
        let program = vec![0xb0, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.c = true;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bcs_not_branch() {
        let program = vec![0xb0, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.c = false;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_beq_branch() {
        let program = vec![0xf0, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.z = true;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_beq_not_branch() {
        let program = vec![0xf0, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.z = false;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bmi_branch() {
        let program = vec![0x30, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.n = true;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bmi_not_branch() {
        let program = vec![0x30, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.n = false;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bne_branch() {
        let program = vec![0xd0, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.z = false;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bne_not_branch() {
        let program = vec![0xd0, 0x81, 0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.p.z = true;
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_brk() {
        let program = vec![0x00, 0x0a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0b0000_0001;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_inx_overflow() {
        let program = vec![0xe8, 0xe8, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.x = 0xff;
        cpu.interpret();

        assert_eq!(cpu.register.x, 1)
    }

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let program = vec![0xa9, 0x05, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x05);
        assert_eq!(cpu.register.p.z, false);
        assert_eq!(cpu.register.p.n, false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let program = vec![0xa9, 0x00, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.p.z, true);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let program = vec![0xa9, 0b1000_0000, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.p.n, true);
    }

    #[test]
    fn test_0xe9_sbc_subtraction() {
        let program = vec![0xe9, 0x40, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0x50;
        cpu.register.p.c = true;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x10);
        assert_eq!(cpu.register.p.c, true);
        assert_eq!(cpu.register.p.v, false);
    }

    #[test]
    fn test_0xe9_sbc_overflow1() {
        let program = vec![0xe9, 0xb0, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0x50;
        cpu.register.p.c = true;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0xa0);
        assert_eq!(cpu.register.p.c, false);
        assert_eq!(cpu.register.p.v, true);
    }

    #[test]
    fn test_0xe9_sbc_overflow2() {
        let program = vec![0xe9, 0x70, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 0xd0;
        cpu.register.p.c = false;
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x5f);
        assert_eq!(cpu.register.p.c, false);
        assert_eq!(cpu.register.p.v, true);
    }

    #[test]
    fn test_sec_set_carry() {
        let program = vec![0x38, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.p.c, true);
    }

    #[test]
    fn test_sed_set_decimal() {
        let program = vec![0xf8, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.p.d, true);
    }

    #[test]
    fn test_sei_disable_interrupt() {
        let program = vec![0x78, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.p.i, true);
    }

    #[test]
    fn test_lda_from_memory() {
        let program = vec![0xa5, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.memory_map.write_memory_byte(0x0010, 0x55);
        cpu.interpret();

        assert_eq!(cpu.register.a, 0x55);
    }

    #[test]
    fn test_0x85_sta_store_a() {
        let program = vec![0x85, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 100;
        cpu.interpret();

        assert_eq!(cpu.memory_map.read_memory_byte(0x0010), 100);
    }

    #[test]
    fn test_0x86_stx_store_x() {
        let program = vec![0x86, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.x = 100;
        cpu.interpret();

        assert_eq!(cpu.memory_map.read_memory_byte(0x0010), 100);
    }

    #[test]
    fn test_0x84_sty_store_y() {
        let program = vec![0x84, 0x10, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.y = 100;
        cpu.interpret();

        assert_eq!(cpu.memory_map.read_memory_byte(0x0010), 100);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let program = vec![0xaa, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 10;
        cpu.interpret();

        assert_eq!(cpu.register.x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.interpret();

        assert_eq!(cpu.register.x, 0xc1)
    }

    #[test]
    fn test_tay_move_a_to_y() {
        let program = vec![0xa8, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.a = 20;
        cpu.interpret();

        assert_eq!(cpu.register.y, 20);
    }

    #[test]
    fn test_tsx_move_sp_to_x() {
        let program = vec![0xba, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.sp = 0x50;
        cpu.interpret();

        assert_eq!(cpu.register.x, 0x50);
    }

    #[test]
    fn test_txa_move_x_to_a() {
        let program = vec![0x8a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.x = 20;
        cpu.interpret();

        assert_eq!(cpu.register.a, 20);
    }

    #[test]
    fn test_txs_move_x_to_sp() {
        let program = vec![0x9a, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.x = 0x50;
        cpu.interpret();

        assert_eq!(cpu.register.sp, 0x50);
    }

    #[test]
    fn test_tya_move_y_to_a() {
        let program = vec![0x98, 0x00];
        let mut cpu = Cpu::new(&program);
        cpu.register.y = 20;
        cpu.interpret();

        assert_eq!(cpu.register.a, 20);
    }
}
