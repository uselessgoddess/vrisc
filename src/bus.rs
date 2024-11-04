use crate::{
  dev::vga::{Vga, VGA_HEIGHT, VGA_WIDTH},
  Dram, Exception, DRAM_SIZE,
};

macro_rules! devices {
  ($($ident:ident = [$addr:expr; $size:expr];)*) => {$(
    pub mod $ident {
      use super::*;

      pub const ADDR: u64 = $addr;
      pub const SIZE: u64 = $size;
      pub const END: u64 = ADDR + $size;
    }
  )*};
}

devices! {
  vga = [0xb8000; Vga::SIZE];
  dram = [0x8000_0000; DRAM_SIZE];
}

#[derive(Debug)]
pub struct Bus {
  pub vga: Vga,
  pub dram: Dram,
}

impl Bus {
  pub fn load(&mut self, addr: u64, size: u8) -> Result<u64, Exception> {
    match addr {
      vga::ADDR..=vga::END => self.vga.load(addr - vga::ADDR, size),
      dram::ADDR..=dram::END => self.dram.load(addr - dram::ADDR, size),
      _ => Err(Exception::LoadAccessFault),
    }
  }

  pub fn store(
    &mut self,
    addr: u64,
    value: u64,
    size: u8,
  ) -> Result<(), Exception> {
    match addr {
      vga::ADDR..=vga::END => self.vga.store(addr - vga::ADDR, value, size),
      dram::ADDR..=dram::END => self.dram.store(addr - dram::ADDR, value, size),
      _ => {
        println!("0x{:x}", addr);
        Err(Exception::StoreAMOAccessFault)
      }
    }
  }
}
