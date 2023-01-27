use crate::cpu::AddressingMode;
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
}

// 本来はグローバル変数で置きたいところだが、std以外の機能が必要になる。
// 仕方なく、Cpuの初期化時に作って持たせる。
pub fn create_opcodes_map() -> HashMap<u8, Opcode> {
    let opcode_list: Vec<Opcode> = vec![
        Opcode::new(0x00, Instruction::BRK, 1, 7, AddressingMode::NoneAddressing),
        Opcode::new(0xaa, Instruction::TAX, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xe8, Instruction::INX, 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xa9, Instruction::LDA, 2, 2, AddressingMode::Immediate),
        Opcode::new(0xa5, Instruction::LDA, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xb5, Instruction::LDA, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0xad, Instruction::LDA, 3, 4, AddressingMode::Absolute),
        // 0xbd cycles +1 if page crossed
        Opcode::new(0xbd, Instruction::LDA, 3, 4, AddressingMode::AbsoluteX),
        // 0xb9 cycles +1 if page crossed
        Opcode::new(0xb9, Instruction::LDA, 3, 4, AddressingMode::AbsoluteY),
        // 0xa1 cycles +1 if page crossed
        Opcode::new(0xa1, Instruction::LDA, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0xb1, Instruction::LDA, 2, 5, AddressingMode::IndirectY),
        Opcode::new(0x85, Instruction::STA, 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x95, Instruction::STA, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x8d, Instruction::STA, 3, 4, AddressingMode::Absolute),
        Opcode::new(0x9d, Instruction::STA, 3, 5, AddressingMode::AbsoluteX),
        Opcode::new(0x99, Instruction::STA, 3, 5, AddressingMode::AbsoluteY),
        Opcode::new(0x81, Instruction::STA, 2, 6, AddressingMode::IndirectX),
        Opcode::new(0x91, Instruction::STA, 2, 6, AddressingMode::IndirectY),
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
