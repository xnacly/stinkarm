/// decoding ARM instructions
pub mod decoder;
/// executing ARM instructions
pub mod exec;

#[derive(Default)]
/// Usermode emulation
pub struct Cpu {
    /// r0-r15 (r13=SP, r14=LR, r15=PC)
    pub r: [u32; 16],
    pub cpsr: u32,

    /// Banked
    pub r8_fiq: u32,
    pub r9_fiq: u32,
    pub r10_fiq: u32,
    pub r11_fiq: u32,
    pub r12_fiq: u32,
    pub r13_fiq: u32,
    pub r14_fiq: u32,

    pub r13_irq: u32,
    pub r14_irq: u32,

    pub r13_svc: u32,
    pub r14_svc: u32,

    pub r13_abt: u32,
    pub r14_abt: u32,

    pub r13_und: u32,
    pub r14_und: u32,

    /// program status
    pub spsr_fiq: u32,
    pub spsr_irq: u32,
    pub spsr_svc: u32,
    pub spsr_abt: u32,
    pub spsr_und: u32,

    // floating-point and NEON
    pub s: [u32; 32],  // single-precision
    pub q: [u128; 16], // NEON
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.r = [0; 16];
        self.cpsr = 0;
    }

    pub fn step(&mut self) {
        todo!("Cpu::step")
    }
}
