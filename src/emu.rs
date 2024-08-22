use crate::{Cpu, Exception, Trap};

pub struct Emu {
  pub cpu: Cpu,
}

impl Emu {
  pub fn new() -> Self {
    Self { cpu: Cpu::new() }
  }

  pub fn with_dram(&mut self, dram: &[u8]) -> &mut Self {
    self.cpu.bus.dram.init(dram);
    self
  }

  pub fn with_pc(&mut self, pc: u64) -> &mut Self {
    self.cpu.pc = pc;
    self
  }

  pub fn cycle(&mut self) -> Result<u64, Exception> {
    match self.cpu.execute() {
      Ok(inst) => Ok(inst),
      Err(ex) => Err(ex),
    }
  }
}