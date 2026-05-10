#[derive(Debug, Copy, Clone)]
pub struct Decoded {
    pub cond: u8,
    pub kind: InstructionKind,
    pub raw: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InstructionKind {
    MovImm,
    Branch,
    Svc,
    LdrLiteral,
    Unknown,
}

struct ArmRule {
    kind: InstructionKind,
    fixed: &'static [(u8, u8, u32)],
}

// Build a classification rule from fixed bit ranges.
//
// This macro does not decode instruction operands. It only records enough
// metadata to answer "does this raw word belong to this instruction kind?".
// `bits hi..lo = value` and `bit n = value` are inclusive bit matches.
//
// Example:
// `arm_rule!(Branch { bits 27..25 = 0b101 })`
// means "classify as Branch when bits 27..25 equal 101".
macro_rules! arm_rule {
    ($kind:ident { $($fixed:tt)* }) => {
        ArmRule {
            kind: InstructionKind::$kind,
            fixed: arm_fixed!($($fixed)*),
        }
    };
}

macro_rules! arm_fixed {
    (@acc [$($out:expr,)*]) => {
        &[$($out),*]
    };
    (@acc [$($out:expr,)*] bit $bit:literal = $value:expr,) => {
        arm_fixed!(@acc [$($out,)* ($bit, $bit, $value),])
    };
    (@acc [$($out:expr,)*] bit $bit:literal = $value:expr, $($rest:tt)+) => {
        arm_fixed!(@acc [$($out,)* ($bit, $bit, $value),] $($rest)+)
    };
    (@acc [$($out:expr,)*] bit $bit:literal = $value:expr) => {
        arm_fixed!(@acc [$($out,)* ($bit, $bit, $value),])
    };
    (@acc [$($out:expr,)*] bits $high:literal .. $low:literal = $value:expr,) => {
        arm_fixed!(@acc [$($out,)* ($high, $low, $value),])
    };
    (@acc [$($out:expr,)*] bits $high:literal .. $low:literal = $value:expr, $($rest:tt)+) => {
        arm_fixed!(@acc [$($out,)* ($high, $low, $value),] $($rest)+)
    };
    (@acc [$($out:expr,)*] bits $high:literal .. $low:literal = $value:expr) => {
        arm_fixed!(@acc [$($out,)* ($high, $low, $value),])
    };
    () => {
        &[]
    };
    ($($fixed:tt)*) => {
        arm_fixed!(@acc [] $($fixed)*)
    };
}

const DECODE_RULES: &[ArmRule] = &[
    arm_rule!(Svc {
        bits 27..24 = 0b1111,
    }),
    arm_rule!(Branch {
        bits 27..25 = 0b101,
    }),
    // LDR literal: `ldr Rt, [pc, #imm12]`.
    arm_rule!(LdrLiteral {
        bits 27..26 = 0b01, // load/store class
        bit 24 = 1,         // P: pre-indexed address
        bit 23 = 1,         // U: add positive offset
        bit 22 = 0,         // B: word transfer, not byte
        bit 21 = 0,         // W: no writeback
        bit 20 = 1,         // L: load, not store
        bits 19..16 = 15,   // Rn: base register is pc/r15
    }),
    // MOV immediate: data-processing immediate with opcode 1101.
    arm_rule!(MovImm {
        bits 27..25 = 0b001,
        bits 24..21 = Op::Mov as u32,
    }),
];

/// Classify a raw 32-bit ARM word into the subset of instructions currently modeled.
pub fn decode_word(word: u32) -> Decoded {
    let cond = bits(word, 31, 28) as u8;
    let kind = DECODE_RULES
        .iter()
        .find(|rule| rule.matches(word))
        .map(|rule| rule.kind)
        .unwrap_or(InstructionKind::Unknown);

    Decoded {
        cond,
        kind,
        raw: word,
    }
}

impl ArmRule {
    fn matches(&self, word: u32) -> bool {
        self.fixed
            .iter()
            .all(|&(high, low, value)| bits(word, high, low) == value)
    }
}

pub fn bits(word: u32, high: u8, low: u8) -> u32 {
    debug_assert!(high < 32);
    debug_assert!(low <= high);
    let width = high - low + 1;
    (word >> low) & ((1 << width) - 1)
}

pub fn bit(word: u32, bit: u8) -> bool {
    bits(word, bit, bit) != 0
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
pub fn decode_rotated_imm(imm12: u32) -> u32 {
    let rotate = ((imm12 >> 8) & 0b1111) * 2;
    (imm12 & 0xff).rotate_right(rotate)
}

#[cfg(test)]
mod tests {
    use super::{InstructionKind, Op, decode_word, op_from_bits};

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
    fn classifies_mov_immediate() {
        let word = 0xe3a0_7004;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::MovImm);
        assert_eq!(decoded.raw, word);
    }

    #[test]
    fn classifies_rotated_mov_immediate() {
        let word = 0xe3a0_0c01;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::MovImm);
        assert_eq!(decoded.raw, word);
    }

    #[test]
    fn preserves_condition_code() {
        let word = 0x03a0_0001;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, 0);
        assert_eq!(decoded.kind, InstructionKind::MovImm);
    }

    #[test]
    fn classifies_ldr_literal() {
        let word = 0xe59f_1014;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::LdrLiteral);
    }

    #[test]
    fn classifies_branch_with_link() {
        let word = 0xeb00_0001;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::Branch);
    }

    #[test]
    fn classifies_branch_without_link() {
        let word = 0xea00_0001;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::Branch);
    }

    #[test]
    fn classifies_negative_branch_offset_as_branch() {
        let word = 0xeaff_fffe;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::Branch);
    }

    #[test]
    fn classifies_svc() {
        let word = 0xef00_0000;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::Svc);
    }

    #[test]
    fn unsupported_data_processing_register_is_unknown() {
        let word = 0xe1a0_0003;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::Unknown);
    }

    #[test]
    fn unsupported_bx_is_unknown() {
        let word = 0xe12f_ff1e;
        let decoded = decode_word(word);

        assert_eq!(decoded.cond, cond(word));
        assert_eq!(decoded.kind, InstructionKind::Unknown);
    }
}
