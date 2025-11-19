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

/// The 4‑bit primary opcode field (bits 24‑21) for dpi instructions, nonexhaustive list
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

// static lookup table
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
/// rotate-right the 8bit immediate according to arm spec
fn decode_rotated_imm(imm12: u32) -> u32 {
    // this works by taking the bits 11-0 of a dp imm instruction, take the first 8 bit as the
    // constant and the remaining 4 bits as a rotate count, the 8 bit value is rotated and the
    // result is taken as a u32
    let rotate = ((imm12 >> 8) & 0b1111) * 2;
    (imm12 & 0xff).rotate_right(rotate)
}

/// process bits defined in word and construct the equivalent Instruction
/// Turn a raw 32‑bit ARM word into an Instruction
pub fn decode_word(word: u32, caddr: u32) -> InstructionContainer {
    let cond = ((word >> 28) & 0xF) as u8;
    let top = ((word >> 25) & 0x7) as u8; // bits 27:25

    // load and store class
    if (top >> 1) & 0b11 == 0b01 {
        // preindex
        let p = ((word >> 24) & 1) != 0;
        // add
        let u = ((word >> 23) & 1) != 0;
        // byte
        let b = ((word >> 22) & 1) != 0;
        // write back
        let w = ((word >> 21) & 1) != 0;
        // load
        let l = ((word >> 20) & 1) != 0;
        // base register
        let rn = ((word >> 16) & 0xF) as u8;
        // destination for load or store
        let rd = ((word >> 12) & 0xF) as u8;
        let imm12 = word & 0xFFF;

        // Literal‑pool version
        if l && rn == 0b1111 && p && u && !w && !b {
            // PC is address_of_this_word + 8
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

    // Branch (top == 0b101) is a simple case
    if top == 0b101 {
        let link = ((word >> 24) & 0x1) != 0;
        let imm24 = (word & 0x00FF_FFFF) as i32;
        // sign-extend 24->32 then << 2
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

    // SVC detection: bits 27..24 == 1111
    if ((word >> 24) & 0xF) as u8 == 0b1111 {
        return InstructionContainer {
            cond,
            // technically arm says svc has a 24bit immediate but we dont care about it, since the
            // linux kernel also doesnt
            instruction: Instruction::Svc,
        };
    }

    // Data-processing immediate (ddi) (top 0b000 or 0b001 when I==1)
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
