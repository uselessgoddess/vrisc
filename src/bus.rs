use crate::{Dram, Exception, DRAM_SIZE};

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
  dram = [0x8000_0000; DRAM_SIZE];
}

pub struct Bus {
  pub dram: Dram,
}

impl Bus {
  pub fn load(&mut self, addr: u64, size: u8) -> Result<u64, Exception> {
    match addr {
      dram::ADDR..=dram::END => self.dram.load(addr, size),
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
      dram::ADDR..=dram::END => self.dram.store(addr, value, size),
      _ => Err(Exception::StoreAMOAccessFault),
    }
  }
}
