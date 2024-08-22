use crate::{
  bus::dram,
  cpu::{BYTE, DWORD, HALF, WORD},
  Exception,
};

pub const DRAM_SIZE: u64 = 1024 * 1024 * 1024;

pub struct Dram {
  dram: Vec<u8>,
}

impl Dram {
  pub fn new() -> Dram {
    Self { dram: vec![0; DRAM_SIZE as usize] }
  }

  pub fn init(&mut self, slice: &[u8]) {
    self.dram.splice(..slice.len(), slice.iter().cloned());
  }

  pub fn load(&mut self, addr: u64, size: u8) -> Result<u64, Exception> {
    Ok(match size {
      BYTE => self.load8(addr),
      HALF => self.load16(addr),
      WORD => self.load32(addr),
      DWORD => self.load64(addr),
      _ => return Err(Exception::StoreAMOAccessFault),
    })
  }

  pub fn store(
    &mut self,
    addr: u64,
    value: u64,
    size: u8,
  ) -> Result<(), Exception> {
    Ok(match size {
      BYTE => self.store8(addr, value),
      HALF => self.store16(addr, value),
      WORD => self.store32(addr, value),
      DWORD => self.store64(addr, value),
      _ => return Err(Exception::StoreAMOAccessFault),
    })
  }

  fn load8(&self, addr: u64) -> u64 {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] as u64
  }

  fn load16(&self, addr: u64) -> u64 {
    let index = (addr - dram::ADDR) as usize;
    return self.dram[index] as u64 | (self.dram[index + 1] as u64) << 8;
  }

  fn load32(&self, addr: u64) -> u64 {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] as u64
      | (self.dram[index + 1] as u64) << 8
      | (self.dram[index + 2] as u64) << 16
      | (self.dram[index + 3] as u64) << 24
  }

  fn load64(&self, addr: u64) -> u64 {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] as u64
      | (self.dram[index + 1] as u64) << 8
      | (self.dram[index + 2] as u64) << 16
      | (self.dram[index + 3] as u64) << 24
      | (self.dram[index + 4] as u64) << 32
      | (self.dram[index + 5] as u64) << 40
      | (self.dram[index + 6] as u64) << 48
      | (self.dram[index + 7] as u64) << 56
  }

  fn store8(&mut self, addr: u64, val: u64) {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] = val as u8
  }

  fn store16(&mut self, addr: u64, val: u64) {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] = ub(val);
    self.dram[index + 1] = ub(val >> 8);
  }

  fn store32(&mut self, addr: u64, val: u64) {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] = ub(val);
    self.dram[index + 1] = ub(val >> 8);
    self.dram[index + 2] = ub(val >> 16);
    self.dram[index + 3] = ub(val >> 24);
  }

  fn store64(&mut self, addr: u64, val: u64) {
    let index = (addr - dram::ADDR) as usize;
    self.dram[index] = ub(val);
    self.dram[index + 1] = ub(val >> 8);
    self.dram[index + 2] = ub(val >> 16);
    self.dram[index + 3] = ub(val >> 24);
    self.dram[index + 4] = ub(val >> 32);
    self.dram[index + 5] = ub(val >> 40);
    self.dram[index + 6] = ub(val >> 48);
    self.dram[index + 7] = ub(val >> 56);
  }
}

const fn ub(x: u64) -> u8 {
  (x & 0xff) as u8
}
