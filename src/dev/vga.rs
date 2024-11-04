use crate::{Dram, Exception};

pub const VGA_WIDTH: u64 = 224;
pub const VGA_HEIGHT: u64 = 126;

#[derive(Debug)]
pub struct Vga {
  pub buf: Dram,
}

impl Vga {
  pub const SIZE: u64 = 3 * VGA_WIDTH * VGA_HEIGHT;

  pub fn new() -> Self {
    Self { buf: Dram::with_capacity(Self::SIZE as usize) }
  }

  pub fn load(&self, addr: u64, size: u8) -> Result<u64, Exception> {
    self.buf.load(addr, size)
  }

  pub fn store(
    &mut self,
    addr: u64,
    value: u64,
    size: u8,
  ) -> Result<(), Exception> {
    //println!("store: {value} to 0x{addr:x}");
    self.buf.store(addr, value, size)
  }
}
