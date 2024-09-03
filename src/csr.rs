use std::ops::Range;

pub type Addr = u16;

pub const MXLEN: usize = 64;
pub const REGISTERS: usize = 1 << 12;

macro_rules! bits {
    [{$a:literal:$b:literal}] => { $a..$b + 1 };
    [{$x:literal}] => { $x..$x + 1 };
}

macro_rules! field {
  ($name:ident = $($tt:tt)*) => {
    pub const $name: std::ops::Range<usize> = bits![{ $($tt)* }];
  };
}

macro_rules! reg {
  {$desc:literal $(#[doc = $doc:expr] $name:ident = $expr:expr)*} => {
    $(
      #[doc = $doc]
      pub const $name: Addr = $expr;
    )*
  };
}

reg! { "Supervisor traps setup"
  /// Machine status register.
  SSTATUS = 0x100
  /// Machine exception delegation
  SEDELEG = 0x102
  /// Machine interrupt delegation
  SIDELEG = 0x103
  /// Machine interrupt-enable
  SIE = 0x104
  /// Machine trap-handler base address
  STVEC = 0x105
}

reg! { "Machine traps setup"
  /// Machine status register.
  MSTATUS = 0x300
  /// ISA and extensions
  MISA = 0x301
  /// Machine exception delegation
  MEDELEG = 0x302
  /// Machine interrupt delegation
  MIDELEG = 0x303
  /// Machine interrupt-enable
  MIE = 0x304
  /// Machine trap-handler base address
  MTVEC = 0x305
}

reg! { "Machine traps handling"
  /// Machine exception program counter
  MEPC = 0x341
  /// Machine trap cause
  MCAUSE = 0x342
  /// Machine bad address or instruction
  MTVAL = 0x343
  /// Machine interrupt pending
  MIP = 0x344
}

pub mod x {
  field![SIE = 1:1];
  field![SPIE = 5:5];
  field![SPP = 8:8];
  //
  field![MIE = 3:3];
  field![MPIE = 7:7];
  field![MPP = 11:12];
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

  pub fn load(&self, addr: Addr) -> u64 {
    match addr {
      _ => self.regs[addr as usize],
    }
  }

  pub fn store(&mut self, addr: Addr, val: u64) {
    match addr {
      _ => self.regs[addr as usize] = val,
    }
  }

  pub fn store_bits(
    &mut self,
    addr: Addr,
    Range { start, end }: Range<usize>,
    val: u64,
  ) {
    let mask = (!0 << end) | !(!0 << start);
    self.store(addr, (self.load(addr) & mask) | (val << start))
  }

  pub fn load_bits(
    &self,
    addr: Addr,
    Range { start, end }: Range<usize>,
  ) -> u64 {
    let mask = if end != MXLEN { !0 << end } else { 0 };
    (self.load(addr) & !mask) >> start
  }

  pub fn load_sstatus(&self, bits: Range<usize>) -> u64 {
    self.load_bits(SSTATUS, bits)
  }

  pub fn load_mstatus(&self, bits: Range<usize>) -> u64 {
    self.load_bits(MSTATUS, bits)
  }

  pub fn store_sstatus(&mut self, bits: Range<usize>, val: u64) {
    self.store_bits(SSTATUS, bits, val);
  }

  pub fn store_mstatus(&mut self, bits: Range<usize>, val: u64) {
    self.store_bits(MSTATUS, bits, val);
  }
}
