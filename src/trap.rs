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
  EnvCallFromUMode,
  EnvCallFromSMode,
  EnvCallFromMMode,
  InstPageFault(u64),
  LoadPageFault(u64),
  StoreAMOPageFault(u64),
}

impl Exception {
  fn epc(&self, pc: u64) -> u64 {
    match self {
      Exception::Breakpoint
      | Exception::EnvCallFromUMode
      | Exception::EnvCallFromSMode
      | Exception::EnvCallFromMMode
      | Exception::InstPageFault(_)
      | Exception::LoadPageFault(_)
      | Exception::StoreAMOPageFault(_) => pc,
      _ => pc.wrapping_add(4),
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
      Exception::InstAddrMisalign
      | Exception::InstAccessFault
      | Exception::LoadAddrMisalign
      | Exception::LoadAccessFault
      | Exception::StoreAMOAddrMisalign
      | Exception::StoreAMOAccessFault => Trap::Fatal,
      Exception::Breakpoint
      | Exception::EnvCallFromUMode
      | Exception::EnvCallFromSMode
      | Exception::EnvCallFromMMode => Trap::Requested,
      Exception::IllegalInst(_)
      | Exception::InstPageFault(_)
      | Exception::LoadPageFault(_)
      | Exception::StoreAMOPageFault(_) => Trap::Invisible,
    }
  }
}
