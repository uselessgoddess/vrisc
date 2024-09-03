#[derive(Debug, PartialEq)]
pub enum Exception {
  InstAddrMisalign,
  InstAccessFault,
  IllegalInst(u64),
  Breakpoint,
  LoadAddrMisalign,
  LoadAccessFault,
  StoreAMOAddrMisalign,
  StoreAMOAccessFault,
  ECallUser,
  ECallSuper,
  ECallMachine,
  InstPageFault(u64),
  LoadPageFault(u64),
  StoreAMOPageFault(u64),
}

impl Exception {
  pub fn epc(&self, pc: u64) -> u64 {
    match self {
      Exception::Breakpoint
      | Exception::ECallUser
      | Exception::ECallSuper
      | Exception::ECallMachine
      | Exception::InstPageFault(_)
      | Exception::LoadPageFault(_)
      | Exception::StoreAMOPageFault(_) => pc,
      _ => pc.wrapping_add(4),
    }
  }

  pub fn cause(&self) -> u64 {
    match self {
      Self::InstAddrMisalign => 0,
      Self::InstAccessFault => 1,
      Self::IllegalInst(_) => 2,
      Self::Breakpoint => 3,
      Self::LoadAddrMisalign => 4,
      Self::LoadAccessFault => 5,
      Self::StoreAMOAddrMisalign => 6,
      Self::StoreAMOAccessFault => 7,
      Self::ECallUser => 8,
      Self::ECallSuper => 9,
      Self::ECallMachine => 11,
      Self::InstPageFault(_) => 12,
      Self::LoadPageFault(_) => 13,
      Self::StoreAMOPageFault(_) => 15,
    }
  }

  pub fn mtval(&self, pc: u64) -> u64 {
    match *self {
      Exception::InstAddrMisalign
      | Exception::InstAccessFault
      | Exception::Breakpoint
      | Exception::LoadAddrMisalign
      | Exception::LoadAccessFault
      | Exception::StoreAMOAddrMisalign
      | Exception::StoreAMOAccessFault => pc,
      Exception::InstPageFault(x)
      | Exception::LoadPageFault(x)
      | Exception::StoreAMOPageFault(x)
      | Exception::IllegalInst(x) => x,
      _ => 0,
    }
  }
}

#[derive(Debug)]
pub enum Trap {
  Contained,
  Requested,
  Invisible,
  Fatal,
}

impl Trap {
  pub fn from_ex(ex: Exception) -> Self {
    match ex {
      Exception::Breakpoint
      | Exception::ECallUser
      | Exception::ECallSuper
      | Exception::ECallMachine => Trap::Requested,
      Exception::IllegalInst(_)
      | Exception::InstPageFault(_)
      | Exception::LoadPageFault(_)
      | Exception::StoreAMOPageFault(_) => Trap::Invisible,
      Exception::InstAddrMisalign
      | Exception::InstAccessFault
      | Exception::LoadAddrMisalign
      | Exception::LoadAccessFault
      | Exception::StoreAMOAddrMisalign
      | Exception::StoreAMOAccessFault => Trap::Fatal,
    }
  }
}
