use crate::{
    bus::Bus,
    opcode::{create_opcodes_map, Instruction, Opcode},
    ppu::Ppu,
    rom::Rom,
};
use bitflags::bitflags;
use std::collections::HashMap;

const BIT0: u8 = 0b0000_0001;
const BIT1: u8 = 0b0000_0010;
const BIT2: u8 = 0b0000_0100;
const BIT3: u8 = 0b0000_1000;
const BIT4: u8 = 0b0001_0000;
const BIT5: u8 = 0b0010_0000;
const BIT6: u8 = 0b0100_0000;
const BIT7: u8 = 0b1000_0000;

pub struct CpuRegister {
    /// Accumulator
    pub a: u8,
    /// Index Register X
    pub x: u8,
    /// Index Register Y
    pub y: u8,
    /// Processor status
    pub p: StatusRegister,
    /// Stack Pointer
    pub sp: u8,
    /// Program Counter
    pub pc: u16,
}

impl Default for CpuRegister {
    fn default() -> Self {
        Self {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            p: StatusRegister::default(),
            sp: Self::DEFAULT_SP,
            pc: 0x00,
        }
    }
}
impl CpuRegister {
    const DEFAULT_SP: u8 = 0xfd;

    pub fn new() -> Self {
        Default::default()
    }
}

bitflags! {
    pub struct StatusRegister: u8{
        /// bit7 negative
        /// 演算結果が1のときセット
        const n = BIT7;
        /// bit6 overflow
        /// 演算結果がオーバーフローを起こした時にセット
        const v = BIT6;
        /// bit5 reserved
        /// 常にセット
        const r = BIT5;
        /// bit4 break mode
        /// BRK発生時にセット、IRQ発生時にクリア
        const b = BIT4;
        /// bit3 decimal mode
        /// 0: デフォルト、1: BCDモード
        const d = BIT3;
        /// bit2 not allowed IRQ
        /// false: IRQ許可、true: IRQ禁止
        const i = BIT2;
        /// bit1 zero
        /// 演算結果が0の時にセット
        const z = BIT1;
        /// bit0 carry
        /// キャリー発生時にセット
        const c = BIT0;
    }
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self::r | Self::i
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

pub struct Cpu<'a> {
    pub register: CpuRegister,
    pub bus: Bus<'a>,
    pub opcodes: HashMap<u8, Opcode>,
}

impl<'a> Cpu<'a> {
    const DEFAULT_PC_ADDRESS: u16 = 0xfffc;
    const STACK_BASE_ADDRESS: u16 = 0x0100;

    pub fn new<'b>(mut bus: Bus<'b>) -> Cpu<'b> {
        let mut register = CpuRegister::new();
        let opcodes = create_opcodes_map();

        register.pc = bus.read_memory_word(Self::DEFAULT_PC_ADDRESS);

        Cpu {
            register,
            bus,
            opcodes,
        }
    }

    pub fn read_memory_byte(&mut self, address: u16) -> u8 {
        self.bus.read_memory_byte(address)
    }

    pub fn read_memory_word(&mut self, address: u16) -> u16 {
        self.bus.read_memory_word(address)
    }

    pub fn write_memory_byte(&mut self, address: u16, value: u8) {
        self.bus.write_memory_byte(address, value)
    }

    pub fn write_memory_word(&mut self, address: u16, value: u16) {
        self.bus.write_memory_word(address, value)
    }

    fn stack_push_byte(&mut self, value: u8) {
        let sp = Self::STACK_BASE_ADDRESS + self.register.sp as u16;
        self.write_memory_byte(sp, value);
        self.register.sp -= 1;
    }

    fn stack_push_word(&mut self, value: u16) {
        self.register.sp -= 1;
        let sp = Self::STACK_BASE_ADDRESS + self.register.sp as u16;
        self.write_memory_word(sp, value);
        self.register.sp -= 1;
    }

    fn stack_pop_byte(&mut self) -> u8 {
        self.register.sp += 1;

        let sp = Self::STACK_BASE_ADDRESS + self.register.sp as u16;
        self.read_memory_byte(sp)
    }

    fn stack_pop_word(&mut self) -> u16 {
        self.register.sp += 1;

        let sp = Self::STACK_BASE_ADDRESS + self.register.sp as u16;
        let result = self.read_memory_word(sp);

        self.register.sp += 1;

        result
    }

    pub fn reset(&mut self) {
        self.register = CpuRegister::new();
        self.register.pc = self.read_memory_word(Self::DEFAULT_PC_ADDRESS);
    }

    fn interrupt_nmi(&mut self) {
        self.stack_push_word(self.register.pc);

        let mut status = self.register.p.bits();
        status &= !BIT4;
        status |= BIT5;

        self.stack_push_byte(status);
        self.register.p = StatusRegister::from_bits(self.register.p.bits() | BIT2).unwrap();

        self.bus.tick(2);
        self.register.pc = self.read_memory_word(0xfffa);
    }

    /// アドレッシングモードに応じたアドレスを返します。
    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.register.pc,
            AddressingMode::ZeroPage => self.read_memory_byte(self.register.pc) as u16,
            AddressingMode::ZeroPageX => {
                let base = self.read_memory_byte(self.register.pc);
                base.wrapping_add(self.register.x) as u16
            }
            AddressingMode::ZeroPageY => {
                let base = self.read_memory_byte(self.register.pc);
                base.wrapping_add(self.register.y) as u16
            }
            AddressingMode::Relative => self.register.pc,
            AddressingMode::Absolute => self.read_memory_word(self.register.pc),
            AddressingMode::AbsoluteX => {
                let base = self.read_memory_word(self.register.pc);
                base.wrapping_add(self.register.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let base = self.read_memory_word(self.register.pc);
                base.wrapping_add(self.register.y as u16)
            }
            AddressingMode::Indirect => {
                let address = self.read_memory_word(self.register.pc);

                // self.read_memory_word(address as u16)
                // 仕様上は上の操作だけで良いが、エミュレート元の6502にバグがある。
                // ページ境界を跨ぐ時に、メモリの参照がおかしくなる。
                // 例えば、30FFが指定された時、lo: 30FF, hi: 3000 として参照する。
                if address & 0x00ff == 0x00ff {
                    let lo = self.read_memory_byte(address);
                    let hi = self.read_memory_byte(address & 0xff00);
                    ((hi as u16) << 8) + lo as u16
                } else {
                    self.read_memory_word(address as u16)
                }
            }
            AddressingMode::IndirectX => {
                let base = self.read_memory_byte(self.register.pc);
                let address = base.wrapping_add(self.register.x);

                // ROMから得た値にXレジスタの値を足したものがベースアドレス（0x00 ~ 0xFF）
                // そこから2byte分のデータを取るが、0xFFを超える場合は0x00に戻す。
                let lo = self.read_memory_byte(address as u16);
                let hi = self.read_memory_byte(address.wrapping_add(1) as u16);

                ((hi as u16) << 8) + lo as u16
            }
            AddressingMode::IndirectY => {
                let base = self.read_memory_byte(self.register.pc);

                // ROMから得た値がベースアドレス（u8）
                // そこから2byte分のデータを取るが、0xFFを超える場合は0x00に戻す。
                let address_lo = self.read_memory_byte(base as u16);
                let address_hi = self.read_memory_byte(base.wrapping_add(1) as u16);

                // 取得したデータの下位、上位を結合（u16）
                // そこにレジスタYの値を加算するが、0xFFFFを超える場合は0x0000に戻す。
                let data = ((address_hi as u16) << 8) + address_lo as u16;
                data.wrapping_add(self.register.y as u16)
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
            self.read_memory_byte(address)
        } else {
            255 - self.read_memory_byte(address)
        };

        let old_a = self.register.a;

        let ret = self.register.a.overflowing_add(value);

        let over = ret.1;

        let ret = ret
            .0
            .overflowing_add(if self.register.p.contains(StatusRegister::c) == true {
                1
            } else {
                0
            });

        let over = over | ret.1;

        // 演算結果
        self.register.a = ret.0;

        // update z, n
        self.update_zero_and_negative_flags(self.register.a);

        // update c
        self.register.p.set(StatusRegister::c, over);

        // update v (ref. http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html)
        if ((old_a ^ ret.0) & (value ^ ret.0) & BIT7) != 0 {
            self.register.p.insert(StatusRegister::v);
        } else {
            self.register.p.remove(StatusRegister::v);
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        self.adc_sbc_sub(mode, true);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        self.register.a &= value;

        self.update_zero_and_negative_flags(self.register.a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let old_value = if *mode == AddressingMode::Accumulator {
            self.register.a
        } else {
            let address = self.get_operand_address(mode);
            self.read_memory_byte(address)
        };

        let result = old_value << 1;

        if (old_value & BIT7) == 0 {
            self.register.p.remove(StatusRegister::c);
        } else {
            self.register.p.insert(StatusRegister::c);
        };

        if *mode == AddressingMode::Accumulator {
            self.register.a = result
        } else {
            let address = self.get_operand_address(mode);
            self.write_memory_byte(address, result);
        };

        self.update_zero_and_negative_flags(result);
    }

    fn bxx_sub(&mut self, mode: &AddressingMode, target_status: bool, trigger: bool) {
        let address = self.get_operand_address(mode);
        let offset = self.read_memory_byte(address) as i8;

        if target_status == trigger {
            self.register.pc = self.register.pc.wrapping_add(offset as u16);
        }
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::c), false);
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::c), true);
    }

    fn beq(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::z), true);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        let reault = value & self.register.a;

        self.register.p.set(StatusRegister::z, reault == 0x00);
        self.register.p.set(StatusRegister::v, value & BIT6 != 0x00);
        self.register.p.set(StatusRegister::n, value & BIT7 != 0x00);
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::n), true);
    }

    fn bne(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::z), false);
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::n), false);
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::v), false);
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        self.bxx_sub(mode, self.register.p.contains(StatusRegister::v), true);
    }

    fn clc(&mut self) {
        self.register.p.remove(StatusRegister::c);
    }

    fn cld(&mut self) {
        self.register.p.remove(StatusRegister::d);
    }

    fn cli(&mut self) {
        self.register.p.remove(StatusRegister::i);
    }

    fn clv(&mut self) {
        self.register.p.remove(StatusRegister::v);
    }

    fn cmp_sub(&mut self, mode: &AddressingMode, subtracted_value: u8) {
        let address = self.get_operand_address(mode);
        let cmp_value = self.read_memory_byte(address);

        if subtracted_value >= cmp_value {
            self.register.p.insert(StatusRegister::c);
        } else {
            self.register.p.remove(StatusRegister::c);
        };

        let result = subtracted_value.wrapping_sub(cmp_value);
        self.update_zero_and_negative_flags(result);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        self.cmp_sub(mode, self.register.a);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        self.cmp_sub(mode, self.register.x);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        self.cmp_sub(mode, self.register.y);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        let result = value.wrapping_sub(1);
        self.write_memory_byte(address, result);
        self.update_zero_and_negative_flags(result);
    }

    fn dex(&mut self) {
        self.register.x = self.register.x.wrapping_sub(1);

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn dey(&mut self) {
        self.register.y = self.register.y.wrapping_sub(1);

        self.update_zero_and_negative_flags(self.register.y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        self.register.a ^= value;
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        let result = value.wrapping_add(1);
        self.write_memory_byte(address, result);
        self.update_zero_and_negative_flags(result);
    }

    fn inx(&mut self) {
        self.register.x = self.register.x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register.x);
    }

    fn iny(&mut self) {
        self.register.y = self.register.y.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register.y);
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.register.pc = address;
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        // 命令長分+3（事前に+1済み）、RTSとの対応で-1しておく。（test ROMに合わせた実装）
        self.stack_push_word(self.register.pc + 2 - 1);

        let address = self.get_operand_address(mode);
        self.register.pc = address;
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        self.register.a = value;
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        self.register.x = value;
        self.update_zero_and_negative_flags(self.register.x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        self.register.y = value;
        self.update_zero_and_negative_flags(self.register.y);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let old_value = if *mode == AddressingMode::Accumulator {
            self.register.a
        } else {
            let address = self.get_operand_address(mode);
            self.read_memory_byte(address)
        };

        let result = old_value >> 1;

        if (old_value & 0x01) == 0 {
            self.register.p.remove(StatusRegister::c);
        } else {
            self.register.p.insert(StatusRegister::c);
        };

        if *mode == AddressingMode::Accumulator {
            self.register.a = result;
        } else {
            let address = self.get_operand_address(mode);
            self.write_memory_byte(address, result);
        };

        self.update_zero_and_negative_flags(result);
    }

    fn nop(&mut self) {
        // 何もしない。
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);

        self.register.a |= value;
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn pha(&mut self) {
        self.stack_push_byte(self.register.a);
    }

    fn php(&mut self) {
        // PHP は bit4 と BIT5 を設定した値を Push する。
        self.stack_push_byte(self.register.p.bits() | BIT4 | BIT5);
    }

    fn pla(&mut self) {
        self.register.a = self.stack_pop_byte();
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn plp(&mut self) {
        // PLP は bit4 を無視する。また、bit5は常にセット。
        let data = (self.stack_pop_byte() & (!BIT4)) | BIT5;
        self.register.p = StatusRegister::from_bits(data).unwrap();
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let old_value = if *mode == AddressingMode::Accumulator {
            self.register.a
        } else {
            let address = self.get_operand_address(mode);
            self.read_memory_byte(address)
        };

        let result = if self.register.p.contains(StatusRegister::c) == true {
            (old_value << 1) | 0x01
        } else {
            old_value << 1
        };

        if (old_value & BIT7) == 0 {
            self.register.p.remove(StatusRegister::c);
        } else {
            self.register.p.insert(StatusRegister::c);
        };

        if *mode == AddressingMode::Accumulator {
            self.register.a = result
        } else {
            let address = self.get_operand_address(mode);
            self.write_memory_byte(address, result);
        };

        self.update_zero_and_negative_flags(result);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let old_value = if *mode == AddressingMode::Accumulator {
            self.register.a
        } else {
            let address = self.get_operand_address(mode);
            self.read_memory_byte(address)
        };

        let result = if self.register.p.contains(StatusRegister::c) == true {
            (old_value >> 1) | BIT7
        } else {
            old_value >> 1
        };

        if (old_value & 0x01) == 0 {
            self.register.p.remove(StatusRegister::c);
        } else {
            self.register.p.insert(StatusRegister::c);
        };

        if *mode == AddressingMode::Accumulator {
            self.register.a = result;
        } else {
            let address = self.get_operand_address(mode);
            self.write_memory_byte(address, result);
        };

        self.update_zero_and_negative_flags(result);
    }

    fn rti(&mut self) {
        self.plp(); // pull Processor Status

        self.register.pc = self.stack_pop_word();
    }

    fn rts(&mut self) {
        // JSRで-1しているので+1（test ROMに合わせた実装）
        self.register.pc = self.stack_pop_word() + 1;
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        self.adc_sbc_sub(mode, false);
    }

    fn sec(&mut self) {
        self.register.p.insert(StatusRegister::c);
    }

    fn sed(&mut self) {
        self.register.p.insert(StatusRegister::d);
    }

    fn sei(&mut self) {
        self.register.p.insert(StatusRegister::i);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.write_memory_byte(address, self.register.a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.write_memory_byte(address, self.register.x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.write_memory_byte(address, self.register.y);
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

    fn alr(&mut self, mode: &AddressingMode) {
        self.and(mode);
        self.lsr(&AddressingMode::Accumulator);
    }

    fn anc(&mut self, mode: &AddressingMode) {
        self.and(mode);
        self.register.p.set(
            StatusRegister::c,
            self.register.p.contains(StatusRegister::z),
        );
    }

    fn arr(&mut self, mode: &AddressingMode) {
        self.and(mode);

        let old_value = self.register.a;

        let result = if self.register.p.contains(StatusRegister::c) == true {
            (old_value >> 1) | BIT7
        } else {
            old_value >> 1
        };

        let bit6 = (old_value & BIT6) == BIT6;
        let bit5 = (old_value & BIT5) == BIT5;
        self.register.p.set(StatusRegister::c, bit6);
        self.register.p.set(StatusRegister::v, bit6 ^ bit5);

        self.register.a = result;

        self.update_zero_and_negative_flags(result);
    }

    fn axs(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);
        let x_and_a = self.register.x & self.register.a;

        let result = x_and_a.wrapping_sub(value);

        self.register.x = result;

        if value <= x_and_a {
            self.register.p.insert(StatusRegister::c);
        }
        self.update_zero_and_negative_flags(result);
    }

    fn lax(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax();
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let result = self.register.a & self.register.x;

        self.write_memory_byte(address, result);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        self.dec(mode);
        self.cmp(mode);
    }

    fn isc(&mut self, mode: &AddressingMode) {
        self.inc(mode);
        self.sbc(mode);
    }

    fn rla(&mut self, mode: &AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        self.ror(mode);
        self.adc(mode);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    fn skb(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);
        // メモリリードするが、何もしない。
    }

    fn ign(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.read_memory_byte(address);
        // メモリリードするが、何もしない。
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.register.p.set(StatusRegister::z, result == 0);

        self.register
            .p
            .set(StatusRegister::n, (result & 0b1000_0000) != 0);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Cpu),
    {
        loop {
            if self.bus.poll_nmi_status() {
                self.interrupt_nmi();
            }

            callback(self);

            let code = self.read_memory_byte(self.register.pc);
            self.register.pc += 1;

            let opcode = self
                .opcodes
                .get(&code)
                .expect(&format!("{} is not recognized", code))
                .clone();

            let mut is_update_pc = true;

            match opcode.instruction {
                Instruction::ADC => self.adc(&opcode.addressing_mode),
                Instruction::AND => self.and(&opcode.addressing_mode),
                Instruction::ASL => self.asl(&opcode.addressing_mode),
                Instruction::BCC => self.bcc(&opcode.addressing_mode),
                Instruction::BCS => self.bcs(&opcode.addressing_mode),
                Instruction::BEQ => self.beq(&opcode.addressing_mode),
                Instruction::BIT => self.bit(&opcode.addressing_mode),
                Instruction::BMI => self.bmi(&opcode.addressing_mode),
                Instruction::BNE => self.bne(&opcode.addressing_mode),
                Instruction::BPL => self.bpl(&opcode.addressing_mode),
                Instruction::BRK => return,
                Instruction::BVC => self.bvc(&opcode.addressing_mode),
                Instruction::BVS => self.bvs(&opcode.addressing_mode),
                Instruction::CLC => self.clc(),
                Instruction::CLD => self.cld(),
                Instruction::CLI => self.cli(),
                Instruction::CLV => self.clv(),
                Instruction::CMP => self.cmp(&opcode.addressing_mode),
                Instruction::CPX => self.cpx(&opcode.addressing_mode),
                Instruction::CPY => self.cpy(&opcode.addressing_mode),
                Instruction::DEC => self.dec(&opcode.addressing_mode),
                Instruction::DEX => self.dex(),
                Instruction::DEY => self.dey(),
                Instruction::EOR => self.eor(&opcode.addressing_mode),
                Instruction::INC => self.inc(&opcode.addressing_mode),
                Instruction::INX => self.inx(),
                Instruction::INY => self.iny(),
                Instruction::JMP => {
                    self.jmp(&opcode.addressing_mode);
                    is_update_pc = false; // PCを動かさない。
                }
                Instruction::JSR => {
                    self.jsr(&opcode.addressing_mode);
                    is_update_pc = false; // PCを動かさない。
                }
                Instruction::LDA => self.lda(&opcode.addressing_mode),
                Instruction::LDX => self.ldx(&opcode.addressing_mode),
                Instruction::LDY => self.ldy(&opcode.addressing_mode),
                Instruction::LSR => self.lsr(&opcode.addressing_mode),
                Instruction::NOP => self.nop(),
                Instruction::ORA => self.ora(&opcode.addressing_mode),
                Instruction::PHA => self.pha(),
                Instruction::PHP => self.php(),
                Instruction::PLA => self.pla(),
                Instruction::PLP => self.plp(),
                Instruction::ROL => self.rol(&opcode.addressing_mode),
                Instruction::ROR => self.ror(&opcode.addressing_mode),
                Instruction::RTI => {
                    self.rti();
                    is_update_pc = false; // PCを動かさない。
                }
                Instruction::RTS => {
                    self.rts();
                    is_update_pc = false; // PCを動かさない。
                }
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
                Instruction::ALR => self.alr(&opcode.addressing_mode),
                Instruction::ANC => self.anc(&opcode.addressing_mode),
                Instruction::ARR => self.arr(&opcode.addressing_mode),
                Instruction::AXS => self.axs(&opcode.addressing_mode),
                Instruction::LAX => self.lax(&opcode.addressing_mode),
                Instruction::SAX => self.sax(&opcode.addressing_mode),
                Instruction::DCP => self.dcp(&opcode.addressing_mode),
                Instruction::ISC => self.isc(&opcode.addressing_mode),
                Instruction::RLA => self.rla(&opcode.addressing_mode),
                Instruction::RRA => self.rra(&opcode.addressing_mode),
                Instruction::SLO => self.slo(&opcode.addressing_mode),
                Instruction::SRE => self.sre(&opcode.addressing_mode),
                Instruction::SKB => self.skb(&opcode.addressing_mode),
                Instruction::IGN => self.ign(&opcode.addressing_mode),
            }

            self.bus.tick(opcode.cycles);

            if is_update_pc {
                self.register.pc += opcode.bytes as u16 - 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bus(program_data: &Vec<u8>) -> Bus {
        use crate::rom::Header;
        use crate::rom::Rom;

        let header = Header::default();
        let mut program = vec![0; 0x8000];
        let charactor = vec![0; 0x2000];

        let program_size = if program_data.len() > 0x8000 {
            0x8000
        } else {
            program_data.len()
        };

        for i in 0..program_size {
            program[i] = program_data[i];
        }

        program[0xfffc - 0x8000] = 0x00;
        program[0xfffd - 0x8000] = 0x80;

        let rom = Rom {
            header,
            program,
            charactor,
        };
        Bus::new(&rom, |_| {})
    }

    #[test]
    fn test_cpu_new() {
        let mut program: Vec<u8> = Vec::new();
        for i in 0..5 {
            program.push(i);
        }
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);

        assert_eq!(cpu.register.sp, 0xfd);
        assert_eq!(cpu.bus.read_memory_byte(0x8002), 0x02);
    }

    #[test]
    fn test_cpu_reset() {
        let mut program: Vec<u8> = Vec::new();
        for i in 0u16..0x8000 {
            program.push((i % 0x100) as u8);
        }
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.pc = 0x0000;

        cpu.reset();

        assert_eq!(cpu.register.pc, 0x8000);
    }

    #[test]
    fn test_0x69_adc_addtion() {
        let program = vec![0x69, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x50;
        cpu.register.p.insert(StatusRegister::c);
        cpu.register.p.insert(StatusRegister::v);
        cpu.run();

        assert_eq!(cpu.register.a, 0x61);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), false);
    }

    #[test]
    fn test_0x69_adc_overflow1() {
        let program = vec![0x69, 0x50, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x50;
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0xa1);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), true);
    }

    #[test]
    fn test_0x69_adc_overflow2() {
        let program = vec![0x69, 0x90, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xd0;
        cpu.register.p.remove(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0x60);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), true);
    }

    #[test]
    fn test_0x29_and() {
        let program = vec![0x29, 0xf0, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x25_and() {
        let program = vec![0x25, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.write_memory_byte(0x0010, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x35_and() {
        let program = vec![0x35, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.register.x = 0xff;
        cpu.write_memory_byte(0x000f, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x2d_and() {
        let program = vec![0x2d, 0x12, 0x05, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.write_memory_byte(0x0512, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x3d_and() {
        let program = vec![0x3d, 0x12, 0x05, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.register.x = 0xff;
        cpu.write_memory_byte(0x0611, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x39_and() {
        let program = vec![0x39, 0x12, 0x05, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.register.y = 0xff;
        cpu.write_memory_byte(0x0611, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x21_and() {
        let program = vec![0x21, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.register.x = 0xff;
        cpu.write_memory_word(0x000f, 0x0512);
        cpu.write_memory_byte(0x0512, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x31_and() {
        let program = vec![0x31, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.register.y = 0xff;
        cpu.write_memory_word(0x0010, 0x0512);
        cpu.write_memory_byte(0x0611, 0xf0);
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x0a_asl() {
        let program = vec![0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0b1100_1111;
        cpu.run();

        assert_eq!(cpu.register.a, 0b1001_1110);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x06_asl() {
        let program = vec![0x06, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0010, 0b0000_1111);
        cpu.run();

        let result = cpu.read_memory_byte(0x0010);
        assert_eq!(result, 0b0001_1110);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_bcc_branch() {
        let program = vec![0x90, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::c);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bcc_not_branch() {
        let program = vec![0x90, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::c);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bcs_branch() {
        let program = vec![0xb0, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::c);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bcs_not_branch() {
        let program = vec![0xb0, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::c);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_beq_branch() {
        let program = vec![0xf0, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::z);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_beq_not_branch() {
        let program = vec![0xf0, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::z);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_0x24_bit() {
        let program = vec![0x24, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x10, 0b1100_0000);
        cpu.register.a = 0x00;
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_bmi_branch() {
        let program = vec![0x30, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::n);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bmi_not_branch() {
        let program = vec![0x30, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::n);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bne_branch() {
        let program = vec![0xd0, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::z);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bne_not_branch() {
        let program = vec![0xd0, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::z);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bpl_branch() {
        let program = vec![0x10, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::n);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bpl_not_branch() {
        let program = vec![0x10, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::n);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_brk() {
        let program = vec![0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bvc_branch() {
        let program = vec![0x50, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::v);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bvc_not_branch() {
        let program = vec![0x50, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::v);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_bvs_branch() {
        let program = vec![0x70, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::v);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0010);
    }

    #[test]
    fn test_bvs_not_branch() {
        let program = vec![0x70, 0x01, 0x00, 0x0a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.remove(StatusRegister::v);
        cpu.register.a = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0000_0001);
    }

    #[test]
    fn test_clc() {
        let program = vec![0x18, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
    }

    #[test]
    fn test_cld() {
        let program = vec![0xd8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::d);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::d), false);
    }

    #[test]
    fn test_cli() {
        let program = vec![0x58, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::i);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::i), false);
    }

    #[test]
    fn test_clv() {
        let program = vec![0xb8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p.insert(StatusRegister::v);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::v), false);
    }

    #[test]
    fn test_0xc9_cmp_carry() {
        let program = vec![0xc9, 0x01, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x01;
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0xc9_cmp_not_carry() {
        let program = vec![0xc9, 0x02, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x01;
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0xe0_cpx_carry() {
        let program = vec![0xe0, 0x01, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 0x01;
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0xe0_cpx_not_carry() {
        let program = vec![0xe0, 0x02, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 0x01;
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0xc0_cpy_carry() {
        let program = vec![0xc0, 0x01, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.y = 0x01;
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0xc0_cpy_not_carry() {
        let program = vec![0xc0, 0x02, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.y = 0x01;
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0xc6_dec() {
        let program = vec![0xc6, 0x02, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0002, 0x00);
        cpu.run();

        let result = cpu.read_memory_byte(0x0002);
        assert_eq!(result, 0xff);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_dex() {
        let program = vec![0xca, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 0;
        cpu.run();

        assert_eq!(cpu.register.x, 0xff);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_dey() {
        let program = vec![0x88, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.y = 0;
        cpu.run();

        assert_eq!(cpu.register.y, 0xff);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x49_eor() {
        let program = vec![0x49, 0x0f, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xff;
        cpu.run();

        assert_eq!(cpu.register.a, 0xf0);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0xe6_inc() {
        let program = vec![0xe6, 0x02, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0002, 0xff);
        cpu.run();

        let result = cpu.read_memory_byte(0x0002);
        assert_eq!(result, 0x00);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_inx() {
        let program = vec![0xe8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 0xff;
        cpu.run();

        assert_eq!(cpu.register.x, 0x00);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_iny() {
        let program = vec![0xc8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.y = 0xff;
        cpu.run();

        assert_eq!(cpu.register.y, 0x00);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0x4c_jmp() {
        let program = vec![
            0x4c, 0x0a, 0x80, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x38, 0x00, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x00,
        ];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
    }

    #[test]
    fn test_0x6c_jmp() {
        let program = vec![
            0x6c, 0x00, 0x02, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x38, 0x00, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x00,
        ];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_word(0x0200, 0x800a);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
    }

    #[test]
    fn test_jsr() {
        let program = vec![
            0x20, 0x0a, 0x80, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x38, 0x00, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x00,
        ];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfb);
        assert_eq!(cpu.read_memory_word(0x01fc), 0x8002);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
    }

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let program = vec![0xa9, 0x05, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.a, 0x05);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let program = vec![0xa9, 0x00, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::z), true);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let program = vec![0xa9, 0b1000_0000, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0xa2_ldx() {
        let program = vec![0xa2, 0b1000_0000, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.x, 0b1000_0000);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0xa0_ldy() {
        let program = vec![0xa0, 0b1000_0000, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.y, 0b1000_0000);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x4a_lsr() {
        let program = vec![0x4a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0b1100_1111;
        cpu.run();

        assert_eq!(cpu.register.a, 0b0110_0111);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0x46_lsr() {
        let program = vec![0x46, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0010, 0b0000_1110);
        cpu.run();

        let result = cpu.read_memory_byte(0x0010);
        assert_eq!(result, 0b0000_0111);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_nop() {
        let program = vec![0xea, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();
    }

    #[test]
    fn test_0x09_ora() {
        let program = vec![0x09, 0x55, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x80;
        cpu.run();

        assert_eq!(cpu.register.a, 0xd5);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_pha() {
        let program = vec![0x48, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x80;
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfc);
        assert_eq!(cpu.read_memory_byte(0x01fd), 0x80);
    }

    #[test]
    fn test_php() {
        let program = vec![0x08, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.p = StatusRegister::from_bits(0b01000101).unwrap();
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfc);
        assert_eq!(cpu.read_memory_byte(0x01fd), 0b01110101);
    }

    #[test]
    fn test_pla() {
        let program = vec![0x68, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.sp = 0xfc;
        cpu.write_memory_word(0x01fd, 0x80);
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfd);
        assert_eq!(cpu.register.a, 0x80);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_plp() {
        let program = vec![0x28, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.sp = 0xfc;
        cpu.write_memory_word(0x01fd, 0b01010101);
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfd);
        assert_eq!(cpu.register.p.bits(), 0b01100101);
    }

    #[test]
    fn test_0x2a_rol() {
        let program = vec![0x2a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0b1100_1111;
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0b1001_1111);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x26_rol() {
        let program = vec![0x26, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0010, 0b0000_1111);
        cpu.run();

        let result = cpu.read_memory_byte(0x0010);
        assert_eq!(result, 0b0001_1110);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_0x6a_ror() {
        let program = vec![0x6a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0b1100_1111;
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0b1110_0111);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), true);
    }

    #[test]
    fn test_0x66_ror() {
        let program = vec![0x66, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0010, 0b0000_1110);
        cpu.run();

        let result = cpu.read_memory_byte(0x0010);
        assert_eq!(result, 0b0000_0111);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::z), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::n), false);
    }

    #[test]
    fn test_rti() {
        let program = vec![
            0x40, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x38, 0x00, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x00,
        ];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.sp = 0xfa;
        cpu.write_memory_word(0x01fc, 0x800a);
        cpu.write_memory_byte(0x01fb, 0b01010101);
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfd);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.bits(), 0b01100101);
    }

    #[test]
    fn test_rts() {
        let program = vec![
            0x60, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x38, 0x00, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea,
            0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0xea, 0x00,
        ];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.sp = 0xfb;
        cpu.write_memory_word(0x01fc, 0x8009);
        cpu.run();

        assert_eq!(cpu.register.sp, 0xfd);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
    }

    #[test]
    fn test_0xe9_sbc_subtraction() {
        let program = vec![0xe9, 0x40, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x50;
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0x10);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), false);
    }

    #[test]
    fn test_0xe9_sbc_overflow1() {
        let program = vec![0xe9, 0xb0, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0x50;
        cpu.register.p.insert(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0xa0);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), false);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), true);
    }

    #[test]
    fn test_0xe9_sbc_overflow2() {
        let program = vec![0xe9, 0x70, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 0xd0;
        cpu.register.p.remove(StatusRegister::c);
        cpu.run();

        assert_eq!(cpu.register.a, 0x5f);
        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
        assert_eq!(cpu.register.p.contains(StatusRegister::v), true);
    }

    #[test]
    fn test_sec_set_carry() {
        let program = vec![0x38, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::c), true);
    }

    #[test]
    fn test_sed_set_decimal() {
        let program = vec![0xf8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::d), true);
    }

    #[test]
    fn test_sei_disable_interrupt() {
        let program = vec![0x78, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.p.contains(StatusRegister::i), true);
    }

    #[test]
    fn test_lda_from_memory() {
        let program = vec![0xa5, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.write_memory_byte(0x0010, 0x55);
        cpu.run();

        assert_eq!(cpu.register.a, 0x55);
    }

    #[test]
    fn test_0x85_sta_store_a() {
        let program = vec![0x85, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 100;
        cpu.run();

        assert_eq!(cpu.read_memory_byte(0x0010), 100);
    }

    #[test]
    fn test_0x86_stx_store_x() {
        let program = vec![0x86, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 100;
        cpu.run();

        assert_eq!(cpu.read_memory_byte(0x0010), 100);
    }

    #[test]
    fn test_0x96_stx() {
        let program = vec![0x96, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 100;
        cpu.register.y = 0xff;
        cpu.run();

        assert_eq!(cpu.read_memory_byte(0x000f), 100);
    }

    #[test]
    fn test_0x84_sty_store_y() {
        let program = vec![0x84, 0x10, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.y = 100;
        cpu.run();

        assert_eq!(cpu.read_memory_byte(0x0010), 100);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let program = vec![0xaa, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 10;
        cpu.run();

        assert_eq!(cpu.register.x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.run();

        assert_eq!(cpu.register.x, 0xc1)
    }

    #[test]
    fn test_tay_move_a_to_y() {
        let program = vec![0xa8, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.a = 20;
        cpu.run();

        assert_eq!(cpu.register.y, 20);
    }

    #[test]
    fn test_tsx_move_sp_to_x() {
        let program = vec![0xba, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.sp = 0x50;
        cpu.run();

        assert_eq!(cpu.register.x, 0x50);
    }

    #[test]
    fn test_txa_move_x_to_a() {
        let program = vec![0x8a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 20;
        cpu.run();

        assert_eq!(cpu.register.a, 20);
    }

    #[test]
    fn test_txs_move_x_to_sp() {
        let program = vec![0x9a, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.x = 0x50;
        cpu.run();

        assert_eq!(cpu.register.sp, 0x50);
    }

    #[test]
    fn test_tya_move_y_to_a() {
        let program = vec![0x98, 0x00];
        let bus = create_test_bus(&program);
        let mut cpu = Cpu::new(bus);
        cpu.register.y = 20;
        cpu.run();

        assert_eq!(cpu.register.a, 20);
    }
}
