use {
  std::{
    alloc::{alloc, dealloc, Layout},
    ffi::c_void,
    mem,
    ptr::slice_from_raw_parts_mut,
    slice,
  },
  vrisc::{Cpu, Emu, REG_COUNT},
};

pub use vrisc;

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
pub struct Slice<T> {
  ptr: *const T,
  len: u32,
}

impl<T> Slice<T> {
  unsafe fn as_slice(&self) -> &[T] {
    slice::from_raw_parts(self.ptr, self.len as usize * size_of::<T>())
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
  pub xregs: [u64; REG_COUNT],
  // pub state: State, -- unsupported
  pub bus: BusRepr,
}

impl CpuRepr {
  pub unsafe fn map_to(self, cpu: &mut Cpu) {
    cpu.pc = self.pc;

    for (i, x) in self.xregs.into_iter().enumerate() {
      cpu.xregs.store(i as u64, x);
    }

    cpu.mode = self.mode.into();
    cpu.bus.dram.init(self.bus.dram.as_slice());
  }

  pub fn from_cpu(cpu: &Cpu) -> Self {
    let slice = cpu.bus.dram.as_slice().to_vec().into_boxed_slice();
    let (ptr, len) = (slice.as_ptr(), slice.len() as u32);
    mem::forget(slice);

    Self {
      pc: cpu.pc,
      mode: cpu.mode.into(),
      xregs: cpu.xregs.into_inner(),
      bus: BusRepr { dram: Slice { ptr, len } },
    }
  }
}

#[repr(C)]
pub struct EmuRepr {
  cpu: CpuRepr,
}

#[no_mangle]
pub unsafe extern "C" fn vfree_array_u8(ptr: *mut c_void, len: u32) {
  unsafe {
    let repr = slice_from_raw_parts_mut(ptr as *mut u8, len as usize);
    let _ = Box::from_raw(repr);
  }
}

#[no_mangle]
pub extern "C" fn vempty_emu(ram: u64) -> *mut c_void {
  Box::into_raw(Box::new(Emu::new(ram as usize))).cast()
}

#[no_mangle]
pub extern "C" fn vfree_emu(emu: *mut c_void) {
  let _ = unsafe { Box::from_raw(emu) };
}

#[no_mangle]
pub unsafe extern "C" fn vmap_emu(emu: *mut c_void, repr: EmuRepr) {
  let mut emu = unsafe { &mut *(emu as *mut Emu) };

  repr.cpu.map_to(&mut emu.cpu);
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
