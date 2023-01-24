use crate::cpu::AddressingMode;
use std::collections::HashMap;

// 本来はグローバル変数で置きたいところだが、std以外の機能が必要になる。
// 仕方なく、Cpuの初期化時に作って持たせる。
pub fn create_opcodes_map() -> HashMap<u8, Opcode> {
    let opcode_list: Vec<Opcode> = vec![
        Opcode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        Opcode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),
        Opcode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
        Opcode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
        // 0xbd cycles +1 if page crossed
        Opcode::new(0xbd, "LDA", 3, 4, AddressingMode::AbsoluteX),
        // 0xb9 cycles +1 if page crossed
        Opcode::new(0xb9, "LDA", 3, 4, AddressingMode::AbsoluteY),
        // 0xa1 cycles +1 if page crossed
        Opcode::new(0xa1, "LDA", 2, 6, AddressingMode::IndirectX),
        Opcode::new(0xb1, "LDA", 2, 5, AddressingMode::IndirectY),
        Opcode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPageX),
        Opcode::new(0x8d, "STA", 3, 4, AddressingMode::Absolute),
        Opcode::new(0x9d, "STA", 3, 5, AddressingMode::AbsoluteX),
        Opcode::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY),
        Opcode::new(0x81, "STA", 2, 6, AddressingMode::IndirectX),
        Opcode::new(0x91, "STA", 2, 6, AddressingMode::IndirectY),
    ];

    let mut map = HashMap::new();
    for opcode in opcode_list {
        map.insert(opcode.code, opcode);
    }
    map
}

pub struct Opcode {
    code: u8,
    instruction: &'static str,
    bytes: u8,
    cycles: u8,
    addressing_mode: AddressingMode,
}

impl Opcode {
    fn new(
        code: u8,
        instruction: &'static str,
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
