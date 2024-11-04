#![feature(stmt_expr_attributes)]

use {
  std::{
    alloc::{alloc, dealloc, Layout},
    mem::ManuallyDrop,
    ptr::slice_from_raw_parts_mut,
  },
  vrisc::{Cpu, Emu, REG_COUNT},
  yuvutils_rs::{YuvRange, YuvStandardMatrix},
};

pub use vrisc;
use vrisc::{
  csr,
  dev::vga::{VGA_HEIGHT, VGA_WIDTH},
};

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

use {rav1e::prelude::*, yuvutils_rs::rgb_to_yuv444};

type Px = u8;
type Av1 = rav1e::Context<Px>;

struct Context {
  emu: Emu,
  av1: Av1,
  packets: Vec<Packet<Px>>,
}

pub fn ctx() -> Config {
  Config::new().with_encoder_config(EncoderConfig {
    width: VGA_WIDTH as usize,
    height: VGA_HEIGHT as usize,
    chroma_sampling: ChromaSampling::Cs444,
    ..Default::default()
  })
}

impl Context {
  pub fn new(ram: usize) -> Self {
    Self {
      packets: vec![],
      emu: Emu::new(ram),
      av1: ctx().new_context().unwrap(),
    }
  }
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
pub extern "C" fn vempty_emu(ram: u64) -> *mut Context {
  Box::into_raw(Box::new(Context::new(ram as usize)))
}

#[no_mangle]
pub unsafe extern "C" fn vfree_emu(ctx: *mut Context) {
  let _ = unsafe { Box::from_raw(ctx) };
}

#[no_mangle]
pub unsafe extern "C" fn vmap_emu(ctx: *mut Context, repr: EmuRepr) {
  let ctx = &mut *ctx;
  repr.cpu.map_to(&mut ctx.emu.cpu);
}

#[no_mangle]
pub unsafe extern "C" fn vcycle_emu(ctx: *mut Context) -> Trap {
  let ctx = &mut *ctx;

  const SYNC: u64 = 131_072;

  if ctx.emu.cpu.state.load(csr::TIME) % SYNC == 0 {
    let av1 = &mut ctx.av1;
    let mut frame = av1.new_frame();
    let Frame { planes: [y, u, v] } = &mut frame;

    let rgb_stride = VGA_WIDTH as u32;

    #[rustfmt::skip]
    rgb_to_yuv444(
      &mut y.data, y.cfg.stride as u32,
      &mut u.data, u.cfg.stride as u32,
      &mut v.data, v.cfg.stride as u32,
      ctx.emu.cpu.bus.vga.buf.as_slice(), rgb_stride,
      VGA_WIDTH as u32,
      VGA_HEIGHT as u32,
      YuvRange::Full,
      YuvStandardMatrix::Bt601,
    );

    av1.send_frame(frame).unwrap();

    recv_frames(av1, &mut ctx.packets).unwrap();
  }

  match ctx.emu.cycle() {
    Ok(_) => Trap::Requested,
    Err(ex) => ctx.emu.cpu.catch_exception(ex).into(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_repr(ctx: *const Context) -> CpuRepr {
  let ctx = &*ctx;
  CpuRepr::from_cpu(&ctx.emu.cpu)
}

#[no_mangle]
pub unsafe extern "C" fn vrecv_packets(ctx: *mut Context) -> Slice<Slice<u8>> {
  let ctx = &mut *ctx;

  let into_slice =
    move |packet: Packet<Px>| Slice::new(packet.data.into_boxed_slice());
  Slice::new(ctx.packets.drain(..).map(into_slice).collect())
}

fn recv_frames(
  av1: &mut Av1,
  packets: &mut Vec<Packet<Px>>,
) -> Result<(), EncoderStatus> {
  loop {
    match av1.receive_packet() {
      Ok(packet) => packets.push(packet),
      Err(EncoderStatus::LimitReached) => break,
      Err(EncoderStatus::Failure) => return Err(EncoderStatus::Failure),
      _ => {}
    }
  }
  Ok(())
}

fn encode_to_fit(
  av1: &mut Av1,
  packets: &mut Vec<Packet<Px>>,
) -> Result<(), EncoderStatus> {
  loop {
    match av1.receive_packet() {
      Ok(packet) => packets.push(packet),
      Err(EncoderStatus::NeedMoreData) => av1.flush(),
      Err(EncoderStatus::LimitReached) => break,
      Err(EncoderStatus::Failure) => return Err(EncoderStatus::Failure),
      _ => {}
    }
  }
  Ok(())
}

#[test]
fn f() {
  let x = Context::new(12).av1.new_frame();
  println!("{}", x.planes[2].cfg.stride);
}
