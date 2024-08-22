use std::ops::RangeInclusive;

pub type Addr = u16;
pub type Range = RangeInclusive<usize>;

pub const REGISTERS: usize = 4096;

macro_rules! reg {
  ($(#[doc = $doc:expr] $name:ident = $expr:expr)*) => {
    pub mod reg {$(
      #[doc = $doc]
      pub const $name: super::Addr = $expr;
    )*}

    pub use reg::*;
  };
}

reg! {
  /// ISA and extensions
  MISA = 0x301
}

pub struct State {
  regs: [u64; REGISTERS],
}

impl State {
  pub fn new() -> Self {
    let mut regs = [0; REGISTERS];
    let misa: u64 = (2 << 62) | // MXL[1:0]=2 (XLEN is 64)
        (1 << 20) | // Extensions[20] (User mode implemented)
        (1 << 18) | // Extensions[18] (Supervisor mode implemented)
        (1 << 12) | // Extensions[12] (Integer Multiply/Divide extension)
        (1 << 8) | // Extensions[8] (RV32I/64I/128I base ISA)
        (1 << 5) | // Extensions[5] (Single-precision floating-point extension)
        (1 << 3) | // Extensions[3] (Double-precision floating-point extension)
        (1 << 2) | // Extensions[2] (Compressed extension)
        1; // Extensions[0] (Atomic extension)
    regs[MISA as usize] = misa;

    Self { regs }
  }
}
