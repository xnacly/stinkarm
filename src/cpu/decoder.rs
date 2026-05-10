#[derive(Debug, Copy, Clone)]
pub struct InstructionContainer {
    pub cond: u8,
    pub instruction: Instruction,
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    MovImm { rd: u8, rhs: u32 },
    // target_offset is signed byte-offset already shifted (<<2)
    Branch { link: bool, target_offset: i32 },
    Svc,
    LdrLiteral { rd: u8, addr: u32 },
    Unknown(u32),
}

/// Data-processing opcode field, encoded in bits 24..21.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Op {
    And = 0b0000,
    Eor = 0b0001,
    Sub = 0b0010,
    Rsb = 0b0011,
    Add = 0b0100,
    Adc = 0b0101,
    Sbc = 0b0110,
    Rsc = 0b0111,
    Tst = 0b1000,
    Teq = 0b1001,
    Cmp = 0b1010,
    Cmn = 0b1011,
    Orr = 0b1100,
    Mov = 0b1101,
    Bic = 0b1110,
    Mvn = 0b1111,
}

// Lookup table for the 4-bit data-processing opcode field.
static OP_TABLE: [Op; 16] = [
    Op::And,
    Op::Eor,
    Op::Sub,
    Op::Rsb,
    Op::Add,
    Op::Adc,
    Op::Sbc,
    Op::Rsc,
    Op::Tst,
    Op::Teq,
    Op::Cmp,
    Op::Cmn,
    Op::Orr,
    Op::Mov,
    Op::Bic,
    Op::Mvn,
];

#[inline(always)]
fn op_from_bits(bits: u8) -> Op {
    debug_assert!(bits <= 0b1111);
    unsafe { *OP_TABLE.get_unchecked(bits as usize) }
}

#[inline(always)]
/// Decode ARM's 12-bit modified immediate.
///
/// The low 8 bits hold the literal byte. Bits 11..8 hold a rotation count,
/// stored in units of two bits. The decoded value is:
/// `imm8.rotate_right(rotation_count * 2)`.
///
/// For example, imm12 `0xc01` means imm8 `0x01` rotated right by 24 bits,
/// which produces `0x00000100`.
fn decode_rotated_imm(imm12: u32) -> u32 {
    let rotate = ((imm12 >> 8) & 0b1111) * 2;
    (imm12 & 0xff).rotate_right(rotate)
}

/// Turn a raw 32-bit ARM word into the subset of instructions currently modeled.
pub fn decode_word(word: u32, caddr: u32) -> InstructionContainer {
    let cond = ((word >> 28) & 0xF) as u8;
    let top = ((word >> 25) & 0x7) as u8; // bits 27:25

    // Single data transfer class: LDR/STR immediate/register offset encodings.
    if (top >> 1) & 0b11 == 0b01 {
        // P: pre-index when set, post-index when clear.
        let p = ((word >> 24) & 1) != 0;
        // U: add offset when set, subtract offset when clear.
        let u = ((word >> 23) & 1) != 0;
        // B: byte transfer when set, word transfer when clear.
        let b = ((word >> 22) & 1) != 0;
        // W: write back the computed address to Rn.
        let w = ((word >> 21) & 1) != 0;
        // L: load when set, store when clear.
        let l = ((word >> 20) & 1) != 0;
        // Rn: base register.
        let rn = ((word >> 16) & 0xF) as u8;
        // Rd: destination/source register for load/store.
        let rd = ((word >> 12) & 0xF) as u8;
        let imm12 = word & 0xFFF;

        // LDR literal is encoded as `ldr Rt, [pc, #imm12]`.
        if l && rn == 0b1111 && p && u && !w && !b {
            // In ARM state, reading PC from an instruction sees current address + 8.
            let pc_seen = caddr.wrapping_add(8);
            let literal_addr = pc_seen.wrapping_add(imm12);

            return InstructionContainer {
                cond,
                instruction: Instruction::LdrLiteral {
                    rd,
                    addr: literal_addr,
                },
            };
        }

        todo!("only LDR with p&u&!w&!b is implemented")
    }

    // Branch and branch-with-link: bits 27..25 == 101.
    if top == 0b101 {
        let link = ((word >> 24) & 0x1) != 0;
        let imm24 = (word & 0x00FF_FFFF) as i32;
        // imm24 is signed, then shifted left by 2 to become a byte offset.
        let signed = (imm24 << 8) >> 8;
        let offset = signed.wrapping_shl(2);
        return InstructionContainer {
            cond,
            instruction: Instruction::Branch {
                link,
                target_offset: offset,
            },
        };
    }

    // SVC: bits 27..24 == 1111. The low 24 immediate bits are ignored for Linux EABI syscalls.
    if ((word >> 24) & 0xF) as u8 == 0b1111 {
        return InstructionContainer {
            cond,
            instruction: Instruction::Svc,
        };
    }

    // Data-processing immediate: bits 27..25 == 001.
    if top == 0b000 || top == 0b001 {
        let i_bit = ((word >> 25) & 0x1) != 0;
        let opcode = ((word >> 21) & 0xF) as u8;
        if i_bit {
            match op_from_bits(opcode) {
                // MOV immediate
                Op::Mov => {
                    let rd = ((word >> 12) & 0xF) as u8;
                    let imm12 = word & 0xFFF;
                    let rhs = decode_rotated_imm(imm12);
                    return InstructionContainer {
                        cond,
                        instruction: Instruction::MovImm { rd, rhs },
                    };
                }
                _ => todo!(),
            }
        }
    }

    InstructionContainer {
        cond,
        instruction: Instruction::Unknown(word),
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Op, decode_word, op_from_bits};

    fn cond(word: u32) -> u8 {
        ((word >> 28) & 0xf) as u8
    }

    #[test]
    fn op_from_bits_maps_data_processing_opcodes() {
        let expected = [
            Op::And,
            Op::Eor,
            Op::Sub,
            Op::Rsb,
            Op::Add,
            Op::Adc,
            Op::Sbc,
            Op::Rsc,
            Op::Tst,
            Op::Teq,
            Op::Cmp,
            Op::Cmn,
            Op::Orr,
            Op::Mov,
            Op::Bic,
            Op::Mvn,
        ];

        for (bits, op) in expected.into_iter().enumerate() {
            assert_eq!(op_from_bits(bits as u8), op);
        }
    }

    #[test]
    fn decodes_mov_immediate() {
        let word = 0xe3a0_7004;
        let decoded = decode_word(word, 0x8000);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::MovImm { rd: 7, rhs: 4 }
        ));
    }

    #[test]
    fn decodes_rotated_mov_immediate() {
        let word = 0xe3a0_0c01;
        let decoded = decode_word(word, 0x8000);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::MovImm { rd: 0, rhs: 0x100 }
        ));
    }

    #[test]
    fn preserves_condition_code() {
        let word = 0x03a0_0001;
        let decoded = decode_word(word, 0x8000);

        assert_eq!(decoded.cond, 0);
        assert!(matches!(
            decoded.instruction,
            Instruction::MovImm { rd: 0, rhs: 1 }
        ));
    }

    #[test]
    fn decodes_ldr_literal() {
        let word = 0xe59f_1014;
        let decoded = decode_word(word, 0x8004);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::LdrLiteral {
                rd: 1,
                addr: 0x8020
            }
        ));
    }

    #[test]
    fn decodes_branch_with_link() {
        let word = 0xeb00_0001;
        let decoded = decode_word(word, 0x8000);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::Branch {
                link: true,
                target_offset: 4
            }
        ));
    }

    #[test]
    fn decodes_branch_without_link() {
        let word = 0xea00_0001;
        let decoded = decode_word(word, 0x8000);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::Branch {
                link: false,
                target_offset: 4
            }
        ));
    }

    #[test]
    fn decodes_negative_branch_offset() {
        let word = 0xeaff_fffe;
        let decoded = decode_word(word, 0x8008);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::Branch {
                link: false,
                target_offset: -8
            }
        ));
    }

    #[test]
    fn decodes_svc() {
        let word = 0xef00_0000;
        let decoded = decode_word(word, 0x8000);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(decoded.instruction, Instruction::Svc));
    }

    #[test]
    fn unsupported_data_processing_register_returns_unknown() {
        let word = 0xe1a0_0003;
        let decoded = decode_word(word, 0x8018);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::Unknown(0xe1a0_0003)
        ));
    }

    #[test]
    fn unsupported_bx_returns_unknown() {
        let word = 0xe12f_ff1e;
        let decoded = decode_word(word, 0x8024);

        assert_eq!(decoded.cond, cond(word));
        assert!(matches!(
            decoded.instruction,
            Instruction::Unknown(0xe12f_ff1e)
        ));
    }
}
