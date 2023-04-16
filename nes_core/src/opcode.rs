use crate::cpu::AddressingMode;
use core::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// Add with Carry
    ADC,
    /// Lgical AND
    AND,
    /// Arithmetic Shift Left
    ASL,
    /// Branch if Carry Clear
    BCC,
    /// Branch if Carry Set
    BCS,
    /// Branch if Equal
    BEQ,
    /// Bit Test
    BIT,
    /// Branch if Minus
    BMI,
    /// Branch if Not Equal
    BNE,
    /// Branch if Positive
    BPL,
    /// Force Interrupt
    BRK,
    /// Branch if Overflow Clear
    BVC,
    /// Branch if Overflow Set
    BVS,
    /// Clear Carry Flag
    CLC,
    /// Clear Decimal Mode
    CLD,
    /// Clear Interrupt Disable
    CLI,
    /// Clear Overflow Flag
    CLV,
    /// Compare
    CMP,
    /// Compare X Register
    CPX,
    /// Compare Y Register
    CPY,
    /// Decrement Memory
    DEC,
    /// Decrement X Register
    DEX,
    /// Decrement Y Register
    DEY,
    /// Exclusive OR
    EOR,
    /// increment Memory
    INC,
    /// increment X Register
    INX,
    /// increment Y Register
    INY,
    /// Jump
    JMP,
    /// Jump to Subroutine
    JSR,
    /// Load Accumulator
    LDA,
    /// Load X Register
    LDX,
    /// Load Y Register
    LDY,
    /// Logical Shift Right
    LSR,
    /// No Operation
    NOP,
    /// Logical Inclusive OR
    ORA,
    /// Push Accumulator
    PHA,
    /// Push Processor Status
    PHP,
    /// Pull Accumulator
    PLA,
    /// Pull Processor Status
    PLP,
    /// Rotate Left
    ROL,
    /// Rotate Right
    ROR,
    /// Return from Interrupt
    RTI,
    /// Return from Subroutine
    RTS,
    /// Subtract with Carry
    SBC,
    /// Set Carry Flag
    SEC,
    /// Set Decimal Flag
    SED,
    /// Set Interrupt Disable
    SEI,
    /// Store Accumulator
    STA,
    /// Store X Register
    STX,
    /// Store Y Register
    STY,
    /// Transfer Accumulator to X
    TAX,
    /// Transfer Accumulator to Y
    TAY,
    /// Transfer Stack Pointer to X
    TSX,
    /// Transfer X to Accumulator
    TXA,
    /// Transfer X to Stack Pointer
    TXS,
    /// Transfer Y to Stack Pointer
    TYA,
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// 本来はグローバル変数で置きたいところだが、std以外の機能が必要になる。
// 仕方なく、Cpuの初期化時に作って持たせる。
pub fn create_opcodes_map() -> HashMap<u8, Opcode> {
    let opcode_list: Vec<Opcode> = vec![
        Opcode::new(0x69, Instruction::ADC, 2, 2, AddressingMode::Immediate),
        Opcode::new(0x65, Instruction::ADC, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x75, Instruction::ADC, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x6d, Instruction::ADC, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x7d, Instruction::ADC, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0x79, Instruction::ADC, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0x61, Instruction::ADC, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0x71, Instruction::ADC, 2, 5, AddressingMode::IndirectY), // cycles +1 if page crossed
        Opcode::new(0x29, Instruction::AND, 2, 2, AddressingMode::Immediate),
        Opcode::new(0x25, Instruction::AND, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x35, Instruction::AND, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x2d, Instruction::AND, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x3d, Instruction::AND, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0x39, Instruction::AND, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0x21, Instruction::AND, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0x31, Instruction::AND, 2, 5, AddressingMode::IndirectY), // cycles +1 if page crossed
        Opcode::new(0x0a, Instruction::ASL, 1, 2, AddressingMode::Accumulator),
        Opcode::new(0x06, Instruction::ASL, 2, 5, AddressingMode::ZeroPage),
        Opcode::new(0x16, Instruction::ASL, 2, 6, AddressingMode::ZeroPageX),
        Opcode::new(0x0e, Instruction::ASL, 3, 6, AddressingMode::Absolute),
        Opcode::new(0x1e, Instruction::ASL, 3, 7, AddressingMode::AbsoluteX),
        Opcode::new(0x90, Instruction::BCC, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0xb0, Instruction::BCS, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0xf0, Instruction::BEQ, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0x24, Instruction::BIT, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x2c, Instruction::BIT, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x30, Instruction::BMI, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0xd0, Instruction::BNE, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0x10, Instruction::BPL, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0x00, Instruction::BRK, 1, 7, AddressingMode::NoneAddressing),
        Opcode::new(0x50, Instruction::BVC, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0x70, Instruction::BVS, 2, 2, AddressingMode::Relative), // cycles +1 if branch succeeds, cycles +2 if to a new page
        Opcode::new(0x18, Instruction::CLC, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xd8, Instruction::CLD, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x58, Instruction::CLI, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xb8, Instruction::CLV, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xc9, Instruction::CMP, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xc5, Instruction::CMP, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xd5, Instruction::CMP, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0xcd, Instruction::CMP, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xdd, Instruction::CMP, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0xd9, Instruction::CMP, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0xc1, Instruction::CMP, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0xd1, Instruction::CMP, 2, 5, AddressingMode::IndirectY), // cycles +1 if page crossed
        Opcode::new(0xe0, Instruction::CPX, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xe4, Instruction::CPX, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xec, Instruction::CPX, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xc0, Instruction::CPY, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xc4, Instruction::CPY, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xcc, Instruction::CPY, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xc6, Instruction::DEC, 2, 5, AddressingMode::ZeroPage),
        Opcode::new(0xd6, Instruction::DEC, 2, 6, AddressingMode::ZeroPageX),
        Opcode::new(0xce, Instruction::DEC, 3, 6, AddressingMode::Absolute),
        Opcode::new(0xde, Instruction::DEC, 3, 7, AddressingMode::AbsoluteX),
        Opcode::new(0xca, Instruction::DEX, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x88, Instruction::DEY, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x49, Instruction::EOR, 2, 2, AddressingMode::Immediate),
        Opcode::new(0x45, Instruction::EOR, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x55, Instruction::EOR, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x4d, Instruction::EOR, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x5d, Instruction::EOR, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0x59, Instruction::EOR, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0x41, Instruction::EOR, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0x51, Instruction::EOR, 2, 5, AddressingMode::IndirectY), // cycles +1 if page crossed
        Opcode::new(0xe6, Instruction::INC, 2, 5, AddressingMode::ZeroPage),
        Opcode::new(0xf6, Instruction::INC, 2, 6, AddressingMode::ZeroPageX),
        Opcode::new(0xee, Instruction::INC, 3, 6, AddressingMode::Absolute),
        Opcode::new(0xfe, Instruction::INC, 3, 7, AddressingMode::AbsoluteX),
        Opcode::new(0xe8, Instruction::INX, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xc8, Instruction::INY, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x4c, Instruction::JMP, 3, 3, AddressingMode::Absolute),
        Opcode::new(0x6c, Instruction::JMP, 3, 5, AddressingMode::Indirect),
        Opcode::new(0x20, Instruction::JSR, 3, 6, AddressingMode::Absolute),
        Opcode::new(0xa9, Instruction::LDA, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xa5, Instruction::LDA, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xb5, Instruction::LDA, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0xad, Instruction::LDA, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xbd, Instruction::LDA, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0xb9, Instruction::LDA, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0xa1, Instruction::LDA, 2, 6, AddressingMode::IndirectX), // cycles +1 if page crossed
        Opcode::new(0xb1, Instruction::LDA, 2, 5, AddressingMode::IndirectY),
        Opcode::new(0xa2, Instruction::LDX, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xa6, Instruction::LDX, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xb6, Instruction::LDX, 2, 4, AddressingMode::ZeroPageY),
        Opcode::new(0xae, Instruction::LDX, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xbe, Instruction::LDX, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0xa0, Instruction::LDY, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xa4, Instruction::LDY, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xb4, Instruction::LDY, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0xac, Instruction::LDY, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xbc, Instruction::LDY, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0x4a, Instruction::LSR, 1, 2, AddressingMode::Accumulator),
        Opcode::new(0x46, Instruction::LSR, 2, 5, AddressingMode::ZeroPage),
        Opcode::new(0x56, Instruction::LSR, 2, 6, AddressingMode::ZeroPageX),
        Opcode::new(0x4e, Instruction::LSR, 3, 6, AddressingMode::Absolute),
        Opcode::new(0x5e, Instruction::LSR, 3, 7, AddressingMode::AbsoluteX),
        Opcode::new(0xea, Instruction::NOP, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x09, Instruction::ORA, 2, 2, AddressingMode::Immediate),
        Opcode::new(0x05, Instruction::ORA, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x15, Instruction::ORA, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x0d, Instruction::ORA, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x1d, Instruction::ORA, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0x19, Instruction::ORA, 3, 4, AddressingMode::AbsoluteY),
        Opcode::new(0x01, Instruction::ORA, 2, 6, AddressingMode::IndirectX), // cycles +1 if page crossed
        Opcode::new(0x11, Instruction::ORA, 2, 5, AddressingMode::IndirectY), // cycles +1 if page crossed
        Opcode::new(0x48, Instruction::PHA, 1, 3, AddressingMode::NoneAddressing),
        Opcode::new(0x08, Instruction::PHP, 1, 3, AddressingMode::NoneAddressing),
        Opcode::new(0x68, Instruction::PLA, 1, 4, AddressingMode::NoneAddressing),
        Opcode::new(0x28, Instruction::PLP, 1, 4, AddressingMode::NoneAddressing),
        Opcode::new(0x2a, Instruction::ROL, 1, 2, AddressingMode::Accumulator),
        Opcode::new(0x26, Instruction::ROL, 2, 5, AddressingMode::ZeroPage),
        Opcode::new(0x36, Instruction::ROL, 2, 6, AddressingMode::ZeroPageX),
        Opcode::new(0x2e, Instruction::ROL, 3, 6, AddressingMode::Absolute),
        Opcode::new(0x3e, Instruction::ROL, 3, 7, AddressingMode::AbsoluteX),
        Opcode::new(0x6a, Instruction::ROR, 1, 2, AddressingMode::Accumulator),
        Opcode::new(0x66, Instruction::ROR, 2, 5, AddressingMode::ZeroPage),
        Opcode::new(0x76, Instruction::ROR, 2, 6, AddressingMode::ZeroPageX),
        Opcode::new(0x6e, Instruction::ROR, 3, 6, AddressingMode::Absolute),
        Opcode::new(0x7e, Instruction::ROR, 3, 7, AddressingMode::AbsoluteX),
        Opcode::new(0x40, Instruction::RTI, 1, 6, AddressingMode::NoneAddressing),
        Opcode::new(0x60, Instruction::RTS, 1, 6, AddressingMode::NoneAddressing),
        Opcode::new(0xe9, Instruction::SBC, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xe5, Instruction::SBC, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xf5, Instruction::SBC, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0xed, Instruction::SBC, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xfd, Instruction::SBC, 3, 4, AddressingMode::AbsoluteX), // cycles +1 if page crossed
        Opcode::new(0xf9, Instruction::SBC, 3, 4, AddressingMode::AbsoluteY), // cycles +1 if page crossed
        Opcode::new(0xe1, Instruction::SBC, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0xf1, Instruction::SBC, 2, 5, AddressingMode::IndirectY), // cycles +1 if page crossed
        Opcode::new(0x38, Instruction::SEC, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xf8, Instruction::SED, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x78, Instruction::SEI, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x85, Instruction::STA, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x95, Instruction::STA, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x8d, Instruction::STA, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x9d, Instruction::STA, 3, 5, AddressingMode::AbsoluteX),
        Opcode::new(0x99, Instruction::STA, 3, 5, AddressingMode::AbsoluteY),
        Opcode::new(0x81, Instruction::STA, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0x91, Instruction::STA, 2, 6, AddressingMode::IndirectY),
        Opcode::new(0x86, Instruction::STX, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x96, Instruction::STX, 2, 4, AddressingMode::ZeroPageY),
        Opcode::new(0x8e, Instruction::STX, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x84, Instruction::STY, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x94, Instruction::STY, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x8c, Instruction::STY, 3, 4, AddressingMode::Absolute),
        Opcode::new(0xaa, Instruction::TAX, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xa8, Instruction::TAY, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xba, Instruction::TSX, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x8a, Instruction::TXA, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x9a, Instruction::TXS, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0x98, Instruction::TYA, 1, 2, AddressingMode::NoneAddressing),
    ];

    let mut map = HashMap::new();
    for opcode in opcode_list {
        map.insert(opcode.code, opcode);
    }
    map
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode {
    code: u8,
    pub instruction: Instruction,
    pub bytes: u8,
    cycles: u8,
    pub addressing_mode: AddressingMode,
}

impl Opcode {
    fn new(
        code: u8,
        instruction: Instruction,
        bytes: u8,
        cycles: u8,
        addressing_mode: AddressingMode,
    ) -> Self {
        Self {
            code,
            instruction,
            bytes,
            cycles,
            addressing_mode,
        }
    }
}
