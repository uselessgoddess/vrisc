use {
  std::{fs, ops::RangeBounds, time::Instant},
  vrisc::{bus::dram, Emu, Exception, Xregs, REG_COUNT},
};

pub const SP: u64 = dram::END;

pub fn xregs<const N: usize>(regs: [(usize, u64); N]) -> [u64; REG_COUNT] {
  let mut xregs = Xregs::new().into_inner();
  for &(i, rx) in regs.iter() {
    xregs[i] = rx;
  }
  xregs
}

#[rustfmt::skip]
macro_rules! xreg {
  (x0) => {0};
  (x1) => {1};
  (x2) => {2};
  (x3) => {3};
  (x4) => {4};
}

macro_rules! xregs {
  ($($reg:ident = $eq:literal),* $(,)?) => {
    xregs([$((xreg!($reg), $eq)),*])
  };
}

fn main() {
  let mut emu = Emu::new(1024 * 1024); // 1Mb ram

  let data = [
    0x93, 0x01, 0x50, 0x00, // addi x3, x0, 5
    0x13, 0x02, 0x60, 0x00, // addi x4, x0, 6
    0x33, 0x81, 0x41, 0x00, // add x2, x3, x4
  ];

  // run(&mut emu, &data, xregs!(x2 = 11, x3 = 5, x4 = 6));

  {
    let bin = fs::read("rv32-addi.bin").unwrap();
    // let bin = fs::read("riscv-tests/isa/rv32ui-p-addi.bin").unwrap();
    run(&mut emu, &bin, xregs!());
  }
}

fn run(emu: &mut Emu, data: &[u8], xregs: [u64; REG_COUNT]) {
  emu.with_dram(data).with_pc(dram::ADDR);

  let start = dram::ADDR;
  let end = start + data.len() as u64;
  loop {
    if !(start..end).contains(&emu.cpu.pc) {
      break;
    }

    match emu.cycle() {
      Ok(inst) => {
        println!("pc: {:x}, inst: {inst:#x}", emu.cpu.pc.wrapping_sub(4));
      }
      Err(ex) => {
        println!(
          "pc: {pc:#x}, inst: {inst:#x}",
          inst = emu.cpu.fetch(32).unwrap(),
          pc = emu.cpu.pc,
        );

        if let Exception::IllegalInst(inst) = ex {
          panic!("invalid instruction: 0x{inst:x}");
        }

        emu.cpu.catch_exception(ex);

        std::thread::sleep_ms(100);
      }
    }
    println!("{:?}", emu.cpu.xregs);
  }

  for (i, &rx) in xregs.iter().enumerate() {
    assert_eq!(rx, emu.cpu.xregs.load(i as u64), "fails at {i}");
  }
}
