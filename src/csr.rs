use std::cmp::{max, min};

pub type Addr = u16;
pub type Range = (usize, usize);

pub const MXLEN: usize = 64;
pub const REGISTERS: usize = 1 << 12;

macro_rules! bits {
    [{$a:literal:$b:literal}] => { ($a, $b + 1) };
    [{$x:literal}] => { ($x, $x + 1) };
}

macro_rules! field {
  ($name:ident = $($tt:tt)*) => {
    pub const $name: crate::csr::Range = bits![{ $($tt)* }];
  };
}

macro_rules! reg {
  {$desc:literal $(#[doc = $doc:expr] $name:ident = $expr:expr)*} => {
    $(
      #[doc = $doc]
      pub const $name: Addr = $expr;
    )*
  };

  {$desc:literal as $ty:ident $(#[doc = $doc:expr] $name:ident = $expr:expr)*} => {
    $(
      #[doc = $doc]
      pub const $name: $ty = $expr;
    )*
  };
}

macro_rules! mask {
  {$($field:expr)*} => {
    $(mask::<{ $field }>() )|*
  };
}

reg! { "User Counter/Timers"
  /// Timer for RDTIME instruction
  TIME = 0xc01
}

reg! { "Supervisor traps setup"
  /// Machine status register
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

reg! { "Supervisor traps handling"
  /// Supervisor exception program counter
  SEPC = 0x141
  /// Supervisor trap cause
  SCAUSE = 0x142
  /// Supervisor bad address or instruction
  STVAL = 0x143
  /// Supervisor interrupt pending
  SIP = 0x144
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

reg! { "MIP fields" as u64
  /// Supervisor software interrupt.
  SSIP_BIT= 1 << 1
  /// Machine software interrupt.
  MSIP_BIT = 1 << 3
  /// Supervisor timer interrupt.
  STIP_BIT = 1 << 5
  /// Machine timer interrupt.
  MTIP_BIT= 1 << 7
  /// Supervisor external interrupt.
  SEIP_BIT= 1 << 9
  /// Machine external interrupt.
  MEIP_BIT = 1 << 11
}

const fn mask<const R: Range>() -> u64 {
  let (start, end) = R;
  let len = end - start;

  (u64::MAX >> (64 - len)) << start
}

const _: () = {
  assert!(mask::<{ x::SIE }>() == 0b10);
  assert!(mask::<{ x::SPIE }>() == 0b100000);
  assert!(mask::<{ x::MPP }>() == 0b1100000000000);
};

const SSTATUS_MASK: u64 = mask! { x::SIE x::SPIE x::SPP };

pub mod x {
  field![SIE = 1:1];
  field![SPIE = 5:5];
  field![SPP = 8:8];
  //
  field![MIE = 3:3];
  field![MPIE = 7:7];
  field![MPP = 11:12];
}

#[derive(Debug)]
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

  pub fn cycle_time(&mut self) {
    self.regs[TIME as usize] = self.regs[TIME as usize].wrapping_add(1);
  }

  pub fn load(&self, addr: Addr) -> u64 {
    match addr {
      SSTATUS => self.regs[MSTATUS as usize] & SSTATUS_MASK,
      SIE => self.regs[MIE as usize] & self.regs[MIDELEG as usize],
      SIP => self.regs[MIP as usize] & self.regs[MIDELEG as usize],
      _ => self.regs[addr as usize],
    }
  }

  pub fn store(&mut self, addr: Addr, val: u64) {
    match addr {
      SSTATUS => {
        self.regs[MSTATUS as usize] =
          (self.regs[MSTATUS as usize] & !SSTATUS_MASK) | (val & SSTATUS_MASK);
      }
      SIE => {
        self.regs[MIE as usize] = (self.regs[MIE as usize]
          & !self.regs[MIDELEG as usize])
          | (val & self.regs[MIDELEG as usize]);
      }
      SIP => {
        let mask = SSIP_BIT & self.regs[MIDELEG as usize];
        self.regs[MIP as usize] =
          (self.regs[MIP as usize] & !mask) | (val & mask);
      }
      _ => self.regs[addr as usize] = val,
    }
  }

  pub fn store_bits(&mut self, addr: Addr, (start, end): Range, val: u64) {
    let mask = (!0 << end) | !(!0 << start);
    self.store(addr, (self.load(addr) & mask) | (val << start))
  }

  pub fn load_bits(&self, addr: Addr, (start, end): Range) -> u64 {
    let mask = if end != MXLEN { !0 << end } else { 0 };
    (self.load(addr) & !mask) >> start
  }

  pub fn load_sstatus(&self, bits: Range) -> u64 {
    self.load_bits(SSTATUS, bits)
  }

  pub fn load_mstatus(&self, bits: Range) -> u64 {
    self.load_bits(MSTATUS, bits)
  }

  pub fn store_sstatus(&mut self, bits: Range, val: u64) {
    self.store_bits(SSTATUS, bits, val);
  }

  pub fn store_mstatus(&mut self, bits: Range, val: u64) {
    self.store_bits(MSTATUS, bits, val);
  }
}
