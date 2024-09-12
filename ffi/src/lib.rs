use {
  std::{
    alloc::{alloc, dealloc, Layout},
    mem::ManuallyDrop,
    ptr::slice_from_raw_parts_mut,
    slice,
  },
  vrisc::{Cpu, Emu, REG_COUNT},
};

pub use vrisc;

#[repr(u32)]
pub enum Trap {
  Contained,
  Requested,
  Invisible,
  Fatal,
}

impl From<Trap> for vrisc::Trap {
  fn from(value: Trap) -> Self {
    match value {
      Trap::Contained => Self::Contained,
      Trap::Requested => Self::Requested,
      Trap::Invisible => Self::Invisible,
      Trap::Fatal => Self::Fatal,
    }
  }
}

impl From<vrisc::Trap> for Trap {
  fn from(value: vrisc::Trap) -> Self {
    match value {
      vrisc::Trap::Contained => Trap::Contained,
      vrisc::Trap::Requested => Trap::Requested,
      vrisc::Trap::Invisible => Trap::Invisible,
      vrisc::Trap::Fatal => Trap::Fatal,
    }
  }
}

#[repr(u32)]
pub enum Mode {
  User = 0b00,
  Supervisor = 0b01,
  Machine = 0b11,
}

impl From<Mode> for vrisc::Mode {
  fn from(value: Mode) -> Self {
    match value {
      Mode::User => Self::User,
      Mode::Supervisor => Self::Supervisor,
      Mode::Machine => Self::Machine,
    }
  }
}

impl From<vrisc::Mode> for Mode {
  fn from(value: vrisc::Mode) -> Self {
    match value {
      vrisc::Mode::User => Mode::User,
      vrisc::Mode::Supervisor => Mode::Supervisor,
      vrisc::Mode::Machine => Mode::Machine,
      vrisc::Mode::Debug => unreachable!(),
    }
  }
}

#[repr(C)]
pub struct RegPlace<T> {
  regs: [T; REG_COUNT],
  len: u8,
}

#[repr(C)]
pub struct Slice<T> {
  ptr: *mut T,
  len: u32,
}

impl<T> Slice<T> {
  fn new(slice: Box<[T]>) -> Self {
    assert!(slice.len() <= u32::MAX as usize);

    let mut slice = ManuallyDrop::new(slice);
    Self { ptr: slice.as_mut_ptr(), len: slice.len() as u32 }
  }

  unsafe fn into_owned(self) -> Box<[T]> {
    Box::from_raw(slice_from_raw_parts_mut(self.ptr, self.len as usize))
  }
}

#[repr(C)]
pub struct BusRepr {
  pub dram: Slice<u8>,
}

#[repr(C)]
pub struct CpuRepr {
  pub pc: u64,
  pub mode: Mode,
  pub xregs: RegPlace<u64>,
  // pub state: State, -- unsupported
  pub bus: BusRepr,
}

impl CpuRepr {
  pub unsafe fn map_to(self, cpu: &mut Cpu) {
    cpu.pc = self.pc;

    for (i, x) in
      self.xregs.regs.into_iter().enumerate().take(self.xregs.len as usize)
    {
      cpu.xregs.store(i as u64, x);
    }

    cpu.mode = self.mode.into();
    cpu.bus.dram.init(&self.bus.dram.into_owned());
  }

  pub fn from_cpu(cpu: &Cpu) -> Self {
    fn trim(slice: &[u8]) -> &[u8] {
      let trim =
        slice.iter().rev().position(|&x| x != 0).unwrap_or(slice.len());
      &slice[..slice.len() - trim]
    }
    let dram =
      Slice::new(trim(cpu.bus.dram.as_slice()).to_vec().into_boxed_slice());
    Self {
      pc: cpu.pc,
      mode: cpu.mode.into(),
      xregs: RegPlace { regs: cpu.xregs.into_inner(), len: REG_COUNT as u8 },
      bus: BusRepr { dram },
    }
  }
}

#[repr(C)]
pub struct EmuRepr {
  cpu: CpuRepr,
}

fn layout(len: u64) -> Layout {
  Layout::array::<u8>(len as usize).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn vmalloc(len: u64) -> *mut u8 {
  alloc(layout(len))
}

#[no_mangle]
pub unsafe extern "C" fn vfree(ptr: *mut u8, len: u64) {
  dealloc(ptr, layout(len))
}

#[no_mangle]
pub extern "C" fn vempty_emu(ram: u64) -> *mut Emu {
  Box::into_raw(Box::new(Emu::new(ram as usize)))
}

#[no_mangle]
pub unsafe extern "C" fn vfree_emu(emu: *mut Emu) {
  let _ = unsafe { Box::from_raw(emu) };
}

#[no_mangle]
pub unsafe extern "C" fn vmap_emu(emu: *mut Emu, repr: EmuRepr) {
  let emu = &mut *emu;
  repr.cpu.map_to(&mut emu.cpu);
}

#[no_mangle]
pub unsafe extern "C" fn vcycle_emu(emu: *mut Emu) -> Trap {
  let emu = &mut *emu;
  match emu.cycle() {
    Ok(_) => Trap::Requested,
    Err(ex) => emu.cpu.catch_exception(ex).into(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_repr(emu: *const Emu) -> CpuRepr {
  let emu = &*emu;
  CpuRepr::from_cpu(&emu.cpu)
}
