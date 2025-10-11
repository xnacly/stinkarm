pub mod decoder;
pub mod exec;

#[derive(Default)]
pub struct Cpu {
    pub regs: [u32; 16],
    pub cpsr: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.regs = [0; 16];
        self.cpsr = 0;
    }

    pub fn step(&mut self) {
        todo!("Cpu::step")
    }
}
