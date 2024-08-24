use crate::{bus::dram, Bus, Dram, Exception, State, Trap, DRAM_SIZE};

pub const REG_COUNT: usize = 32;
pub const POINTER_TO_DTB: u64 = 0x1020;

/// Access type that is used in the virtual address translation process. It decides which exception
/// should raises (InstructionPageFault, LoadPageFault or StoreAMOPageFault).
#[derive(Debug, PartialEq, PartialOrd)]
pub enum AccessType {
  /// Raises the exception InstructionPageFault. It is used for an instruction fetch.
  Instruction,
  /// Raises the exception LoadPageFault.
  Load,
  /// Raises the exception StoreAMOPageFault.
  Store,
}

#[derive(Debug)]
pub struct Xregs {
  xregs: [u64; REG_COUNT],
}

impl Xregs {
  pub fn new() -> Self {
    let mut xregs = [0; REG_COUNT];
    xregs[2] = dram::END;
    xregs[11] = POINTER_TO_DTB;
    Self { xregs }
  }

  pub fn into_inner(self) -> [u64; REG_COUNT] {
    self.xregs
  }

  pub fn load(&self, index: u64) -> u64 {
    self.xregs[index as usize]
  }

  pub fn store(&mut self, index: u64, value: u64) {
    self.xregs[index as usize] = value;
  }
}

pub const BYTE: u8 = 8;
pub const HALF: u8 = 16;
pub const WORD: u8 = 32;
pub const DWORD: u8 = 64;

pub struct Cpu {
  pub xregs: Xregs,
  pub pc: u64,
  pub state: State,
  pub bus: Bus,
}

impl Cpu {
  pub fn new() -> Self {
    Self {
      xregs: Xregs::new(),
      pc: 0,
      state: State::new(),
      bus: Bus { dram: Dram::new() },
    }
  }

  pub(crate) fn debug(&self, _inst: u64, _name: &str) {
    println!("DEBUG INST: {_name}");
  }

  /// Translate a virtual address to a physical address for the paged virtual-memory system.
  fn translate(
    &mut self,
    addr: u64,
    _access: AccessType,
  ) -> Result<u64, Exception> {
    Ok(addr)
  }

  pub fn catch_exception(&mut self, ex: Exception) -> Trap {
    // TODO: there is no implemented way to handle illegal instruction by invisible traps
    if let Exception::IllegalInst(_) = ex {
      return Trap::Fatal;
    }

    // TODO: there is no csr manipulation for now
    Trap::from_ex(ex)
  }

  pub(crate) fn store(
    &mut self,
    v_addr: u64,
    value: u64,
    size: u8,
  ) -> Result<(), Exception> {
    let p_addr = self.translate(v_addr, AccessType::Store)?;
    self.bus.store(p_addr, value, size)
  }

  pub fn fetch(&mut self, size: u8) -> Result<u64, Exception> {
    let (HALF | WORD) = size else {
      return Err(Exception::InstAccessFault);
    };

    let p_pc = self.translate(self.pc, AccessType::Instruction)?;

    // The result of the read method can be `LoadAccessFault`. In fetch(), an error
    // should be `InstAccessFault`.
    match self.bus.load(p_pc, size) {
      Ok(value) => Ok(value),
      Err(_) => Err(Exception::InstAccessFault),
    }
  }

  pub fn execute(&mut self) -> Result<u64, Exception> {
    let inst = self.fetch(WORD)?;
    self.execute_general(inst)?;
    self.pc += 4;
    Ok(inst)
  }
}
